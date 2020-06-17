use super::{FatDate, FatTime};
use crate::{read_u16_le, read_u32_le};
use bitflags::bitflags;

#[derive(Debug, Eq, PartialEq)]
pub struct DirEntry {
    /// 文件名
    pub stem_raw: [u8; 8],
    /// 扩展名
    pub ext_raw: [u8; 3],
    pub attribute: FileAttribute,
    pub create_ms: u8,
    pub create_time: FatTime,
    pub create_date: FatDate,
    pub last_access_date: FatDate,
    pub first_cluster: u32,
    pub last_modified_time: FatTime,
    pub last_modified_date: FatDate,
    pub size: u32,
}

impl DirEntry {
    pub fn parse(data: &[u8]) -> Result<Self, usize> {
        if data.len() < 0x20 {
            Err(data.len())?
        }

        let mut stem_raw = [0; 8];
        stem_raw.copy_from_slice(&data[0x00..0x08]);
        let mut ext_raw = [0; 3];
        ext_raw.copy_from_slice(&data[0x08..0x0B]);
        let attribute = FileAttribute::from_bits_truncate(data[11]);
        let create_ms = data[13];
        let create_time = FatTime::parse_u16(read_u16_le(data, 14));
        let create_date = FatDate::parse_u16(read_u16_le(data, 16));
        let last_access_date = FatDate::parse_u16(read_u16_le(data, 18));
        let first_cluster = ((read_u16_le(data, 20) as u32) << 16) | read_u16_le(data, 26) as u32;
        let last_modified_time = FatTime::parse_u16(read_u16_le(data, 22));
        let last_modified_date = FatDate::parse_u16(read_u16_le(data, 24));
        let size = read_u32_le(data, 28);

        Ok(Self {
            stem_raw,
            ext_raw,
            attribute,
            create_ms,
            create_time,
            create_date,
            last_access_date,
            first_cluster,
            last_modified_time,
            last_modified_date,
            size,
        })
    }

    pub fn new_root() -> Self {
        Self {
            stem_raw: [0; 8],
            ext_raw: [0; 3],
            attribute: FileAttribute::DIRECTORY,
            create_ms: 0,
            create_time: FatTime::new(0, 0, 0),
            create_date: FatDate::new(0, 0, 0),
            last_access_date: FatDate::new(0, 0, 0),
            first_cluster: 0,
            last_modified_time: FatTime::new(0, 0, 0),
            last_modified_date: FatDate::new(0, 0, 0),
            size: 0,
        }
    }

    pub fn stem(&self) -> &str {
        core::str::from_utf8(&self.stem_raw).unwrap()
    }
    pub fn ext(&self) -> &str {
        if self.is_directory() {
            return "   ";
        }
        core::str::from_utf8(&self.ext_raw).unwrap()
    }

    pub fn is_read_only(&self) -> bool {
        self.attribute.contains(FileAttribute::READ_ONLY)
    }
    pub fn is_hidden(&self) -> bool {
        self.attribute.contains(FileAttribute::HIDDEN)
    }
    pub fn is_system(&self) -> bool {
        self.attribute.contains(FileAttribute::SYSTEM)
    }
    pub fn is_volume_id(&self) -> bool {
        self.attribute.contains(FileAttribute::VOLUME_ID)
    }
    pub fn is_directory(&self) -> bool {
        self.attribute.contains(FileAttribute::DIRECTORY)
    }
    pub fn is_archive(&self) -> bool {
        self.attribute.contains(FileAttribute::ARCHIVE)
    }
    pub fn is_lfn_entry(&self) -> bool {
        self.is_read_only() && self.is_hidden() && self.is_system() && self.is_volume_id()
    }

    pub fn is_eod(&self) -> bool {
        self.stem_raw[0] == 0x00
    }
    pub fn is_unused(&self) -> bool {
        self.stem_raw[0] == 0xE5
    }
}

bitflags! {
    pub struct FileAttribute: u8 {
        const READ_ONLY = 0x01;
        const HIDDEN    = 0x02;
        const SYSTEM    = 0x04;
        const VOLUME_ID = 0x08;
        const DIRECTORY = 0x10;
        const ARCHIVE   = 0x20;
    }
}

#[cfg(test)]
#[test]
fn test_dir_entry() {
    let data = b"\x4B\x45\x52\x4E\x45\x4C\x20\x20\x45\x4C\x46\x20\x00\x00\x0F\xBE\xD0\x50\xD0\x50\x00\x00\x0F\xBE\xD0\x50\x02\x00\xF0\xE4\x0E\x00";

    let parsed = DirEntry::parse(data).unwrap();
    assert_eq!(parsed.stem(), "KERNEL  ");
    assert_eq!(parsed.ext(), "ELF");
    assert_eq!(parsed.attribute, FileAttribute::ARCHIVE);
    assert_eq!(parsed.create_ms, 0);
    assert_eq!(parsed.create_time, FatTime::new(23, 48, 30));
    assert_eq!(parsed.create_date, FatDate::new(20, 6, 16));
    assert_eq!(parsed.last_access_date, FatDate::new(20, 6, 16));
    assert_eq!(parsed.first_cluster, 2);
    assert_eq!(parsed.last_modified_time, FatTime::new(23, 48, 30));
    assert_eq!(parsed.last_modified_date, FatDate::new(20, 6, 16));
    assert_eq!(parsed.size, 976112);
}
