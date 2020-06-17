use crate::{read_u16_le, read_u32_le};

#[derive(Debug, Eq, PartialEq)]
pub struct FAT16BPB {
    /// OEM名称（空格补齐）。
    pub oem_name_raw: [u8; 8],
    /// 每个扇区的字节数。基本输入输出系统参数块从这里开始。
    pub bytes_per_sector: u16,
    /// 每簇扇区数
    pub sector_per_cluster: u8,
    /// 保留扇区数（包括启动扇区）
    pub perserved_sectors: u16,
    /// 文件分配表数目
    pub fat_count: u8,
    /// 最大根目录条目个数
    pub max_root_dir_items: u16,
    /// 总扇区数
    pub total_sectors: u32,
    /// 介质描述
    pub media_type: u8,
    /// 每个文件分配表的扇区（FAT16）
    pub sector_per_fat: u16,
    /// 每磁道的扇区
    pub sector_per_track: u16,
    /// 磁头数
    pub tracks: u16,
    /// 隐藏扇区
    pub hidden_sectors: u32,
    /// 物理驱动器号（FAT16）
    pub drive_number: u8,
    /// 当前磁头（FAT16）
    pub current_head: u8,
    /// 签名（FAT16）
    pub signature: u8,
    /// ID（FAT16）
    pub id: u32,
    /// 卷标（非FAT32）
    pub volume_name_raw: [u8; 11],
    /// FAT文件系统类型（如FAT、FAT12、FAT16）
    pub fs_type_raw: [u8; 8],
}

impl FAT16BPB {
    pub fn parse(data: &[u8]) -> Result<Self, usize> {
        if data.len() < 0x3E {
            Err(data.len())?
        }

        // 0x00  3  跳转指令（跳过开头一段区域）
        // 0x03  8  OEM名称（空格补齐）。MS-DOS检查这个区域以确定使用启动记录中的哪一部分数据 [1] 。常见值是IBM 3.3（在“IBM”和“3.3”之间有两个空格）和MSDOS5.0.
        let mut oem_name_raw = [0; 8];
        oem_name_raw.copy_from_slice(&data[0x03..0x03 + 8]);
        // 0x0b  2  每个扇区的字节数。基本输入输出系统参数块从这里开始。
        let bytes_per_sector = read_u16_le(data, 0x0b);
        // 0x0d  1  每簇扇区数
        let sector_per_cluster = data[0x0D];
        // 0x0e  2  保留扇区数（包括启动扇区）
        let perserved_sectors = read_u16_le(data, 0x0e);
        // 0x10  1  文件分配表数目
        let fat_count = data[0x10];
        // 0x11  2  最大根目录条目个数
        let max_root_dir_items = read_u16_le(data, 0x11);
        // 0x13  2  总扇区数（如果是0，就使用偏移0x20处的4字节值）
        let total_sectors_a = read_u16_le(data, 0x13);
        // 0x15  1  介质描述
        let media_type = data[0x15];
        // 0x16  2  每个文件分配表的扇区（FAT16）
        let sector_per_fat = read_u16_le(data, 0x16);
        // 0x18  2  每磁道的扇区
        let sector_per_track = read_u16_le(data, 0x18);
        // 0x1a  2  磁头数
        let tracks = read_u16_le(data, 0x1a);
        // 0x1c  4  隐藏扇区
        let hidden_sectors = read_u32_le(data, 0x1c);
        // 0x20  4  总扇区数（如果超过65535，参见偏移0x13）
        let total_sectors_b = read_u32_le(data, 0x20);
        // 0x24  1  物理驱动器个数（FAT16）
        let drive_number = data[0x24];
        // 0x25  1  当前磁头（FAT16）
        let current_head = data[0x25];
        // 0x26  1  签名（FAT16）
        let signature = data[0x26];
        // 0x27  4  ID（FAT16）
        let id = read_u32_le(data, 0x27);
        // 0x2b  11  卷标（非FAT32）
        let mut volume_name_raw = [0; 11];
        volume_name_raw.copy_from_slice(&data[0x2b..0x2b + 11]);
        // 0x36  8  FAT文件系统类型（如FAT、FAT12、FAT16）
        let mut fs_type_raw = [0; 8];
        fs_type_raw.copy_from_slice(&data[0x36..0x36 + 8]);

        let total_sectors = if total_sectors_a != 0 {
            total_sectors_a as u32
        } else {
            total_sectors_b
        };

        Ok(Self {
            oem_name_raw,
            bytes_per_sector,
            sector_per_cluster,
            perserved_sectors,
            fat_count,
            max_root_dir_items,
            total_sectors,
            media_type,
            sector_per_fat,
            sector_per_track,
            tracks,
            hidden_sectors,
            drive_number,
            current_head,
            signature,
            id,
            volume_name_raw,
            fs_type_raw,
        })
    }

    pub fn oem_name(&self) -> &str {
        core::str::from_utf8(&self.oem_name_raw).unwrap()
    }
    pub fn volume_name(&self) -> &str {
        core::str::from_utf8(&self.volume_name_raw).unwrap()
    }
    pub fn fs_type(&self) -> &str {
        core::str::from_utf8(&self.fs_type_raw).unwrap()
    }
}

#[cfg(test)]
#[test]
fn test_bpb_parse() {
    let data = &[
        0xEB, 0x3E, 0x90, 0x4D, 0x53, 0x57, 0x49, 0x4E, 0x34, 0x2E, 0x31, 0x00, 0x02, 0x10, 0x01,
        0x00, 0x02, 0x00, 0x02, 0x00, 0x00, 0xF8, 0xFC, 0x00, 0x3F, 0x00, 0x10, 0x00, 0x3F, 0x00,
        0x00, 0x00, 0xC1, 0xBF, 0x0F, 0x00, 0x80, 0x00, 0x29, 0xFD, 0x1A, 0xBE, 0xFA, 0x51, 0x45,
        0x4D, 0x55, 0x20, 0x56, 0x56, 0x46, 0x41, 0x54, 0x20, 0x46, 0x41, 0x54, 0x31, 0x36, 0x20,
        0x20, 0x20,
    ];

    let parsed = FAT16BPB::parse(data).unwrap();
    assert_eq!(parsed.oem_name(), "MSWIN4.1");
    assert_eq!(parsed.bytes_per_sector, 512);
    assert_eq!(parsed.sector_per_cluster, 16);
    assert_eq!(parsed.perserved_sectors, 1);
    assert_eq!(parsed.fat_count, 2);
    assert_eq!(parsed.max_root_dir_items, 512);
    assert_eq!(parsed.total_sectors, 1032129);
    assert_eq!(parsed.media_type, 248);
    assert_eq!(parsed.sector_per_fat, 252);
    assert_eq!(parsed.sector_per_track, 63);
    assert_eq!(parsed.tracks, 16);
    assert_eq!(parsed.hidden_sectors, 63);
    assert_eq!(parsed.drive_number, 0x80);
    assert_eq!(parsed.current_head, 0);
    assert_eq!(parsed.signature, 41);
    assert_eq!(parsed.id, 4206762749);
    assert_eq!(parsed.volume_name(), "QEMU VVFAT ");
    assert_eq!(parsed.fs_type(), "FAT16   ");
}
