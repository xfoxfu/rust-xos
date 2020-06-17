use crate::drivers::IDE;
use fatpart::{BlockError, Device};
use spin::Mutex;

pub struct MutexIDE<'a>(pub &'a Mutex<IDE>);

impl<'a> Device for MutexIDE<'a> {
    fn block_size(&self) -> Result<usize, BlockError> {
        Ok(512)
    }
    fn read_block(&self, offset: usize, size: usize, buf: &mut [u8]) -> Result<(), BlockError> {
        self.0
            .try_lock()
            .unwrap()
            .read_lba(offset as u32, size as u8, buf)
            .map_err(|e| BlockError::WithStatus(e.bits() as usize))
    }
}
