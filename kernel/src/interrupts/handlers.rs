use super::consts;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

pub fn reg_idt(idt: &mut InterruptDescriptorTable) {
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.double_fault.set_handler_fn(double_fault_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);
    idt[(consts::Interrupts::IRQ0 as u8 + consts::IRQ::Timer as u8) as usize]
        .set_handler_fn(clock_handler);
    idt[consts::Interrupts::Syscall as usize].set_handler_fn(unsafe {
        core::mem::transmute(syscall_handler_wrap as *mut extern "C" fn())
    });
}

pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn clock_handler(_stack_frame: &mut InterruptStackFrame) {
    static ANGLE: spin::Mutex<u16> = spin::Mutex::new(90);
    const ANGLE_INCR: u16 = 15;
    super::ack(consts::Interrupts::IRQ0 as u8);
    x86_64::instructions::interrupts::without_interrupts(|| {
        use embedded_graphics::drawable::*;
        use embedded_graphics::pixelcolor::*;
        use embedded_graphics::prelude::*;
        use embedded_graphics::primitives::*;
        use embedded_graphics::style::*;

        let value;
        // 自增
        if let Some(mut angle_locked) = ANGLE.try_lock() {
            *angle_locked += ANGLE_INCR;
            if *angle_locked >= 360 {
                *angle_locked = 0;
            }
            value = *angle_locked as f32 / 180f32 * core::f32::consts::PI;
        } else {
            value = 0.0;
        }

        let (cx, cy) = (8 * 79, 16 * 24);
        let len = 16u32;

        #[allow(unused_imports)]
        use micromath::F32Ext;
        let (dx, dy) = (
            (len as f32 * value.cos()) as i32,
            (len as f32 * value.sin()) as i32,
        );

        if let Some(mut display) = crate::display::get_display() {
            Circle::new(Point::new(cx, cy), len)
                .into_styled(PrimitiveStyle::with_fill(Rgb888::WHITE))
                .draw(&mut *display)
                .unwrap(); // FIXME: report error later

            Line::new(Point::new(cx, cy), Point::new(cx - dx, cy - dy))
                .into_styled(PrimitiveStyle::with_stroke(Rgb888::BLACK, 3))
                .draw(&mut *display)
                .unwrap(); // FIXME: report error later
        }
    })
}

pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "EXCEPTION: DOUBLE FAULT\n{:#?}, {}",
        stack_frame, error_code
    );
}

pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    panic!(
        "EXCEPTION: PAGE FAULT\n{:#?}, {:?} addr={:?}",
        stack_frame,
        error_code,
        x86_64::registers::control::Cr2::read()
    );
}

pub extern "x86-interrupt" fn syscall_handler_wrap(sf: &mut InterruptStackFrame) {
    let a0: u64;
    let a1: u64;
    let a2: u64;
    let a3: u64;
    unsafe {
        asm!("", out("rax") a0, out("rbx") a1, out("rcx") a2, out("rdx") a3);
    }
    super::syscall::syscall_handler(a0, a1, a2, a3, sf);
}
