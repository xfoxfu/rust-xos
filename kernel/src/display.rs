#![allow(dead_code)]

use boot::GraphicInfo;
use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics::{drawable::Pixel, geometry::Size, DrawTarget};
use spin::Mutex;

#[derive(Debug)]
pub enum DisplayError {
    OutOfBound(usize, usize),
}

once_mutex!(pub DISPLAY: GOPDisplay<'static>);

pub fn initialize(graphic: &'static GraphicInfo) {
    init_DISPLAY(GOPDisplay::new(graphic));
}

guard_access_fn! {
    #[doc = "基于GOP的显示器"]
    pub get_display(DISPLAY: GOPDisplay<'static>)
}

pub struct GOPDisplay<'a>(&'a GraphicInfo, &'a mut [u32]);

impl<'a> GOPDisplay<'a> {
    pub fn new(graphic: &'a GraphicInfo) -> Self {
        GOPDisplay(graphic, unsafe {
            core::slice::from_raw_parts_mut(
                graphic.fb_addr as *mut u32,
                graphic.mode.resolution().0 * graphic.mode.stride(),
            )
        })
    }
}

impl<'a> GOPDisplay<'a> {
    pub fn get_pixel(&self, x: usize, y: usize) -> Result<u32, DisplayError> {
        Ok(*self
            .1
            .get(y * self.0.mode.stride() + x)
            .ok_or_else(|| DisplayError::OutOfBound(x, y))?)
    }
    pub fn set_pixel(&mut self, x: usize, y: usize, raw_color: u32) -> Result<(), DisplayError> {
        *self
            .1
            .get_mut(y * self.0.mode.stride() + x)
            .ok_or_else(|| DisplayError::OutOfBound(x, y))? = raw_color;

        Ok(())
    }

    pub fn clear(&mut self) {
        // the following is safe because offset are computed correctly
        unsafe {
            rlibc::memset(
                self.0.fb_addr as *mut u8,
                0,
                self.0.mode.resolution().1 * self.0.mode.stride() * 4,
            );
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
            // the following is safe because offset are computed correctly
            unsafe {
                rlibc::memmove(
                    (self.0.fb_addr as *mut u8).add(y * stride_u8),
                    (self.0.fb_addr as *mut u8).add((y + n) * stride_u8),
                    stride_u8,
                );
            }
        }
        unsafe {
            // the following is safe because offset are computed correctly
            rlibc::memset(
                (self.0.fb_addr as *mut u8).add((ym - n) * stride_u8),
                0,
                n * stride_u8,
            );
        }
    }
}

impl<'a> DrawTarget<Rgb888> for GOPDisplay<'a> {
    type Error = DisplayError;

    fn draw_pixel(&mut self, pixel: Pixel<Rgb888>) -> Result<(), Self::Error> {
        let Pixel(coord, color) = pixel;

        self.set_pixel(
            coord.x as usize,
            coord.y as usize,
            // FIXME: currently only support BGR LE
            (color.r() as u32) << 16 | (color.g() as u32) << 8 | (color.b() as u32),
        )?;

        Ok(())
    }
    fn size(&self) -> Size {
        let (x, y) = self.0.mode.resolution();
        Size::new(x as u32, y as u32)
    }
}
