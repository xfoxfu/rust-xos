use x86_64::structures::paging::OffsetPageTable;
use x86_64::VirtAddr;

#[derive(Debug)]
pub struct Process<'a> {
    page_table: OffsetPageTable<'a>,
    rsp: VirtAddr,
    rip: VirtAddr,
}
