use super::FAT16BPB;
use crate::read_u16_le;
use core::ops::Range;

pub struct FAT16Table<'a> {
    data: &'a [u8],
    bpb: &'a FAT16BPB,
}

impl<'a> FAT16Table<'a> {
    pub fn new(bpb: &'a FAT16BPB, data: &'a [u8]) -> Result<Self, usize> {
        if data.len() < (bpb.sector_per_fat as usize * bpb.bytes_per_sector as usize) {
            Err(data.len())?
        }

        Ok(Self { data, bpb })
    }

    /// 获取第 id 个 FAT 表项对应的扇区范围
    pub fn cluster_sector(&self, id: u16) -> Range<u16> {
        let start = self.bpb.perserved_sectors
            + self.bpb.fat_count as u16 * self.bpb.sector_per_fat
            + self.bpb.sector_per_cluster as u16 * id;
        let end = start + self.bpb.sector_per_cluster as u16;
        start..end
    }

    /// 获取第 id 个 FAT 表项的下一个 FAT 表项
    pub fn next_cluster(&self, id: u16) -> Option<u16> {
        let raw = read_u16_le(self.data, 2 * id as usize);
        if raw > 0x0001 && raw < 0xFFF0 {
            Some(raw)
        } else {
            None
        }
    }
}
