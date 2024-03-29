use alloc::string::String;
use crossbeam_queue::ArrayQueue;
use pc_keyboard::DecodedKey;
use spin::Once;

pub static KEY_BUFFER: Once<ArrayQueue<DecodedKey>> = Once::new();

const DEFAULT_CAPACITY: usize = 80;

pub fn buffer<'a>() -> &'a ArrayQueue<DecodedKey> {
    &KEY_BUFFER.get().unwrap()
}

/// 初始化键盘输入设备
///
/// 需要确保内存已经初始化；在键盘中断初始化完成前，无法获得输入
pub unsafe fn init() {
    KEY_BUFFER.call_once(|| ArrayQueue::new(DEFAULT_CAPACITY));
    debug!("keybaord buffer initialized");
}

/// 读取按键，非阻塞
pub fn get_key() -> Option<DecodedKey> {
    x86_64::instructions::interrupts::without_interrupts(|| buffer().pop())
}

/// 读取按键，阻塞直到存在按键
pub fn get_key_block() -> DecodedKey {
    loop {
        if let Some(k) = get_key() {
            return k;
        }
    }
}

/// 读取一行输入
pub fn getline_block() -> String {
    use crate::console::get_console_sure;

    let mut s = String::with_capacity(DEFAULT_CAPACITY);
    while let DecodedKey::Unicode(k) = get_key_block() {
        match k {
            '\n' => break,
            // backspace
            '\x08' => {
                if !s.is_empty() {
                    crate::console::get_console_sure().move_cursor(-1, 0);
                    print!("  "); // draw a char more to clear hint
                    crate::console::get_console_sure().move_cursor(-2, 0);
                    s.pop(); // remove previous char
                }
            }
            c => {
                print!("{}", k);
                s.push(c)
            }
        }
        get_console_sure().draw_hint();
    }
    println!();
    s
}
