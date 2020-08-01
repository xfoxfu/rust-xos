use fatpart::Device;

pub struct SysDevice;

impl Device for SysDevice {
    fn block_size(&self) -> Result<usize, fatpart::BlockError> {
        Ok(512) // TODO: read from system
    }
    fn read_block(
        &self,
        offset: usize,
        size: usize,
        buf: &mut [u8],
    ) -> Result<(), fatpart::BlockError> {
        xlibr::sys_read_disk(offset as u64, buf);
        Ok(())
    }
}
