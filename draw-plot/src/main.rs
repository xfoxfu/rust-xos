#![no_std]
#![no_main]
#![feature(asm)]

extern crate rlibc;
#[macro_use]
extern crate xlibr;

const COLORS: [u32; 19] = [
    0x00f44336, 0x00e91e63, 0x009c27b0, 0x00673ab7, 0x003f51b5, 0x002196f3, 0x0003a9f4, 0x0000bcd4,
    0x00009688, 0x004caf50, 0x008bc34a, 0x00cddc39, 0x00ffeb3b, 0x00ffc107, 0x00ff9800, 0x00ff5722,
    0x00795548, 0x009e9e9e, 0x00607d8b,
];

#[export_name = "_start"]
pub extern "C" fn __impl_start(boot_info: &'static boot::BootInfo) {
    println!("Hello world");
    let (base_x, base_y, max_x, max_y) = (0, 0, 800, 600);
    let mut row = base_y as isize;
    let mut col = base_x as isize;

    let mut row_incr = 2isize;
    let mut col_incr = 1isize;
    let mut color = 0;
    loop {
        xlibr::sys_plot_pixel(col as usize, row as usize, COLORS[color]);
        row += row_incr;
        col += col_incr;

        if col <= base_x || col > max_x {
            col_incr = -col_incr;
        }
        if row <= base_y || row > max_y {
            row_incr = -row_incr;
        }

        if col <= base_x || col > max_x || row <= base_y || row > max_y {
            color += 1;
        }

        if color >= 19 {
            color = 0;
        }

        // wait for a little while
        for _ in 0..5_000_00 {
            unsafe { asm!("nop") }
        }

        if let Some(xlibr::DecodedKey::Unicode('\x1b')) = xlibr::sys_read_key() {
            break;
        }
    }
}
