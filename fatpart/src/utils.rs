#[inline(always)]
pub fn read_u16_le(data: &[u8], offset: usize) -> u16 {
    ((data[offset + 1] as u16) << 8) | (data[offset] as u16)
}

#[inline(always)]
pub fn read_u32_le(data: &[u8], offset: usize) -> u32 {
    ((data[offset + 3] as u32) << 24)
        | ((data[offset + 2] as u32) << 16)
        | ((data[offset + 1] as u32) << 8)
        | (data[offset] as u32)
}

#[cfg(test)]
#[test]
fn test_read_un_le() {
    let data: &[u8] = &[0x53, 0x57, 0x49, 0x4E, 0x34];
    assert_eq!(read_u16_le(data, 0), 22355);
    assert_eq!(read_u16_le(data, 2), 20041);
    assert_eq!(read_u32_le(data, 0), 1313429331);
    assert_eq!(read_u32_le(data, 1), 877545815);
}
