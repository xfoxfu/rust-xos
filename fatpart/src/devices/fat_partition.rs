use super::Partition;
use crate::{BlockError, Device, DirEntry, Directory, FAT16Table, FatDevice, FAT16BPB};
#[cfg(not(test))]
use alloc::{vec, vec::Vec};
#[cfg(test)]
use std::{vec, vec::Vec};

pub struct FATPartition<'a, T> {
    partition: Partition<'a, T>,
    fat_meta: FAT16BPB,
    fat_raw: Vec<u8>,
}

impl<'a, T> FATPartition<'a, T>
where
    T: Device,
{
    pub fn new(partition: Partition<'a, T>) -> Self {
        let mut sector = vec![0; 512];

        partition.read_block(0, 1, &mut sector).unwrap();
        let fat_meta = FAT16BPB::parse(&sector).unwrap();

        let mut fat_raw =
            vec![0; fat_meta.bytes_per_sector as usize * fat_meta.sector_per_fat as usize];
        partition
            .read_block(
                fat_meta.hidden_sectors as usize,
                fat_meta.sector_per_fat as usize,
                &mut fat_raw,
            )
            .unwrap();

        Self {
            partition,
            fat_meta,
            fat_raw,
        }
    }

    pub fn root_directory(&'a self) -> Directory<'a, FATPartition<'a, T>> {
        Directory::new(self, DirEntry::new_root())
    }
}

impl<'a, T> Device for FATPartition<'a, T>
where
    T: Device,
{
    fn block_size(&self) -> Result<usize, BlockError> {
        self.partition.block_size()
    }
    fn read_block(&self, offset: usize, size: usize, buf: &mut [u8]) -> Result<(), BlockError> {
        self.partition.read_block(offset, size, buf)
    }
}

impl<'a, T> FatDevice for FATPartition<'a, T>
where
    T: Device,
{
    fn fat_meta(&self) -> &crate::FAT16BPB {
        &self.fat_meta
    }
    fn fat_table(&self) -> crate::FAT16Table {
        FAT16Table::new(&self.fat_meta, &self.fat_raw).unwrap()
    }
}
