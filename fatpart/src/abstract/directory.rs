use super::{BlockError, Entry, FatDevice, File};
use crate::DirEntry;
#[cfg(not(test))]
use alloc::{vec, vec::Vec};
#[cfg(test)]
use std::{vec, vec::Vec};

pub struct Directory<'a, T> {
    device: &'a T,
    entry: DirEntry,
}

impl<'a, T> Directory<'a, T>
where
    T: FatDevice,
{
    pub fn new(device: &'a T, entry: DirEntry) -> Self {
        Self { device, entry }
    }

    fn childs_from_sector(&mut self, sector: usize) -> Result<Vec<Entry<'a, T>>, BlockError> {
        let mut childs = vec![];
        let mut buf = vec![0; self.device.block_size()?];

        self.device.read_block(sector, 1, &mut buf)?;
        for i in (0..512).step_by(0x20) {
            let file = crate::DirEntry::parse(&buf[i..]).map_err(|s| BlockError::WithStatus(s))?;
            if file.is_eod() {
                break;
            }
            if !file.is_unused() && !file.is_lfn_entry() {
                let entry = if file.is_directory() {
                    Directory::new(self.device, file).into()
                } else {
                    File::new(self.device, file).into()
                };
                childs.push(entry);
            }
        }

        Ok(childs)
    }

    fn childs_from_cluster(&mut self, cluster: usize) -> Result<Vec<Entry<'a, T>>, BlockError> {
        let mut childs = vec![];
        for sector in self.device.fat_table().cluster_sector(cluster as u16) {
            childs.append(&mut self.childs_from_sector(sector as usize)?);
        }
        Ok(childs)
    }

    pub fn load_childs(&mut self) -> Result<Vec<Entry<'a, T>>, BlockError> {
        let mut childs = vec![];

        let mut cluster = Some(self.entry.first_cluster as u16);
        while let Some(cluster_id) = cluster {
            childs.append(&mut self.childs_from_cluster(cluster_id as usize)?);
            cluster = self.device.fat_table().next_cluster(cluster_id);
        }

        Ok(childs)
    }
}

impl<'a, T> core::fmt::Debug for Directory<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Directory")
            .field("entry", &self.entry)
            .finish()
    }
}
