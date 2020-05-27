// use thiserror::Error;

// #[derive(Error, Debug)]
pub enum BlockError {
    // #[error("device busy")]
    Busy,
    // #[error("unknown device")]
    UnknownDevice,
    // #[error("unknown error")]
    Unknown,
    // #[error("status {0} error")]
    WithStatus(usize),
}

pub trait BlockDevice {
    fn block_size(&self) -> Result<usize, BlockError>;
    fn read_block(&mut self, offset: usize, buf: &mut [u8]) -> Result<(), BlockError>;
}
