#![no_std]
#![no_main]
#![feature(asm)]

extern crate alloc;
extern crate rlibc;
#[macro_use]
extern crate xlibr;

#[export_name = "_start"]
pub extern "C" fn __impl_start() -> ! {
    println!("hello world");
    let mut vec = alloc::vec::Vec::with_capacity(80);
    vec.resize(160, 42);
    println!("{:?}", vec);
    xlibr::sys_exit()
}
