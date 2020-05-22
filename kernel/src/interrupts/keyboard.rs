use super::consts;
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;
use x86_64::{
    instructions::port::Port,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame},
};

pub fn reg_idt(idt: &mut InterruptDescriptorTable) {
    idt[(consts::Interrupts::IRQ0 as u8 + consts::IRQ::Keyboard as u8) as usize]
        .set_handler_fn(interrupt_handler);
}

pub fn init() {
    use super::enable_irq;
    enable_irq(consts::IRQ::Keyboard as u8);
}

/// Receive character from keyboard
/// Should be called on every interrupt
pub fn receive() -> Option<DecodedKey> {
    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
            Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
        );
    }

    let mut keyboard = KEYBOARD.lock();
    let mut data_port = Port::<u8>::new(0x60);
    let mut status_port = Port::<u8>::new(0x64);

    // Output buffer status = 1
    if unsafe { status_port.read() } & 0x1 != 0 {
        let scancode = unsafe { data_port.read() };
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            return keyboard.process_keyevent(key_event);
        }
    }
    None
}

pub extern "x86-interrupt" fn interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    super::ack(1);
    if let Some(key) = receive() {
        match key {
            DecodedKey::Unicode(character) => print!("{}", character),
            DecodedKey::RawKey(key) => print!("{:?}", key),
        }
    }
}
