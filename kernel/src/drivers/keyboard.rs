use alloc::collections::VecDeque;
use alloc::string::String;
use pc_keyboard::DecodedKey;
use spin::Mutex;

once_mutex!(KEY_BUFFER: VecDeque<DecodedKey>);

const DEFAULT_CAPACITY: usize = 80;

/// 初始化键盘输入设备
///
/// 需要确保内存已经初始化；在键盘中断初始化完成前，无法获得输入
pub unsafe fn init() {
    init_KEY_BUFFER(VecDeque::with_capacity(DEFAULT_CAPACITY));
    debug!("keybaord buffer initialized");
}

guard_access_fn!(pub buffer(KEY_BUFFER: VecDeque<DecodedKey>));

/// 读取按键，非阻塞
pub fn get_key() -> Option<DecodedKey> {
    x86_64::instructions::interrupts::without_interrupts(|| buffer()?.pop_front())
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
    let mut s = String::with_capacity(DEFAULT_CAPACITY);
    while let DecodedKey::Unicode(k) = get_key_block() {
        match k {
            '\n' => break,
            // backspace
            '\x08' => {
                if s.len() > 0 {
                    crate::console::get_console_sure().move_cursor(-1, 0);
                    print!(" ");
                    crate::console::get_console_sure().move_cursor(-1, 0);
                    s.pop(); // remove previous char
                }
            }
            c => {
                print!("{}", k);
                s.push(c)
            }
        }
    }
    s
}
