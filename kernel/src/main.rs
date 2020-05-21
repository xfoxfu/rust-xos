#![no_std]
#![no_main]
#![feature(llvm_asm)]

use boot::BootInfo;
#[cfg(not(test))]
use core::panic::PanicInfo;

#[macro_use]
mod console;
mod display;
mod logging;
mod uefi_clock;

extern crate rlibc;
#[macro_use]
extern crate log;

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
    display::initialize(&boot_info.graphic_info);
    _svc!(display::DISPLAY :mut).clear();

    console::initialize();
    println!("console initialized");

    logging::initialize();
    info!("logging initialized");

    let rs = unsafe { boot_info.system_table.runtime_services() };

    uefi_clock::initialize(rs);
    info!(
        "uefi clock initialized, now = {}",
        _svc!(uefi_clock::UEFI_CLOCK).now()
    );

    info!(
        "kernel loaded, firmware vendor={:?} version={:?}",
        boot_info.system_table.firmware_vendor(),
        boot_info.system_table.firmware_revision()
    );

    for mem in boot_info.memory_map.clone().iter {
        if mem.ty == boot::MemoryType::CONVENTIONAL {
            println!("{:x?}", mem);
        }
    }

    info!("kernel exit, shutdown in 10s");

    _svc!(uefi_clock::UEFI_CLOCK).spin_wait_for_ns(10_000_000_000i64);

    unsafe {
        boot_info.system_table.runtime_services().reset(
            boot::ResetType::Shutdown,
            boot::UefiStatus::SUCCESS,
            None,
        );
    }
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("PANIC - {}", info);
    loop {}
}
