use crate::{BlockError, Device, PartitionMeta};

pub struct Partition<'a, T> {
    inner: &'a T,
    meta: PartitionMeta,
}

impl<'a, T> Partition<'a, T>
where
    T: Device,
{
    pub fn new(inner: &'a T, meta: PartitionMeta) -> Self {
        Self { inner, meta }
    }
}

impl<'a, T> Device for Partition<'a, T>
where
    T: Device,
{
    fn block_size(&self) -> Result<usize, BlockError> {
        self.inner.block_size()
    }
    fn read_block(&self, offset: usize, size: usize, buf: &mut [u8]) -> Result<(), BlockError> {
        self.inner
            .read_block(offset + self.meta.begin_lba as usize, size, buf)
    }
}
