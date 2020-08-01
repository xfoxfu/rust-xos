use super::FatDevice;
use crate::DirEntry;
use crate::{vec, BlockError, Vec};

pub struct File<'a, T> {
    pub device: &'a T,
    pub entry: DirEntry,
}

impl<'a, T> File<'a, T>
where
    T: FatDevice,
{
    pub fn new(device: &'a T, entry: DirEntry) -> Self {
        Self { device, entry }
    }

    pub fn sectors(&self) -> Vec<u16> {
        let mut sectors = vec![];

        let mut cluster = Some(self.entry.first_cluster as u16);
        while let Some(cluster_id) = cluster {
            sectors.append(&mut self.device.fat_table().cluster_sector(cluster_id).collect());
            cluster = self.device.fat_table().next_cluster(cluster_id);
        }

        sectors
    }

    pub fn cluster_sectors(&self) -> Vec<(u16, Vec<u16>)> {
        let mut ret = vec![];

        let mut cluster = Some(self.entry.first_cluster as u16);
        while let Some(cluster_id) = cluster {
            ret.push((
                cluster_id,
                self.device.fat_table().cluster_sector(cluster_id).collect(),
            ));
            cluster = self.device.fat_table().next_cluster(cluster_id);
        }

        ret
    }

    pub fn load_to(&self, dst: &mut [u8]) -> Result<(), BlockError> {
        let mut offset = 0;
        let mut cluster = Some(self.entry.first_cluster as u16);
        while let Some(cluster_id) = cluster {
            for sector in self.device.fat_table().cluster_sector(cluster_id) {
                self.device
                    .read_block(sector as usize, 1, &mut dst[offset..(offset + 512)])?;
                offset += 512;
            }
            cluster = self.device.fat_table().next_cluster(cluster_id);
        }

        Ok(())
    }
}

impl<'a, T> core::fmt::Debug for File<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("File").field("entry", &self.entry).finish()
    }
}
