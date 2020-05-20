#![no_std]
#![no_main]
#![feature(llvm_asm)]

use boot::BootInfo;
use core::panic::PanicInfo;

#[macro_use]
mod console;
mod display;
mod logging;

extern crate rlibc;
#[macro_use]
extern crate log;

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    display::initialize(&boot_info.graphic_info);
    display::DISPLAY.lock().as_mut().unwrap().clear();

    console::initialize();
    logging::initialize();

    println!("Hello world!");
    info!("hello world");
    // println!("{:#x?}", boot_info);
    println!("Hello world!");
    warn!("some warning");
    println!("Hello world!");

    for i in 0..100 {
        println!("Hello world! i={}", i);
    }

    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("PANIC - {}", info);
    loop {}
}
