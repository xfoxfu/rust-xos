// This is from https://github.com/phil-opp/blog_os/blob/post-09/src/memory.rs

use boot::{MemoryMap, MemoryType};
use spin::Mutex;
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

pub static OFFSET_PAGE_TABLE: Mutex<Option<OffsetPageTable>> = Mutex::new(None);
pub static FRAME_ALLOCATOR: Mutex<Option<BootInfoFrameAllocator>> = Mutex::new(None);

pub unsafe fn init(physical_memory_offset: VirtAddr, memory_map: &'static MemoryMap) {
    *OFFSET_PAGE_TABLE.lock() = Some(inner_init(physical_memory_offset));
    *FRAME_ALLOCATOR.lock() = Some(BootInfoFrameAllocator::init(memory_map));
}

/// Initialize a new OffsetPageTable.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn inner_init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

pub fn physical_to_virtual(addr: usize) -> usize {
    addr + 0xFFFF800000000000
}

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

type BootInfoFrameIter = impl Iterator<Item = PhysFrame>;

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    frames: BootInfoFrameIter,
}

fn create_frame_iter(memory_map: &'static MemoryMap) -> BootInfoFrameIter {
    memory_map
        .clone()
        .iter
        // 获得可用内存列表
        .filter(|r| r.ty == MemoryType::CONVENTIONAL)
        // 转换为段内可用起始地址列表（4KiB对齐）
        .flat_map(|r| (0..r.page_count).map(move |v| (v * 4096 + r.phys_start)))
        // 创建 `PhysFrame` 类型
        .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> BootInfoFrameAllocator {
        BootInfoFrameAllocator {
            frames: create_frame_iter(memory_map),
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.frames.next()
    }
}
