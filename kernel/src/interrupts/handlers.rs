use super::consts;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub fn reg_idt(idt: &mut InterruptDescriptorTable) {
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.double_fault.set_handler_fn(double_fault_handler);
    idt[(consts::Interrupts::IRQ0 as u8 + consts::IRQ::Timer as u8) as usize]
        .set_handler_fn(clock_handler);
}

pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn clock_handler(_stack_frame: &mut InterruptStackFrame) {
    static mut CHR: char = '|';
    super::ack(consts::Interrupts::IRQ0 as u8);
    x86_64::instructions::interrupts::without_interrupts(|| unsafe {
        CHR = match CHR {
            '|' => '/',
            '/' => '-',
            '-' => '\\',
            '\\' => '|',
            _ => unreachable!(),
        };
        crate::console::CONSOLE
            .lock()
            .as_mut()
            .unwrap()
            .write_char_at(0, 0, CHR);
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
