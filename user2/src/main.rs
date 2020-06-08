#![no_std]
#![no_main]
#![feature(llvm_asm)]

extern crate rlibc;

#[export_name = "_start"]
pub extern "C" fn __impl_start(boot_info: &'static boot::BootInfo) {
    let (width, height) = boot_info.graphic_info.mode.resolution();
    let (hw, hh) = (width as isize / 2, height as isize / 2);
    xlibr::display(boot_info, 0, hh, hw, hh * 2, 2_500)
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    loop {}
}
