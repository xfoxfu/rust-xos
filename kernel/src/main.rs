#![no_std]
#![no_main]
#![feature(llvm_asm)]

use boot::BootInfo;
use core::panic::PanicInfo;
use embedded_graphics::pixelcolor::Rgb888;

mod console;
mod display;
mod log;

extern crate rlibc;

static HELLO: &[u8] = b"Hello World!";

const COLORS: [u32; 19] = [
    0x00f44336, 0x00e91e63, 0x009c27b0, 0x00673ab7, 0x003f51b5, 0x002196f3, 0x0003a9f4, 0x0000bcd4,
    0x00009688, 0x004caf50, 0x008bc34a, 0x00cddc39, 0x00ffeb3b, 0x00ffc107, 0x00ff9800, 0x00ff5722,
    0x00795548, 0x009e9e9e, 0x00607d8b,
];

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    use display::GOPDisplay;

    static mut display: Option<GOPDisplay> = None;

    unsafe {
        display = Some(GOPDisplay::new(&boot_info.graphic_info));
    }

    unsafe {
        let mut console = console::Console::new(display.as_mut().unwrap());
        console.write("Hello world!\nLorem ipsum\rdolor");
    }

    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
