use crate::{
    interrupts::Registers,
    memory::{physical_to_virtual, BootInfoFrameAllocator},
};
use alloc::vec::Vec;
use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::registers::rflags::RFlags;
use x86_64::structures::idt::{InterruptStackFrame, InterruptStackFrameValue};
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

#[derive(Debug)]
pub struct Process {
    id: usize,
    state: ProcessState,
    state_isf: InterruptStackFrameValue,
    state_reg: Registers,
    /// 页表所处地址
    page_table_addr: (PhysFrame, Cr3Flags),
    /// 若非内核进程，则具备独立页表及其控制，否则没有
    page_table: Option<OffsetPageTable<'static>>, // 实际生命周期和 Process 一致
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProcessState {
    Ready,
    Running,
}

impl Process {
    pub fn new(frame_alloc: &mut BootInfoFrameAllocator, id: usize) -> Self {
        // 1. 为进程创建新的页表
        let new_frame = frame_alloc
            .allocate_frame()
            .expect("cannot alloc page table for new process");
        let page_table_addr = new_frame;
        // 1.1. 复制当前页表
        unsafe {
            rlibc::memcpy(
                physical_to_virtual(page_table_addr.start_address().as_u64() as usize) as *mut u8,
                physical_to_virtual(Cr3::read().0.start_address().as_u64() as usize) as *const u8,
                core::mem::size_of::<PageTable>(),
            );
        }
        // 1.2. 构建页表对象
        let page_table_raw = unsafe {
            (physical_to_virtual(page_table_addr.start_address().as_u64() as usize)
                as *mut PageTable)
                .as_mut()
        }
        .unwrap();
        let page_table = unsafe {
            OffsetPageTable::new(
                page_table_raw,
                VirtAddr::new_truncate(crate::memory::PHYSICAL_OFFSET),
            )
        };
        // 2. 创建伪上下文信息
        let state = ProcessState::Ready;
        let state_isf = InterruptStackFrameValue {
            instruction_pointer: VirtAddr::new_truncate(0),
            code_segment: 0,
            cpu_flags: 0,
            stack_pointer: VirtAddr::new_truncate(0),
            stack_segment: 0,
        };
        let state_reg = Registers::default();
        // 3. 返回
        Self {
            id,
            state,
            state_isf,
            state_reg,
            page_table_addr: (page_table_addr, Cr3::read().1),
            page_table: Some(page_table),
        }
    }
}

impl Process {
    pub fn id(&self) -> usize {
        self.id
    }
    pub fn state_isf_mut(&mut self) -> &mut InterruptStackFrameValue {
        &mut self.state_isf
    }
    pub fn page_table_mut(&mut self) -> &mut OffsetPageTable<'static> {
        self.page_table.as_mut().unwrap()
    }
    pub fn pause(&mut self) {
        self.state = ProcessState::Ready;
    }
    pub fn resume(&mut self) {
        self.state = ProcessState::Running;
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        // TODO: deallocate memory
    }
}

once_mutex!(pub PROCESS_LIST: Vec<Process>);
guard_access_fn! {
    pub get_process_list(PROCESS_LIST: Vec<Process>)
}

/// 初始化进程系统，需要保证内存已经正确初始化
pub fn init() {
    init_PROCESS_LIST(Vec::new());
    let mut alloc = crate::memory::get_frame_alloc_sure();
    let mut list = get_process_list_sure();
    // 内核伪进程
    let mut kproc = Process::new(&mut *alloc, list.len());
    kproc.state = ProcessState::Running;
    // 对于内核，其页表只是伪作，实际上还是要使用 `memory` 模块的页表
    kproc.page_table_addr = Cr3::read();
    list.push(kproc);
    info!("process manager initialized");
}

/// 创建进程，不会自动切换过去
pub fn spawn_process(entry: VirtAddr, stacktop: VirtAddr) {
    let mut list = get_process_list_sure();
    let mut proc = Process::new(
        &mut *crate::memory::get_frame_alloc_sure(),
        list.last().unwrap().id + 1,
    );
    proc.state = ProcessState::Ready;
    proc.state_isf_mut().instruction_pointer = entry;
    proc.state_isf_mut().stack_pointer = stacktop;
    // 参考自 rCore 设置进程初始 flags
    proc.state_isf.cpu_flags =
        (RFlags::IOPL_HIGH | RFlags::IOPL_LOW | RFlags::INTERRUPT_FLAG).bits();
    list.push(proc);
}

/// 结束进程，执行前确保已经切换到有效进程上下文中
pub fn kill_current_process() {
    get_process_list_sure().retain(|p| p.state != ProcessState::Running);
}

/// 将给定的中断栈帧和寄存器切换到第一个就绪进程
pub fn switch_first_ready_process(sf: &mut InterruptStackFrame, regs: &mut Registers) {
    // 1. 暂停当前正在运行的进程，并保存其状态
    let prev_id = save_current_process(sf, regs);
    // info!("paused {:?}", prev_id);
    // 2. 寻找可以运行的进程
    let mut list = get_process_list_sure();
    if let Some(proc) = find_next_process(&mut *list, prev_id.unwrap_or(0)) {
        proc.state = ProcessState::Running;
        // 2b. 若非当前进程，则需要进行切换
        if prev_id == None || proc.id != prev_id.unwrap() {
            unsafe {
                let sf_mut = sf.as_mut();
                sf_mut.instruction_pointer = proc.state_isf.instruction_pointer;
                sf_mut.stack_pointer = proc.state_isf.stack_pointer;
                sf_mut.cpu_flags = proc.state_isf.cpu_flags;
                *regs = proc.state_reg.clone();
                // 更新 Cr3 后会自动刷新 TLB
                Cr3::write(proc.page_table_addr.0, proc.page_table_addr.1);
            };
        }
        trace!(
            "switched to process {} {:?} {:?} {:?}",
            proc.id,
            sf,
            regs,
            Cr3::read()
        );
    } else {
        // 2c. 是当前进程则只需要修改状态记录，不需要切换
        list.iter_mut()
            .find(|p| p.id == prev_id.unwrap())
            .unwrap()
            .state = ProcessState::Running;
    }
}

pub fn save_current_process(sf: &mut InterruptStackFrame, regs: &mut Registers) -> Option<usize> {
    let mut list = get_process_list_sure();
    // a. 若存在正在运行的进程，则保存状态
    if let Some(running_proc) = list
        .iter_mut()
        .filter(|p| p.state == ProcessState::Running)
        .next()
    {
        running_proc.state = ProcessState::Ready;
        running_proc.state_isf = sf.clone();
        running_proc.state_reg = regs.clone();
        Some(running_proc.id)
    } else {
        // b. 否则不保存状态
        None
    }
}

fn find_next_process(list: &mut Vec<Process>, prev: usize) -> Option<&mut Process> {
    if list.first().unwrap().state == ProcessState::Running || list.len() <= 1 {
        // 若内核正在运行，或没有用户程序，则应当切换到内核
        Some(list.first_mut().unwrap())
    } else {
        // 否则，切换到第一个可用的用户程序
        let next_pos = list.iter().position(|p| p.id == prev).unwrap() + 1;
        if next_pos >= list.len() {
            if prev != 1 {
                Some(&mut list[1])
            } else {
                None
            }
        } else {
            Some(&mut list[next_pos])
        }
    }
}
