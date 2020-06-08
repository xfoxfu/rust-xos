use super::consts;
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;
use x86_64::{
    instructions::port::Port,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame},
};

/// 注册中断向量，本函数应当在中断向量表初始化代码中调用
pub fn reg_idt(idt: &mut InterruptDescriptorTable) {
    idt[(consts::Interrupts::IRQ0 as u8 + consts::IRQ::Keyboard as u8) as usize]
        .set_handler_fn(interrupt_handler);
}

/// 初始化键盘驱动
///
/// 需要内存初始化
pub unsafe fn init() {
    use super::enable_irq;
    enable_irq(consts::IRQ::Keyboard as u8);
    debug!("keyboard IRQ enabled");
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
    super::ack(super::consts::IRQ::Keyboard as u8);
    if let Some(key) = receive() {
        trace!("key readed {:?}", key);
        if let Some(mut buf) = crate::drivers::keyboard::buffer() {
            buf.push_back(key);
        } else {
            trace!(
                "keyboard input ignored because of uninitialized keyboard driver {:?}",
                key
            );
        }
    }
}
