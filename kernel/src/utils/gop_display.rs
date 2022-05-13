use boot::GraphicInfo;
use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics::prelude::Point;
use embedded_graphics::prelude::{Dimensions, OriginDimensions};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::{draw_target::DrawTarget, geometry::Size, Pixel};

#[derive(Debug)]
pub enum DisplayError {
    OutOfBound(usize, usize),
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

impl<'a> OriginDimensions for GOPDisplay<'a> {
    fn size(&self) -> Size {
        Size::new(self.resolution().0 as u32, self.resolution().1 as u32)
    }
}

impl<'a> DrawTarget for GOPDisplay<'a> {
    type Color = Rgb888;
    type Error = DisplayError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            let Pixel(coord, color) = pixel;

            self.set_pixel(
                coord.x as usize,
                coord.y as usize,
                // FIXME: currently only support BGR LE
                (color.r() as u32) << 16 | (color.g() as u32) << 8 | (color.b() as u32),
            )?;
        }
        Ok(())
    }
}
