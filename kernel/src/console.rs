use crate::display::get_display_sure;
use core::fmt::Arguments;
use core::fmt::Write;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Line, PrimitiveStyle};
use embedded_graphics::text::Alignment;
use embedded_graphics::text::Text;
use embedded_graphics::text::TextStyleBuilder;
use profont::PROFONT_12_POINT;

once_mutex!(pub CONSOLE: Console);

const FONT_X: u8 = 8;
const FONT_Y: u8 = 15;

pub fn initialize() {
    init_CONSOLE(Console::new());
}

guard_access_fn!(pub get_console (CONSOLE: Console));

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
        let (disp_x, disp_y) = get_display_sure().resolution();
        (disp_x / FONT_X as usize, disp_y / FONT_Y as usize)
    }

    fn get_char_pos(&self, x: usize, y: usize) -> (usize, usize) {
        (x * FONT_X as usize, y * FONT_Y as usize)
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
        get_display_sure().scrollup_n(FONT_Y);
    }

    pub fn write_char_at(&mut self, x: usize, y: usize, c: char) {
        let mut buf = [0u8; 2];
        let str_c = c.encode_utf8(&mut buf);
        Text::with_text_style(
            str_c,
            Point::new(x as i32 * FONT_X as i32, y as i32 * FONT_Y as i32),
            MonoTextStyleBuilder::new()
                .font(&profont::PROFONT_12_POINT)
                .text_color(Rgb888::WHITE)
                .background_color(Rgb888::BLACK)
                .build(),
            TextStyleBuilder::new()
                .alignment(Alignment::Left)
                .baseline(embedded_graphics::text::Baseline::Top)
                .build(),
        )
        .draw(&mut *get_display_sure())
        .unwrap();
    }

    pub fn write(&mut self, s: &str) {
        for c in s.chars() {
            match c {
                '\n' => {
                    self.next_row();
                }
                // '\r' => self.x_pos = 0,
                _ => {
                    self.write_char_at(self.x_pos, self.y_pos, c);
                    self.next_char()
                }
            }
        }
    }

    pub fn move_cursor(&mut self, dx: isize, dy: isize) {
        self.x_pos = (self.x_pos as isize + dx) as usize;
        self.y_pos = (self.y_pos as isize + dy) as usize;
    }

    pub fn draw_hint(&mut self) {
        let (x, y) = (self.x_pos, self.y_pos);
        let (cx, cy) = self.get_char_pos(x, y);
        Line::new(
            Point::new(cx as i32, cy as i32),
            Point::new(cx as i32, cy as i32 + FONT_Y as i32 - 1),
        )
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, 1))
        .draw(&mut *get_display_sure())
        .unwrap(); // FIXME: report error later
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
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        get_console_sure().write_fmt(args).unwrap();
    });
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("[PANIC] {}", info);
    loop {}
}
