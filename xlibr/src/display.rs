use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics::prelude::{Dimensions, Point};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::{draw_target::DrawTarget, prelude::Size, Pixel};

pub struct SysDisplay;

impl Dimensions for SysDisplay {
    fn bounding_box(&self) -> embedded_graphics::primitives::Rectangle {
        let (x, y) = crate::sys_display_resolution();
        Rectangle::new(Point::new(0, 0), Size::new(x as u32, y as u32))
    }
}

impl<'a> DrawTarget for SysDisplay {
    type Color = Rgb888;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            let Pixel(coord, color) = pixel;

            crate::sys_plot_pixel(
                if coord.x < 0 { 0 } else { coord.x as usize },
                if coord.y < 0 { 0 } else { coord.y as usize },
                (color.r() as u32) << 16 | (color.g() as u32) << 8 | (color.b() as u32),
            );
        }

        Ok(())
    }
}
