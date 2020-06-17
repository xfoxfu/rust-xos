use super::MutexIDE;
use crate::alloc::borrow::ToOwned;
use crate::drivers::IDE;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use boot::BootInfo;
use spin::Mutex;
use x86_64::structures::paging::FrameAllocator;

fn print_sector(sector: &[u8]) {
    assert_eq!(sector.len(), 512);
    for i in 0..32 {
        for j in 0..4 {
            for k in 0..4 {
                print!("{:02x}", sector[i * 16 + j * 4 + k]);
            }
            print!(" ");
        }
        println!();
    }
}

fn list(ide: &mut IDE) -> Vec<String> {
    let mut buf = vec![0; 512];
    ide.read_lba(0, 1, &mut buf).expect("failed to read disk");

    buf.resize(buf.iter().position(|v| v == &b'\0').unwrap(), 0);

    let entire = String::from_utf8(buf).expect("executables list is not UTF-8");
    entire
        .split('\n')
        .filter(|v| v.len() > 0)
        .map(str::to_owned)
        .collect()
}

fn run_program(id: u32, boot_info: &'static BootInfo, ide: &mut IDE) {
    info!("loading file {} to memory", id);
    let buf = {
        let pages = 4;
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
        ide.read_lba(1 + id * 32, pages as u8 * 8, &mut buf)
            .unwrap();
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

    debug!("jump to {:x}", elf.header.pt2.entry_point());
    trace!("inst = {:016x}", unsafe {
        *(elf.header.pt2.entry_point() as *mut u64)
    });
    crate::uefi_clock::get_clock_sure().spin_wait_for_ns(1_000_000_000);
    *crate::interrupts::get_user_running_sure() = true;
    unsafe {
        asm!("call {}", in(reg) elf.header.pt2.entry_point()/* , in(reg) stacktop*/, in("rdi") boot_info);
    }
    *crate::interrupts::get_user_running_sure() = false;

    elf_loader::unmap_elf(&elf, &mut *crate::memory::get_page_table_sure())
        .expect("failed to unload elf");
}

fn print_help(progs: &Vec<String>) {
    println!("Programs:");
    for (v, p) in progs.iter().enumerate() {
        println!("{} - {}", v, p);
    }
    println!(
        "Others:
q - quit
h - help"
    )
}

fn main_iter(boot_info: &'static BootInfo, ide: &mut IDE, progs: &Vec<String>) -> bool {
    use fatpart::*;

    let mutex = Mutex::new(IDE::from_id(0));
    let ide = MutexIDE(&mutex);
    ide.0.lock().init().unwrap();

    let mut sector = vec![0; 512];

    // load partition table
    ide.0.lock().read_lba(0, 1, &mut sector).unwrap();
    let partition = fatpart::MBRPartitionTable::parse_sector(&sector)
        .unwrap()
        .partition0;
    info!(
        "parsed partition, begin={}, total={}",
        partition.begin_lba, partition.total_lba
    );

    let fat = FATPartition::new(Partition::new(&ide, partition));
    let mut root = fat.root_directory();
    info!("childs = {:?}", root.load_childs().unwrap());

    return false;
}

pub fn main(boot_info: &'static BootInfo, ide: &mut IDE) -> u8 {
    let progs = list(ide);

    print_help(&progs);

    while main_iter(boot_info, ide, &progs) {}

    0
}
