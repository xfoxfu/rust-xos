use spin::Mutex;
use x86_64::{structures::idt::InterruptStackFrame, VirtAddr};

static RETURN_IP: Mutex<Option<VirtAddr>> = Mutex::new(None);

#[repr(u64)]
pub enum Syscall {
    SpawnProcess = 1,
    ExitProcess = 2,
    PrintStr = 5,
    ReadKey = 6,
    PlotPixel = 7,
    Sleep = 8,
    DisplayResolution = 9,
}

pub extern "C" fn syscall_handler(
    a0: u64,
    a1: u64,
    a2: u64,
    a3: u64,
    sf: &mut InterruptStackFrame,
) {
    use Syscall::*;

    debug!("syscall = {:x} {:x} {:x} {:x}", a0, a1, a2, a3);
    match a0 {
        v if v == SpawnProcess as u64 => spawn_process(a1, sf),
        v if v == ExitProcess as u64 => exit_process(sf),
        v if v == PrintStr as u64 => print_str(unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                a1 as *const u8,
                a2 as usize,
            ))
        }),
        v if v == ReadKey as u64 => read_str(unsafe {
            (a1 as *mut Option<pc_keyboard::DecodedKey>)
                .as_mut()
                .unwrap()
        }),
        v if v == PlotPixel as u64 => {
            plot_pixel(a1 as usize, a2 as usize, (a3 & 0xFFFFFFFF) as u32)
        }
        v if v == Sleep as u64 => sleep(a1),
        v if v == DisplayResolution as u64 => {
            display_resolution(unsafe { (a1 as *mut u64).as_mut().unwrap() }, unsafe {
                (a2 as *mut u64).as_mut().unwrap()
            })
        }
        _ => (),
    }
}

pub fn spawn_process(target: u64, s: &mut InterruptStackFrame) {
    *RETURN_IP.lock() = Some(s.instruction_pointer);
    unsafe {
        s.as_mut().instruction_pointer = VirtAddr::new(target);
    }
}

pub fn exit_process(s: &mut InterruptStackFrame) {
    if let Some(target) = *RETURN_IP.lock() {
        unsafe {
            s.as_mut().instruction_pointer = target;
        }
    } else {
        panic!("process exited without return point");
    }
}

pub fn print_str(s: &str) {
    print!("{}", s)
}

pub fn read_str(s: &mut Option<pc_keyboard::DecodedKey>) {
    *s = crate::drivers::get_key();
}

pub fn plot_pixel(x: usize, y: usize, color: u32) {
    let _ = crate::display::get_display_sure().set_pixel(x, y, color);
}

pub fn sleep(ns: u64) {
    crate::uefi_clock::get_clock_sure().spin_wait_for_ns(ns as usize);
}

pub fn display_resolution(px: &mut u64, py: &mut u64) {
    let (x, y) = crate::display::get_display_sure().resolution();
    *px = x as u64;
    *py = y as u64;
}
