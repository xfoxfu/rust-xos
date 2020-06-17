#![no_std]
#![allow(dead_code)]

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(not(test))]
extern crate alloc;

#[allow(unused_imports)]
#[macro_use]
extern crate log;

#[cfg(not(test))]
pub(crate) use alloc::{vec, vec::Vec};
#[cfg(test)]
pub(crate) use std::{vec, vec::Vec};

mod r#abstract;
mod devices;
mod r#struct;
mod utils;

pub(crate) use utils::*;

pub use devices::*;
pub use r#abstract::*;
pub use r#struct::*;
