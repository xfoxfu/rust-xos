use boot::GraphicInfo;
use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics::{drawable::Pixel, geometry::Size, DrawTarget};
use spin::Mutex;

pub static DISPLAY: Mutex<Option<GOPDisplay>> = Mutex::new(None);

pub fn initialize(graphic: &'static GraphicInfo) {
    *DISPLAY.lock() = Some(GOPDisplay::new(graphic));
}

pub struct GOPDisplay<'a>(&'a GraphicInfo);

impl<'a> GOPDisplay<'a> {
    pub fn new(graphic: &'a GraphicInfo) -> Self {
        GOPDisplay(graphic)
    }
}

impl<'a> GOPDisplay<'a> {
    pub fn clear(&mut self) {
        for i in 0..self.0.mode.resolution().1 {
            for j in 0..self.0.mode.stride() {
                unsafe {
                    *(self.0.fb_addr as *mut u32)
                        .add(i * self.0.mode.stride() + j)
                        .as_mut()
                        .unwrap() = 0u32;
                }
            }
        }
    }

    pub fn resolution(&self) -> (usize, usize) {
        self.0.mode.resolution()
    }

    pub fn scrollup_n(&self, n: u8) {
        let stride_u8 = self.0.mode.stride() * core::mem::size_of::<u32>();
        let (_xm, ym) = self.resolution();
        let n = n as usize;
        for y in 0..(ym - n) {
            unsafe {
                rlibc::memmove(
                    (self.0.fb_addr as *mut u8).add(y * stride_u8),
                    (self.0.fb_addr as *mut u8).add((y + n) * stride_u8),
                    stride_u8,
                );
            }
        }
        unsafe {
            rlibc::memset(
                (self.0.fb_addr as *mut u8).add((ym - n) * stride_u8),
                0,
                n * stride_u8,
            );
        }
    }
}

impl<'a> DrawTarget<Rgb888> for GOPDisplay<'a> {
    type Error = &'static str;

    fn draw_pixel(&mut self, pixel: Pixel<Rgb888>) -> Result<(), Self::Error> {
        let Pixel(coord, color) = pixel;

        unsafe {
            *(self.0.fb_addr as *mut u32)
                .add((coord.y as usize) * self.0.mode.stride() + (coord.x as usize))
                .as_mut()
                .unwrap() = (color.r() as u32) << 16 | (color.g() as u32) << 8 | (color.b() as u32);
        }

        Ok(())
    }
    fn size(&self) -> Size {
        let (x, y) = self.0.mode.resolution();
        Size::new(x as u32, y as u32)
    }
}
