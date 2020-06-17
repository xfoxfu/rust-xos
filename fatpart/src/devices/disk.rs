use crate::{vec, BlockError, Device, Partition};

pub struct Disk<'a, T> {
    inner: &'a T,
}

impl<'a, T> Disk<'a, T>
where
    T: Device,
{
    pub fn new(inner: &'a T) -> Self {
        Self { inner }
    }

    pub fn partitions(&self) -> [Partition<'a, T>; 4] {
        let mut sector = vec![0; 512];
        self.read_block(0, 1, &mut sector).unwrap();
        let parts = crate::MBRPartitionTable::parse_sector(&sector).unwrap();
        [
            Partition::new(self.inner, parts.partition0),
            Partition::new(self.inner, parts.partition1),
            Partition::new(self.inner, parts.partition2),
            Partition::new(self.inner, parts.partition3),
        ]
    }
}

impl<'a, T> Device for Disk<'a, T>
where
    T: Device,
{
    fn block_size(&self) -> Result<usize, BlockError> {
        self.inner.block_size()
    }
    fn read_block(&self, offset: usize, size: usize, buf: &mut [u8]) -> Result<(), BlockError> {
        self.inner.read_block(offset, size, buf)
    }
}
