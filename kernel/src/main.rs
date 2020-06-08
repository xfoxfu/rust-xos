#![no_std]
#![no_main]
#![feature(llvm_asm, abi_x86_interrupt, alloc_error_handler)]
#![feature(type_alias_impl_trait)]

use boot::BootInfo;
use x86_64::{structures::paging::FrameAllocator, VirtAddr};

#[macro_use]
mod macros;
#[macro_use]
mod console;

mod allocator;
mod display;
mod drivers;
mod interrupts;
mod logging;
mod memory;
mod uefi_clock;

extern crate alloc;
extern crate rlibc;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;

macro_rules! _svc {
    ($t: path) => {
        $t.lock().as_ref().unwrap()
    };
    ($t: path :mut) => {
        $t.lock().as_mut().unwrap()
    };
}

boot::entry_point!(kmain);

pub fn kmain(boot_info: &'static BootInfo) -> ! {
    // 初始化显示驱动
    display::initialize(&boot_info.graphic_info);
    display::get_display_sure().clear();

    // 初始化图形终端
    console::initialize();
    println!("console initialized");

    // 初始化日志系统
    logging::initialize();
    info!("logging initialized");

    // 初始化中断（CPU 异常、时钟）
    unsafe {
        interrupts::init();
    }
    info!("interrupts initialized");

    // 加载 UEFI 相关特性
    let rs = unsafe { boot_info.system_table.runtime_services() };

    uefi_clock::initialize(rs);
    info!(
        "uefi clock initialized, now = {}",
        uefi_clock::get_clock()
            .expect("UEFI clock not initialized")
            .now()
    );

    // 初始化帧分配器
    unsafe {
        memory::init(
            VirtAddr::new_truncate(memory::PHYSICAL_OFFSET as u64),
            &boot_info.memory_map,
        );
    }
    // 初始化堆内存
    allocator::init_heap(
        &mut *memory::get_page_table_sure(),
        &mut *memory::get_frame_alloc_sure(),
    )
    .unwrap();

    info!("memory allocator initialized");

    // 初始化键盘驱动
    unsafe {
        drivers::keyboard::init();
    }

    // 内核加载完成
    info!(
        "kernel loaded, firmware vendor={} version={:?}",
        boot_info.system_table.firmware_vendor(),
        boot_info.system_table.firmware_revision()
    );

    let mut ide = drivers::ide::IDE::from_id(1);
    ide.init().unwrap();

    for i in 0..4 {
        info!("loading file to memory");
        let buf = {
            let pages = 4;
            let mem_start = memory::get_frame_alloc_sure()
                .allocate_frame()
                .unwrap()
                .start_address()
                .as_u64();
            debug!("alloc = {}", mem_start);
            for i in 1..pages {
                let addr = memory::get_frame_alloc_sure()
                    .allocate_frame()
                    .unwrap()
                    .start_address()
                    .as_u64();
                debug!("alloc = {}", addr);
            }
            let mut buf =
                unsafe { core::slice::from_raw_parts_mut(mem_start as *mut u8, pages * 0x1000) };
            info!("read = {}", pages as u8 * 8);
            ide.read_lba(1 + i * 32, pages as u8 * 8, &mut buf).unwrap();
            &mut buf[..pages * 0x1000]
        };
        info!(
            "loaded = {:02x}{:02x}{:02x}{:02x} | {:02x}{:02x}{:02x}{:02x}",
            buf[0x0000 + 0],
            buf[0x0000 + 1],
            buf[0x0000 + 2],
            buf[0x0000 + 3],
            buf[0x1000 + 0],
            buf[0x1000 + 1],
            buf[0x1000 + 2],
            buf[0x1000 + 3]
        );

        let elf = xmas_elf::ElfFile::new(&buf).unwrap();
        elf_loader::map_elf(
            &elf,
            &mut *memory::get_page_table_sure(),
            &mut *memory::get_frame_alloc_sure(),
        )
        .unwrap();

        // elf_loader::map_stack(
        //     //    0xFFFF_FF01_0000_0000
        //     0x0000_1101_0000_0000,
        //     //    0x0000_1111_0000_0000
        //     512,
        //     memory::OFFSET_PAGE_TABLE.lock().as_mut().unwrap(),
        //     memory::FRAME_ALLOCATOR.lock().as_mut().unwrap(),
        // )
        // .expect("failed to map stack");
        // let stacktop: usize = 0x0000_1101_0000_0000 + 510 * 0x1000;

        info!("wait for 1s and jump to {:x}", elf.header.pt2.entry_point());
        info!("inst = {:016x}", unsafe {
            *(elf.header.pt2.entry_point() as *mut u64)
        });
        uefi_clock::get_clock_sure().spin_wait_for_ns(1_000_000_000);
        unsafe {
            llvm_asm!("call $0"
                :: "r"(elf.header.pt2.entry_point())/* , "{rsp}"(stacktop) */, "{rdi}"(boot_info)
                :: "intel");
        }

        elf_loader::unmap_elf(&elf, &mut *memory::get_page_table_sure())
            .expect("failed to unload elf");
    }

    info!("kernel exit, shutdown in 5s");
    uefi_clock::get_clock_sure().spin_wait_for_ns(5_000_000_000);

    unsafe {
        boot_info.system_table.runtime_services().reset(
            boot::ResetType::Shutdown,
            boot::UefiStatus::SUCCESS,
            None,
        );
    }
}
