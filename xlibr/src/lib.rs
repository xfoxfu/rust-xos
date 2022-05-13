#![no_std]
#![feature(core_intrinsics)]
#![feature(alloc_error_handler)]
#![feature(asm_const)]

extern crate alloc;
extern crate rlibc;

#[macro_use]
mod console;

mod allocator;
mod display;
mod ide_device;
mod input;
mod syscall;

pub use console::_print;
pub use display::*;
pub use input::*;
pub use pc_keyboard::{DecodedKey, KeyCode};
pub use syscall::*;

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
