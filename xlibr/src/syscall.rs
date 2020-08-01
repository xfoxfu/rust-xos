#[repr(u64)]
pub enum Syscall {
    SpawnProcess = 1,
    ExitProcess = 2,
    PrintStr = 5,
    ReadKey = 6,
    PlotPixel = 7,
    Sleep = 8,
    DisplayResolution = 9,
    Allocate = 10,
    Deallocate = 11,
    ReadDisk = 12,
}

pub fn syscall(id: u64, arg0: u64, arg1: u64, arg2: u64) {
    unsafe {
        asm!("int {id}", id = const 0x80, in("rax") id, in("rbx") arg0, in("rcx") arg1, in("rdx") arg2);
    }
}

pub fn sys_exit() -> ! {
    syscall(Syscall::ExitProcess as u64, 0, 0, 0);
    unsafe { core::intrinsics::unreachable() }
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

pub fn sys_display_resolution() -> (u64, u64) {
    let (mut x, mut y) = (0, 0);
    syscall(
        Syscall::DisplayResolution as u64,
        (&mut x) as *mut _ as u64,
        (&mut y) as *mut _ as u64,
        0,
    );
    (x, y)
}

pub fn sys_allocate(layout: &core::alloc::Layout) -> *mut u8 {
    let mut ptr = 0 as *mut u8;
    syscall(
        Syscall::Allocate as u64,
        layout as *const _ as u64,
        &mut ptr as *mut *mut u8 as u64,
        0,
    );
    ptr
}

pub fn sys_deallocate(ptr: *mut u8, layout: &core::alloc::Layout) {
    syscall(
        Syscall::Deallocate as u64,
        ptr as u64,
        layout as *const _ as u64,
        0,
    );
}

pub fn sys_read_disk(id: u64, dst: *mut u8) {}
