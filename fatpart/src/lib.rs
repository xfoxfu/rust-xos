#![no_std]
#![allow(dead_code)]

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(not(test))]
extern crate alloc;

mod r#abstract;
mod devices;
mod r#struct;
mod utils;

pub(crate) use utils::*;

pub use devices::*;
pub use r#abstract::*;
pub use r#struct::*;
