const COLORS: [u32; 19] = [
    0x00f44336, 0x00e91e63, 0x009c27b0, 0x00673ab7, 0x003f51b5, 0x002196f3, 0x0003a9f4, 0x0000bcd4,
    0x00009688, 0x004caf50, 0x008bc34a, 0x00cddc39, 0x00ffeb3b, 0x00ffc107, 0x00ff9800, 0x00ff5722,
    0x00795548, 0x009e9e9e, 0x00607d8b,
];

use embedded_graphics::{
    pixelcolor::Rgb888, prelude::*, primitives::Circle, style::PrimitiveStyle,
};

pub fn display(
    boot_info: &'static boot::BootInfo,
    base_x: isize,
    base_y: isize,
    max_x: isize,
    max_y: isize,
    iters: usize,
) {
    let mut display = xklibr::GOPDisplay::new(&boot_info.graphic_info);

    let mut row = base_y;
    let mut col = base_x;

    let mut row_incr = 2;
    let mut col_incr = 1;
    let mut color = 0;
    for _ in 0..iters {
        // ignore error because no panic handler is present
        let _ = Circle::new(Point::new(col as i32, row as i32), 1)
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(
                (COLORS[color] >> 16 & 0xFF) as u8,
                (COLORS[color] >> 8 & 0xFF) as u8,
                (COLORS[color] >> 0 & 0xFF) as u8,
            )))
            .draw(&mut display);

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
            unsafe { llvm_asm!("nop") }
        }
    }
}
