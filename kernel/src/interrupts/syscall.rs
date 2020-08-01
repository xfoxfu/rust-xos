use super::handlers::Registers;
use core::alloc::Layout;
use spin::Mutex;
use x86_64::{structures::idt::InterruptStackFrame, VirtAddr};

static RETURN_POINT: Mutex<Option<(VirtAddr, VirtAddr, u64, Registers)>> = Mutex::new(None);

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

pub extern "C" fn syscall_handler(
    a0: u64,
    a1: u64,
    a2: u64,
    a3: u64,
    sf: &mut InterruptStackFrame,
    regs: &mut Registers,
) {
    use Syscall::*;

    debug!("syscall = {:x} {:x} {:x} {:x}", a0, a1, a2, a3);
    match a0 {
        v if v == SpawnProcess as u64 => spawn_process(a1, a2, sf, regs),
        v if v == ExitProcess as u64 => exit_process(sf, regs),
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
        v if v == Allocate as u64 => {
            allocate(unsafe { (a1 as *const Layout).as_ref().unwrap() }, unsafe {
                (a2 as *mut *mut u8).as_mut().unwrap()
            })
        }
        v if v == Deallocate as u64 => deallocate(unsafe { a1 as *mut u8 }, unsafe {
            (a2 as *const Layout).as_ref().unwrap()
        }),
        _ => (),
    }
}

pub fn spawn_process(target: u64, stack: u64, s: &mut InterruptStackFrame, regs: &mut Registers) {
    crate::process::save_current_process(s, regs);
    crate::process::get_process_list_sure()
        .first_mut()
        .unwrap()
        .pause();
    crate::process::switch_first_ready_process(s, regs);
}

pub fn exit_process(s: &mut InterruptStackFrame, regs: &mut Registers) {
    crate::process::kill_current_process();
    crate::process::switch_first_ready_process(s, regs);
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

pub fn allocate(layout: &core::alloc::Layout, ptr: &mut *mut u8) {
    *ptr = crate::allocator::ALLOCATOR
        .lock()
        .allocate_first_fit(layout.clone())
        .unwrap()
        .as_ptr();
    info!("allocated {:x} => {:x}", ptr as *mut _ as u64, *ptr as u64);
}

pub fn deallocate(ptr: *mut u8, layout: &core::alloc::Layout) {
    unsafe {
        crate::allocator::ALLOCATOR
            .lock()
            .deallocate(core::ptr::NonNull::new_unchecked(ptr), layout.clone());
    }
    info!("deallocated {:x}", ptr as u64);
}

pub fn read_disk(id: u64, dst: *mut u8) {}
