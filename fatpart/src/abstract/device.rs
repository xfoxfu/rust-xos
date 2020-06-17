#[derive(Debug, Eq, PartialEq)]
pub enum BlockError {
    Busy,
    UnknownDevice,
    Unknown,
    WithStatus(usize),
}

pub trait Device {
    fn block_size(&self) -> Result<usize, BlockError>;
    fn read_block(&self, offset: usize, size: usize, buf: &mut [u8]) -> Result<(), BlockError>;
}

pub trait FatDevice: Device {
    fn fat_meta(&self) -> &crate::FAT16BPB;
    fn fat_table(&self) -> crate::FAT16Table;
}
