#![no_std]
#![allow(dead_code)]

#[cfg(test)]
#[macro_use]
extern crate std;

mod partition;

pub struct FAT12 {}

pub use partition::{MBRPartitionTable, PartitionMeta};
