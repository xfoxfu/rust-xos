use super::FatDevice;
use crate::DirEntry;

pub struct File<'a, T> {
    device: &'a T,
    entry: DirEntry,
}

impl<'a, T> File<'a, T>
where
    T: FatDevice,
{
    pub fn new(device: &'a T, entry: DirEntry) -> Self {
        Self { device, entry }
    }
}

impl<'a, T> core::fmt::Debug for File<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("File").field("entry", &self.entry).finish()
    }
}
