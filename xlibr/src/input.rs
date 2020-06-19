use crate::syscall::*;
use pc_keyboard::DecodedKey;

pub fn read_char() -> char {
    loop {
        let input = sys_read_key();
        match input {
            Some(DecodedKey::Unicode(u)) => {
                print!("{}", u);
                return u;
            }
            _ => (),
        }
    }
}

pub fn read_u64() -> u64 {
    let mut val = 0;
    loop {
        let c = read_char();
        match &c {
            '0'..='9' => {
                val *= 10;
                val += (c as u8 - '0' as u8) as u64;
            }
            _ => return val,
        }
    }
}

pub fn read_str(buf: &mut [u8]) -> &[u8] {
    let mut idx = 0;
    loop {
        let c = read_char();
        match c {
            '\n' | ' ' => break,
            c => {
                buf[idx] = c as u8;
                idx += 1;
            }
        }
    }
    &buf[0..idx]
}
