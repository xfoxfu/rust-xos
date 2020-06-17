#[repr(u64)]
pub enum Syscall {
    PrintStr = 5,
    ReadKey = 6,
    PlotPixel = 7,
    Sleep = 8,
}

pub fn syscall(id: u64, arg0: u64, arg1: u64, arg2: u64) {
    unsafe {
        asm!("int {id}", id = const 0x80, in("rax") id, in("rbx") arg0, in("rcx") arg1, in("rdx") arg2);
    }
}

pub fn sys_print_str(s: &str) {
    syscall(
        Syscall::PrintStr as u64,
        s.as_ptr() as u64,
        s.len() as u64,
        0,
    );
}
pub fn sys_read_key() -> Option<pc_keyboard::DecodedKey> {
    let mut s: Option<pc_keyboard::DecodedKey> = None;
    syscall(Syscall::ReadKey as u64, (&mut s) as *mut _ as u64, 0, 0);
    s
}
pub fn sys_plot_pixel(x: usize, y: usize, color: u32) {
    syscall(Syscall::PlotPixel as u64, x as u64, y as u64, color as u64);
}
pub fn sys_sleep(ns: u64) {
    syscall(Syscall::Sleep as u64, ns, 0, 0);
}
