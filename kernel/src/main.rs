#![no_std]
#![no_main]
#![feature(unsafe_block_in_unsafe_fn)]
#![feature(type_alias_impl_trait)]
#![feature(naked_functions)]
#![feature(core_intrinsics)]
#![feature(asm)]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]
#![warn(unsafe_op_in_unsafe_fn)]

use boot::BootInfo;
use x86_64::VirtAddr;

#[macro_use]
mod macros;
#[macro_use]
mod console;

mod allocator;
mod apps;
mod display;
mod driver_holder;
mod drivers;
mod gdt;
mod interrupts;
mod logging;
mod memory;
mod process;
mod uefi_clock;
mod utils;

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
    gdt::init();

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
        uefi_clock::get_clock_sure().now()
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

    // 初始化进程管理
    process::init();

    // 开中断
    x86_64::instructions::interrupts::enable();

    // 内核加载完成
    info!(
        "kernel loaded, firmware vendor={} version={:?}",
        boot_info.system_table.firmware_vendor(),
        boot_info.system_table.firmware_revision()
    );

    unsafe {
        drivers::ide::init();
        drivers::filesystem::init();
    }

    let exit_code = apps::shell_main(boot_info);
    info!("init process exit = {}, shutdown in 5s", exit_code);
    uefi_clock::get_clock_sure().spin_wait_for_ns(5_000_000_000);

    unsafe {
        boot_info.system_table.runtime_services().reset(
            boot::ResetType::Shutdown,
            boot::UefiStatus::SUCCESS,
            None,
        );
    }
}
