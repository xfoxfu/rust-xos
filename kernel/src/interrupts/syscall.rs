#[repr(u64)]
pub enum Syscall {
    PrintStr = 5,
    ReadKey = 6,
    PlotPixel = 7,
    Sleep = 8,
}

pub extern "C" fn syscall_handler(a0: u64, a1: u64, a2: u64, a3: u64) {
    debug!("syscall = {:x} {:x} {:x} {:x}", a0, a1, a2, a3);
    match a0 {
        v if v == Syscall::PrintStr as u64 => print_str(unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                a1 as *const u8,
                a2 as usize,
            ))
        }),
        v if v == Syscall::ReadKey as u64 => read_str(unsafe {
            (a1 as *mut Option<pc_keyboard::DecodedKey>)
                .as_mut()
                .unwrap()
        }),
        v if v == Syscall::PlotPixel as u64 => {
            plot_pixel(a1 as usize, a2 as usize, (a3 & 0xFFFFFFFF) as u32)
        }
        v if v == Syscall::Sleep as u64 => sleep(a1),
        _ => (),
    }
}

pub fn print_str(s: &str) {
    print!("{}", s)
}

pub fn read_str(s: &mut Option<pc_keyboard::DecodedKey>) {
    *s = crate::drivers::get_key();
}

pub fn plot_pixel(x: usize, y: usize, color: u32) {
    crate::display::get_display_sure().set_pixel(x, y, color);
}

pub fn sleep(ns: u64) {
    crate::uefi_clock::get_clock_sure().spin_wait_for_ns(ns as usize);
}
