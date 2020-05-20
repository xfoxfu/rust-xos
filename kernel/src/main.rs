#![no_std]
#![no_main]
#![feature(llvm_asm)]

use boot::BootInfo;
use core::panic::PanicInfo;

mod console;
mod display;
mod log;

extern crate rlibc;

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    use display::GOPDisplay;

    static mut DISPLAY: Option<GOPDisplay> = None;

    unsafe {
        DISPLAY = Some(GOPDisplay::new(&boot_info.graphic_info));
    }

    unsafe {
        let mut console = console::Console::new(DISPLAY.as_mut().unwrap());
        console.write("Hello world!\nLorem ipsum\rdolor");
    }

    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
