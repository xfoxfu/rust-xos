use crate::display::GOPDisplay;
use embedded_graphics::{
    fonts::{Font8x16, Text},
    pixelcolor::Rgb888,
    prelude::*,
    style::TextStyle,
};

pub struct Console {
    x_pos: usize,
    y_pos: usize,
    buffer: &'static mut GOPDisplay<'static>,
}

impl Console {
    pub fn new(buffer: &'static mut GOPDisplay<'static>) -> Self {
        Self {
            x_pos: 0,
            y_pos: 0,
            buffer,
        }
    }
}

impl Console {
    pub fn write(&mut self, s: &str) {
        for c in s.chars() {
            match c {
                '\n' => self.y_pos += 1,
                '\r' => self.x_pos = 0,
                _ => {
                    let mut buf = [0u8; 2];
                    let str_c = c.encode_utf8(&mut buf);
                    Text::new(
                        str_c,
                        Point::new(self.x_pos as i32 * 8, self.y_pos as i32 * 16),
                    )
                    .into_styled(TextStyle::new(Font8x16, Rgb888::WHITE))
                    .draw(self.buffer)
                    .unwrap();
                    self.x_pos += 1;
                }
            }
        }
    }
}
