use alloc::collections::VecDeque;
use pc_keyboard::DecodedKey;
use spin::Mutex;

once_mutex!(KEY_BUFFER: VecDeque<DecodedKey>);

/// 初始化键盘输入设备
///
/// 需要确保内存已经初始化；在键盘中断初始化完成前，无法获得输入
pub unsafe fn init() {
    init_KEY_BUFFER(VecDeque::with_capacity(80));
    debug!("keybaord buffer initialized");
}

guard_access_fn!(pub buffer(KEY_BUFFER: VecDeque<DecodedKey>));

/// 读取按键，非阻塞
pub fn get_key() -> Option<DecodedKey> {
    x86_64::instructions::interrupts::without_interrupts(|| buffer()?.front().copied())
}

/// 读取按键，阻塞直到存在按键
pub fn get_key_block() -> DecodedKey {
    loop {
        if let Some(k) = get_key() {
            return k;
        }
    }
}
