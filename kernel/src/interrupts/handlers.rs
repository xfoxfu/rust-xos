use super::consts;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

pub fn reg_idt(idt: &mut InterruptDescriptorTable) {
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX);
    }
    idt.invalid_tss.set_handler_fn(invalid_tss_handler);
    idt.segment_not_present
        .set_handler_fn(segment_not_present_handler);
    idt.stack_segment_fault
        .set_handler_fn(stack_segment_fault_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);
    idt[(consts::Interrupts::IRQ0 as u8 + consts::IRQ::Timer as u8) as usize]
        .set_handler_fn(unsafe { core::mem::transmute(clock_handler_wrapper as *mut fn()) });
    idt[consts::Interrupts::Syscall as usize].set_handler_fn(unsafe {
        core::mem::transmute(syscall_handler_naked_wrapper as *mut fn())
    });
}

pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn invalid_tss_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: INVALID TSS {}\n{:#?}", error_code, stack_frame);
}

pub extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: u64,
) {
    error!(
        "EXCEPTION: SEGMENT NOT PRESENT {:#x}\n{:#?}",
        error_code, stack_frame
    );
}

pub extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: u64,
) {
    println!(
        "EXCEPTION: STACK SEGMENT FAULT {}\n{:#?}",
        error_code, stack_frame
    );
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

#[repr(align(8), C)]
#[derive(Debug, Clone, Default)]
pub struct Registers {
    r15: usize,
    r14: usize,
    r13: usize,
    r12: usize,
    r11: usize,
    r10: usize,
    r9: usize,
    r8: usize,
    rdi: usize,
    rsi: usize,
    rdx: usize,
    rcx: usize,
    rbx: usize,
    rax: usize,
    rbp: usize,
}

macro_rules! wrap {
    ($fn: ident => $w:ident) => {
        #[naked]
        pub unsafe extern "C" fn $w() {
            unsafe {
                asm!(
                    "
                push rbp
                push rax
                push rbx
                push rcx
                push rdx
                push rsi
                push rdi
                push r8
                push r9
                push r10
                push r11
                push r12
                push r13
                push r14
                push r15
                mov rsi, rsp  // 第二个参数：寄存器列表
                mov rdi, rsp
                add rdi, 15*8 // 第一个参数：中断帧
                call {}
                pop r15 
                pop r14 
                pop r13 
                pop r12 
                pop r11 
                pop r10 
                pop r9  
                pop r8  
                pop rdi 
                pop rsi 
                pop rdx 
                pop rcx 
                pop rbx 
                pop rax 
                pop rbp 
                iretq
                ",
                sym $fn
                );
                core::intrinsics::unreachable()
            }
        }
    };
}

wrap!(syscall_handler_naked => syscall_handler_naked_wrapper);

pub extern "C" fn syscall_handler_naked(sf: &mut InterruptStackFrame, regs: &mut Registers) {
    super::syscall::syscall_handler(
        regs.rax as u64,
        regs.rbx as u64,
        regs.rcx as u64,
        regs.rdx as u64,
        sf,
        regs,
    );
}

wrap!(clock_handler => clock_handler_wrapper);

pub extern "C" fn clock_handler(sf: &mut InterruptStackFrame, regs: &mut Registers) {
    crate::process::switch_first_ready_process(sf, regs);
    clock_draw();
    super::ack(consts::Interrupts::IRQ0 as u8);
}

fn clock_draw() {
    static ANGLE: spin::Mutex<u16> = spin::Mutex::new(90);
    const ANGLE_INCR: u16 = 15;

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
