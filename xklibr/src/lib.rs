#![no_std]

mod gop_display;
mod uefi_clock;

pub use gop_display::GOPDisplay;
pub use uefi_clock::UefiClock;
