use crate::drivers::{fs, OsDevice, OsFile};
use alloc::vec::Vec;
use boot::BootInfo;
use fatpart::{Entry, File};
use x86_64::structures::paging::FrameAllocator;

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
    // 映射到页表
    elf_loader::map_elf(
        &elf,
        &mut *crate::memory::get_page_table_sure(),
        &mut *crate::memory::get_frame_alloc_sure(),
    )
    .unwrap();
    // temporarily disable stack relocation
    // FIXME: enable stack relocation after GDT set up
    if false {
        elf_loader::map_stack(
            0x0000_2000_0000_0000,
            512,
            &mut *crate::memory::get_page_table_sure(),
            &mut *crate::memory::get_frame_alloc_sure(),
        )
        .expect("failed to map stack");
        let _stacktop = 0x0000_2000_0000_0000u64 + 512 * 0x1000;
    }

    debug!("jump to {:x}", elf.header.pt2.entry_point());
    trace!("inst = {:016x}", unsafe {
        *(elf.header.pt2.entry_point() as *mut u64)
    });

    run_process(elf.header.pt2.entry_point(), 0);
    info!("process exited");

    elf_loader::unmap_elf(&elf, &mut *crate::memory::get_page_table_sure())
        .expect("failed to unload elf");
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

    true
}

pub fn main(boot_info: &'static BootInfo) -> u8 {
    let progs = list();

    print_help(&progs);

    while main_iter(boot_info, &progs) {}

    0
}
