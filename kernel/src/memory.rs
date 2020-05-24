// This is from https://github.com/phil-opp/blog_os/blob/post-09/src/memory.rs

use boot::{MemoryMap, MemoryType};
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

/// Initialize a new OffsetPageTable.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
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

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator<T> {
    frames: T,
}

impl<T> BootInfoFrameAllocator<T>
where
    T: Iterator<Item = PhysFrame>,
{
    pub unsafe fn new(frames: T) -> Self {
        Self { frames }
    }
}

impl BootInfoFrameAllocator<()> {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(
        memory_map: &'static MemoryMap,
    ) -> BootInfoFrameAllocator<impl Iterator<Item = PhysFrame>> {
        let frames = memory_map
            .clone()
            .iter
            // 获得可用内存列表
            .filter(|r| r.ty == MemoryType::CONVENTIONAL)
            // 转换为段内可用起始地址列表（4KiB对齐）
            .flat_map(|r| (0..r.page_count).map(move |v| (v * 4096 + r.phys_start)))
            // 创建 `PhysFrame` 类型
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)));
        BootInfoFrameAllocator::new(frames)
    }
}

unsafe impl<T> FrameAllocator<Size4KiB> for BootInfoFrameAllocator<T>
where
    T: Iterator<Item = PhysFrame>,
{
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.frames.next()
    }
}
