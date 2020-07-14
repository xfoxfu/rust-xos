use crate::{
    interrupts::Registers,
    memory::{physical_to_virtual, BootInfoFrameAllocator},
};
use alloc::vec::Vec;
use x86_64::structures::idt::{InterruptStackFrame, InterruptStackFrameValue};
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

#[derive(Debug)]
pub struct Process {
    id: usize,
    state: ProcessState,
    state_isf: InterruptStackFrameValue,
    state_reg: Registers,
    page_table_addr: PhysAddr,
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
        let page_table_addr = new_frame.start_address();
        // 1.1. 复制当前页表
        unsafe {
            rlibc::memcpy(
                physical_to_virtual(page_table_addr.as_u64() as usize) as *mut u8,
                physical_to_virtual(
                    x86_64::registers::control::Cr3::read()
                        .0
                        .start_address()
                        .as_u64() as usize,
                ) as *const u8,
                core::mem::size_of::<PageTable>(),
            );
        }
        // 1.2. 构建页表对象
        let page_table_raw = unsafe {
            (physical_to_virtual(page_table_addr.as_u64() as usize) as *mut PageTable).as_mut()
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
            page_table_addr,
            page_table: Some(page_table),
        }
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
    // unsafe {
    //     let (_, cr3f) = x86_64::registers::control::Cr3::read();
    //     x86_64::registers::control::Cr3::write(
    //         PhysFrame::from_start_address_unchecked(kproc.page_table_addr),
    //         cr3f,
    //     );
    // }
    list.push(kproc);
}

/// 创建进程，不会自动切换过去
pub fn spawn_process() {
    let mut list = get_process_list_sure();
    let mut proc = Process::new(&mut *crate::memory::get_frame_alloc_sure(), list.len());
    proc.state = ProcessState::Running;
    list.push(proc);
}

/// 结束进程，执行前确保已经切换到有效进程上下文中
pub fn kill_process(id: usize) {
    let mut list = get_process_list_sure();
    if id >= list.len() {
        panic!("kill invalid process");
    }
    list.remove(id);
}

/// 将给定的中断栈帧和寄存器切换到第一个就绪进程
pub fn switch_first_ready_process(sf: &mut InterruptStackFrame, regs: &mut Registers) {
    let mut list = get_process_list_sure();
    // 1. 暂停当前正在运行的进程，并保存其状态
    let running_proc = list
        .iter_mut()
        .filter(|p| p.state == ProcessState::Running)
        .next()
        .unwrap();
    running_proc.state = ProcessState::Ready;
    running_proc.state_isf = sf.clone();
    running_proc.state_reg = regs.clone();
    let previous_proc_id = running_proc.id;
    trace!("paused process {}", previous_proc_id);
    // 2. 寻找可以运行的进程
    // 2.a. 若存在可以运行的进程，则执行进程
    if let Some(proc) = list
        .iter_mut()
        .filter(|p| p.state == ProcessState::Ready && p.id != previous_proc_id)
        .next()
    {
        proc.state = ProcessState::Running;
        unsafe {
            *sf.as_mut() = proc.state_isf.clone();
            *regs = proc.state_reg.clone();
        }
        trace!("switched to process {}", proc.id);
    } else if list.len() > 1 {
        list[previous_proc_id].state = ProcessState::Running;
        trace!("restore process {}", previous_proc_id);
    } else {
        // 2.b. 若不存在，则切换到内核运行
        let kproc = list.first_mut().unwrap();
        kproc.state = ProcessState::Running;
        if kproc.id == previous_proc_id {
            // 当内核就是当前运行进程时，就不需要恢复上下文了
            trace!("reset kernel to running");
        } else {
            unsafe {
                *sf.as_mut() = kproc.state_isf.clone();
                *regs = kproc.state_reg.clone();
            }
            trace!("switched to kernel {}", 0);
        }
    }
}
