use crate::display::DISPLAY;
use core::fmt::Arguments;
use core::fmt::Write;
use embedded_graphics::{
    fonts::{Font8x16, Text},
    pixelcolor::Rgb888,
    prelude::*,
    style::TextStyle,
};
use spin::Mutex;

pub static CONSOLE: Mutex<Option<Console>> = Mutex::new(None);

const FONT_X: u8 = 8;
const FONT_Y: u8 = 16;

pub fn initialize() {
    *CONSOLE.lock() = Some(Console::new());
}

pub struct Console {
    x_pos: usize,
    y_pos: usize,
    // FIXME: use reference to display
    // buffer: &'static mut GOPDisplay<'static>,
}

impl Console {
    pub fn new() -> Self {
        Self { x_pos: 0, y_pos: 0 }
    }
}

impl Console {
    pub fn size(&self) -> (usize, usize) {
        let (disp_x, disp_y) = DISPLAY.lock().as_ref().unwrap().resolution();
        (disp_x / FONT_X as usize, disp_y / FONT_Y as usize)
    }

    pub fn next_row(&mut self) {
        self.y_pos += 1;
        if self.y_pos >= self.size().1 {
            self.scroll();
            self.y_pos = self.size().1 - 1;
        }
        self.x_pos = 0;
    }

    pub fn next_char(&mut self) {
        self.x_pos += 1;
        if self.x_pos >= self.size().0 {
            self.next_row()
        }
    }

    pub fn scroll(&mut self) {
        DISPLAY.lock().as_mut().unwrap().scrollup_n(FONT_Y);
    }

    pub fn write(&mut self, s: &str) {
        for c in s.chars() {
            match c {
                '\n' => {
                    self.next_row();
                }
                // '\r' => self.x_pos = 0,
                _ => {
                    let mut buf = [0u8; 2];
                    let str_c = c.encode_utf8(&mut buf);
                    Text::new(
                        str_c,
                        Point::new(
                            self.x_pos as i32 * FONT_X as i32,
                            self.y_pos as i32 * FONT_Y as i32,
                        ),
                    )
                    .into_styled(TextStyle::new(Font8x16, Rgb888::WHITE))
                    .draw(DISPLAY.lock().as_mut().unwrap())
                    .unwrap();
                    self.next_char()
                }
            }
        }
    }
}

impl Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: Arguments) {
    CONSOLE.lock().as_mut().unwrap().write_fmt(args).unwrap();
}
