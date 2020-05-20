#![no_std]
#![deny(warnings)]

use core::fmt;
pub use uefi::data_types::chars::*;
pub use uefi::data_types::*;
pub use uefi::prelude::SystemTable;
pub use uefi::proto::console::gop::{GraphicsOutput, ModeInfo};
pub use uefi::table::boot::{MemoryAttribute, MemoryDescriptor, MemoryMapIter, MemoryType};
pub use uefi::table::runtime::*;
pub use uefi::table::Runtime;
pub use uefi::Status as UefiStatus;

/// This structure represents the information that the bootloader passes to the kernel.
#[repr(C)]
pub struct BootInfo {
    /// The memory map
    pub memory_map: MemoryMap,

    /// The offset into the virtual address space where the physical memory is mapped.
    pub physical_memory_offset: u64,

    /// The graphic output information
    pub graphic_info: GraphicInfo,

    /// UEFI SystemTable
    pub system_table: SystemTable<Runtime>,
}

pub struct MemoryMap {
    pub iter: MemoryMapIter<'static>,
}

/// Graphic output information
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct GraphicInfo {
    /// Graphic mode
    pub mode: ModeInfo,
    /// Framebuffer base physical address
    pub fb_addr: u64,
    /// Framebuffer size
    pub fb_size: u64,
}

impl Clone for MemoryMap {
    fn clone(&self) -> Self {
        unsafe { core::ptr::read(self) }
    }
}

impl fmt::Debug for MemoryMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_list();
        for mmap in self.clone().iter {
            f.entry(mmap);
        }
        f.finish()
    }
}
