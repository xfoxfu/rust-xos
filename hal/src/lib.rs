//! HAL

#![no_std]

mod block_device;
mod filesystem;
mod parted_disk;

pub use block_device::BlockDevice;
pub use block_device::BlockError;
