use crate::drivers::{fs, OsDevice, OsFile};
use alloc::vec::Vec;
use boot::BootInfo;
use fatpart::{Entry, File};
use x86_64::{
    instructions::interrupts, registers::rflags::RFlags, structures::paging::FrameAllocator,
    VirtAddr,
};

fn list() -> Vec<File<'static, OsDevice>> {
    fs().root_directory()
        .load_childs()
        .unwrap()
        .into_iter()
        .filter_map(|e| match e {
            Entry::Dir(_) => None,
            Entry::File(f) => Some(f),
        })
        .collect()
}

fn run_program_prepare() {
    // 关中断，避免进程未创建好就被切换
    interrupts::disable();
}

fn run_program(file: &OsFile, boot_info: &'static BootInfo) {
    info!("loading file {} to memory", file.entry.stem());
    let sectors = file.sectors();
    let buf = {
        let pages = (sectors.len() + 7) / 8;
        // 分配内存帧
        let mem_start = crate::memory::get_frame_alloc_sure()
            .allocate_frame()
            .unwrap()
            .start_address()
            .as_u64();
        trace!("alloc = {}", mem_start);
        for _ in 1..pages {
            let addr = crate::memory::get_frame_alloc_sure()
                .allocate_frame()
                .unwrap()
                .start_address()
                .as_u64();
            trace!("alloc = {}", addr);
        }
        // 加载磁盘内容
        let mut buf =
            unsafe { core::slice::from_raw_parts_mut(mem_start as *mut u8, pages * 0x1000) };

        file.load_to(&mut buf).unwrap();
        &mut buf[..pages * 0x1000]
    };

    // 解析 ELF 文件
    let elf = xmas_elf::ElfFile::new(&buf).unwrap();

    const STACK_BOT: u64 = 0x0000_2000_0000_0000;
    const STACK_PAGES: u64 = 512;
    const STACK_TOP: u64 = STACK_BOT + STACK_PAGES * 0x1000;

    crate::process::spawn_process(
        VirtAddr::new_truncate(elf.header.pt2.entry_point()),
        VirtAddr::new_truncate(STACK_TOP),
    );

    let mut list = crate::process::get_process_list_sure();
    let proc = list.last_mut().unwrap();

    // 映射到页表
    elf_loader::map_elf(
        &elf,
        proc.page_table_mut(),
        &mut *crate::memory::get_frame_alloc_sure(),
    )
    .unwrap();

    elf_loader::map_stack(
        STACK_BOT,
        STACK_PAGES,
        proc.page_table_mut(),
        &mut *crate::memory::get_frame_alloc_sure(),
    )
    .expect("failed to map stack");
}

fn run_program_launch() {
    // 开中断
    interrupts::enable();
    unsafe {
        asm!("
                push rbp
                int {id}
                pop rbp",
            id = const 0x80,
            in("rax") crate::interrupts::Syscall::SpawnProcess as u64,
            in("rbx") 0,
            in("rcx") 0,
        );
    }
}

fn run_process(entry: u64, stacktop: u64) {
    unsafe {
        asm!("
            push rbp
            int {id}
            pop rbp",
            id = const 0x80,
            in("rax") crate::interrupts::Syscall::SpawnProcess as u64,
            in("rbx") entry,
            in("rcx") stacktop,
        );
    }
}

fn print_help(progs: &[OsFile]) {
    println!("Programs:");
    for (v, p) in progs.iter().enumerate() {
        println!(
            "{} - {}.{} {}B",
            v,
            p.entry.stem(),
            p.entry.ext(),
            p.entry.size
        );
    }
    println!(
        "Others:
q - quit
h - help"
    )
}

fn main_iter(boot_info: &'static BootInfo, progs: &[OsFile]) -> bool {
    print!("> ");
    let prog = crate::drivers::keyboard::getline_block();

    let cflags = run_program_prepare();

    for c in prog.chars() {
        match c {
            '0'..='9' => {
                let id = c as u32 - '0' as u32;
                if (id as usize) < progs.len() {
                    run_program(&progs[id as usize], boot_info);
                } else {
                    println!("unknown process {}", id)
                }
            }
            'h' => print_help(progs),
            'q' => return false,
            _ => (),
        }
    }

    run_program_launch();

    true
}

pub fn main(boot_info: &'static BootInfo) -> u8 {
    let progs = list();

    print_help(&progs);

    while main_iter(boot_info, &progs) {}

    0
}
