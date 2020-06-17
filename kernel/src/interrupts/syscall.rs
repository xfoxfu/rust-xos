#[repr(u8)]
pub enum Syscall {
    PrintStr = 0x05,
    ReadStr = 0x06,
    PlotPixel = 0x07,
}

pub fn print_str(str: &str) {}
pub fn read_str(str: &str) {}
pub fn plot_pixel(x: usize, y: usize, color: u32) {}
