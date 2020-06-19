use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics::{drawable::Pixel, prelude::Size, DrawTarget};

pub struct SysDisplay;

impl<'a> DrawTarget<Rgb888> for SysDisplay {
    type Error = ();

    fn draw_pixel(&mut self, pixel: Pixel<Rgb888>) -> Result<(), Self::Error> {
        let Pixel(coord, color) = pixel;

        crate::sys_plot_pixel(
            if coord.x < 0 { 0 } else { coord.x as usize },
            if coord.y < 0 { 0 } else { coord.y as usize },
            (color.r() as u32) << 16 | (color.g() as u32) << 8 | (color.b() as u32),
        );

        Ok(())
    }
    fn size(&self) -> Size {
        let (x, y) = crate::sys_display_resolution();
        Size::new(x as u32, y as u32)
    }
}
