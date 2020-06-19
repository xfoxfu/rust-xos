#![no_std]
#![feature(asm)]
#![feature(core_intrinsics)]

extern crate rlibc;

mod display;
mod syscall;

#[macro_use]
mod console;

pub use console::_print;
pub use display::*;
pub use pc_keyboard::{DecodedKey, KeyCode};
pub use syscall::*;

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
