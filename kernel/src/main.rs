#![no_std]
#![no_main]
#![feature(llvm_asm)]

use boot::{BootInfo, GraphicInfo};
use core::panic::PanicInfo;
use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics::{drawable::Pixel, geometry::Size, DrawTarget};

extern crate rlibc;

static HELLO: &[u8] = b"Hello World!";

const COLORS: [u32; 19] = [
    0x00f44336, 0x00e91e63, 0x009c27b0, 0x00673ab7, 0x003f51b5, 0x002196f3, 0x0003a9f4, 0x0000bcd4,
    0x00009688, 0x004caf50, 0x008bc34a, 0x00cddc39, 0x00ffeb3b, 0x00ffc107, 0x00ff9800, 0x00ff5722,
    0x00795548, 0x009e9e9e, 0x00607d8b,
];

struct GOPDisplay<'a>(&'a GraphicInfo);

impl<'a> DrawTarget<Rgb888> for GOPDisplay<'a> {
    type Error = &'static str;

    fn draw_pixel(&mut self, pixel: Pixel<Rgb888>) -> Result<(), Self::Error> {
        let Pixel(coord, color) = pixel;

        unsafe {
            *(self.0.fb_addr as *mut u32)
                .offset(((coord.y as usize) * self.0.mode.stride() + (coord.x as usize)) as isize)
                .as_mut()
                .unwrap() = (color.r() as u32) << 16 | (color.g() as u32) << 8 | (color.b() as u32);
        }

        Ok(())
    }
    fn size(&self) -> Size {
        let (x, y) = self.0.mode.resolution();
        return Size::new(x as u32, y as u32);
    }
}

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    let mode = boot_info.graphic_info.mode;
    let (display_x, display_y) = mode.resolution();
    let (display_x, display_y) = (display_x as isize, display_y as isize);
    let fb_addr = boot_info.graphic_info.fb_addr;
    let fb_size = boot_info.graphic_info.fb_size;

    // info!("fb_addr={} fb_size={}", fb_addr, fb_size);

    for i in 0..display_x * display_y {
        unsafe {
            *(fb_addr as *mut u32).offset(i).as_mut().unwrap() = 0x000F0F0F;
        }
    }

    use embedded_graphics::{egtext, fonts::Font8x16, pixelcolor::Rgb565, prelude::*, text_style};

    let mut display = GOPDisplay(&boot_info.graphic_info);

    egtext!(
        text = "Hello Rust!",
        top_left = (0, 0),
        style = text_style!(
            font = Font8x16,
            text_color = Rgb888::WHITE,
            background_color = Rgb888::BLACK,
        )
    )
    .draw(&mut display)
    .expect("Draw successfully");

    // info!("PRESS ESC TO SHUTDOWN");
    // bs.stall(10_000); // wait for 10ms

    let base_x: isize = 0;
    let base_y: isize = 0;
    let max_x: isize = display_x;
    let max_y: isize = display_y;

    let mut row = base_x;
    let mut col = base_y;

    let mut row_incr = 2;
    let mut col_incr = 1;
    let mut color = 0;
    loop {
        if col >= base_x && col < max_x && row >= base_y && row < max_y {
            unsafe {
                *(fb_addr as *mut u32)
                    .offset(row * display_x + col)
                    .as_mut()
                    .unwrap() = COLORS[color];
            }
        } else {
            // warn!("invalid position ({}, {})", col, row);
        }

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

        // bs.stall(10_000); // wait for 10ms
        for _ in 0..1_000 {
            unsafe {
                llvm_asm! {"nop"}
            }
        }
    }

    // shutdown
    // info!("Shut down ......");
    // bs.stall(1000_000); // wait for 100ms
    // let rs = st.runtime_services();
    // rs.reset(
    //     uefi::table::runtime::ResetType::Shutdown,
    //     Status::SUCCESS,
    //     None,
    // );

    // let vga_buffer = 0xb8000 as *mut u8;

    // for (i, &byte) in HELLO.iter().enumerate() {
    //     unsafe {
    //         *vga_buffer.offset(i as isize * 2) = byte;
    //         *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
    //     }
    // }

    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
