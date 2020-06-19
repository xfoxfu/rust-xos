#![no_std]
#![no_main]
#![feature(asm)]

extern crate rlibc;
#[macro_use]
extern crate xlibr;

#[export_name = "_start"]
pub extern "C" fn __impl_start() -> ! {
    println!("input a char, string, space and a integer");
    let ch = xlibr::read_char();
    let mut s = [0; 80];
    let ss = xlibr::read_str(&mut s);
    let a = xlibr::read_u64();
    println!(
        "ch={}, a={}, s={}",
        ch,
        a,
        core::str::from_utf8(&ss).unwrap()
    );
    xlibr::sys_exit()
}
