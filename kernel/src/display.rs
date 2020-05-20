use boot::GraphicInfo;
use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics::{drawable::Pixel, geometry::Size, DrawTarget};

pub struct GOPDisplay<'a>(&'a GraphicInfo);

impl<'a> GOPDisplay<'a> {
    pub fn new(graphic: &'a GraphicInfo) -> Self {
        GOPDisplay(graphic)
    }
}

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
