//! Simple ELF OS Loader on UEFI
//!
//! 1. Load config from "\EFI\Boot\rboot.conf"
//! 2. Load kernel ELF file
//! 3. Map ELF segments to virtual memory
//! 4. Map kernel stack and all physical memory
//! 5. Startup all processors
//! 6. Exit boot and jump to ELF entry

#![no_std]
#![no_main]
#![feature(llvm_asm, abi_efiapi)]
// #![deny(warnings)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;

use alloc::boxed::Box;
use boot::{BootInfo, GraphicInfo, MemoryMap};
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::media::file::*;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::proto::pi::mp::MPServices;
use uefi::table::boot::*;
use uefi::table::cfg::{ACPI2_GUID, SMBIOS_GUID};
use x86_64::registers::control::*;
use x86_64::structures::paging::*;
use x86_64::{PhysAddr, VirtAddr};
use xmas_elf::ElfFile;

mod config;
mod page_table;

const CONFIG_PATH: &str = "\\EFI\\BOOT\\rboot.conf";

#[entry]
fn efi_main(image: uefi::Handle, st: SystemTable<Boot>) -> Status {
    // Initialize utilities (logging, memory allocation...)
    uefi_services::init(&st).expect_success("failed to initialize utilities");

    info!("bootloader is running");
    let bs = st.boot_services();
    let config = {
        let mut file = open_file(bs, CONFIG_PATH);
        let buf = load_file(bs, &mut file);
        config::Config::parse(buf)
    };

    let graphic_info = init_graphic(bs, config.resolution);
    info!("config: {:#x?}", config);

    let acpi2_addr = st
        .config_table()
        .iter()
        .find(|entry| entry.guid == ACPI2_GUID)
        .expect("failed to find ACPI 2 RSDP")
        .address;
    info!("acpi2: {:?}", acpi2_addr);

    let smbios_addr = st
        .config_table()
        .iter()
        .find(|entry| entry.guid == SMBIOS_GUID)
        .expect("failed to find SMBIOS")
        .address;
    info!("smbios: {:?}", smbios_addr);

    let elf = {
        let mut file = open_file(bs, config.kernel_path);
        let buf = load_file(bs, &mut file);
        ElfFile::new(buf).expect("failed to parse ELF")
    };
    unsafe {
        ENTRY = elf.header.pt2.entry_point() as usize;
    }

    let max_mmap_size = st.boot_services().memory_map_size();
    let mmap_storage = Box::leak(vec![0; max_mmap_size].into_boxed_slice());
    let mmap_iter = st
        .boot_services()
        .memory_map(mmap_storage)
        .expect_success("failed to get memory map")
        .1;
    let max_phys_addr = mmap_iter
        .map(|m| m.phys_start + m.page_count * 0x1000)
        .max()
        .unwrap()
        .max(0x1_0000_0000); // include IOAPIC MMIO area

    let mut page_table = current_page_table();
    // root page table is readonly
    // disable write protect
    unsafe {
        Cr0::update(|f| f.remove(Cr0Flags::WRITE_PROTECT));
        Efer::update(|f| f.insert(EferFlags::NO_EXECUTE_ENABLE));
    }
    page_table::map_elf(&elf, &mut page_table, &mut UEFIFrameAllocator(bs))
        .expect("failed to map ELF");
    page_table::map_stack(
        config.kernel_stack_address,
        config.kernel_stack_size,
        &mut page_table,
        &mut UEFIFrameAllocator(bs),
    )
    .expect("failed to map stack");
    page_table::map_physical_memory(
        config.physical_memory_offset,
        max_phys_addr,
        &mut page_table,
        &mut UEFIFrameAllocator(bs),
    );
    // recover write protect
    unsafe {
        Cr0::update(|f| f.insert(Cr0Flags::WRITE_PROTECT));
    }

    // FIXME: multi-core
    //  All application processors will be shutdown after ExitBootService.
    //  Disable now.
    // start_aps(bs);

    info!("exit boot services");

    let (rt, mmap_iter) = st
        .exit_boot_services(image, mmap_storage)
        .expect_success("Failed to exit boot services");
    // NOTE: alloc & log can no longer be used

    // construct BootInfo
    let bootinfo = BootInfo {
        memory_map: MemoryMap { iter: mmap_iter },
        physical_memory_offset: config.physical_memory_offset,
        graphic_info,
        system_table: rt,
    };
    let stacktop = config.kernel_stack_address + config.kernel_stack_size * 0x1000;
    unsafe {
        jump_to_entry(&bootinfo, stacktop);
    }
}

/// Open file at `path`
fn open_file(bs: &BootServices, path: &str) -> RegularFile {
    info!("opening file: {}", path);
    // FIXME: use LoadedImageProtocol to get the FileSystem of this image
    let fs = bs
        .locate_protocol::<SimpleFileSystem>()
        .expect_success("failed to get FileSystem");
    let fs = unsafe { &mut *fs.get() };

    let mut root = fs.open_volume().expect_success("failed to open volume");
    let handle = root
        .open(path, FileMode::Read, FileAttribute::empty())
        .expect_success("failed to open file");

    match handle.into_type().expect_success("failed to into_type") {
        FileType::Regular(regular) => regular,
        _ => panic!("Invalid file type"),
    }
}

/// Load file to new allocated pages
fn load_file(bs: &BootServices, file: &mut RegularFile) -> &'static mut [u8] {
    info!("loading file to memory");
    let mut info_buf = [0u8; 0x100];
    let info = file
        .get_info::<FileInfo>(&mut info_buf)
        .expect_success("failed to get file info");
    let pages = info.file_size() as usize / 0x1000 + 1;
    let mem_start = bs
        .allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, pages)
        .expect_success("failed to allocate pages");
    let buf = unsafe { core::slice::from_raw_parts_mut(mem_start as *mut u8, pages * 0x1000) };
    let len = file.read(buf).expect_success("failed to read file");
    info!("file size={}", len);
    &mut buf[..len]
}

/// If `resolution` is some, then set graphic mode matching the resolution.
/// Return information of the final graphic mode.
fn init_graphic(bs: &BootServices, resolution: Option<(usize, usize)>) -> GraphicInfo {
    let gop = bs
        .locate_protocol::<GraphicsOutput>()
        .expect_success("failed to get GraphicsOutput");
    let gop = unsafe { &mut *gop.get() };

    if let Some(resolution) = resolution {
        let mode = gop
            .modes()
            .map(|mode| mode.expect("Warnings encountered while querying mode"))
            .find(|ref mode| {
                let info = mode.info();
                info.resolution() == resolution
            })
            .expect("graphic mode not found");
        info!("switching graphic mode");
        gop.set_mode(&mode)
            .expect_success("Failed to set graphics mode");
    }
    GraphicInfo {
        mode: gop.current_mode_info(),
        fb_addr: gop.frame_buffer().as_mut_ptr() as u64,
        fb_size: gop.frame_buffer().size() as u64,
    }
}

/// Get current page table from CR3
fn current_page_table() -> OffsetPageTable<'static> {
    let p4_table_addr = Cr3::read().0.start_address().as_u64();
    let p4_table = unsafe { &mut *(p4_table_addr as *mut PageTable) };
    unsafe { OffsetPageTable::new(p4_table, VirtAddr::new(0)) }
}

/// Use `BootServices::allocate_pages()` as frame allocator
struct UEFIFrameAllocator<'a>(&'a BootServices);

unsafe impl FrameAllocator<Size4KiB> for UEFIFrameAllocator<'_> {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let addr = self
            .0
            .allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 1)
            .expect_success("failed to allocate frame");
        let frame = PhysFrame::containing_address(PhysAddr::new(addr));
        Some(frame)
    }
}

/// Startup all application processors
#[allow(dead_code)]
fn start_aps(bs: &BootServices) {
    info!("starting application processors");
    let mp = bs
        .locate_protocol::<MPServices>()
        .expect_success("failed to get MPServices");
    let mp = mp.get();

    // this event will never be signaled
    let event = unsafe {
        bs.create_event(EventType::empty(), Tpl::APPLICATION, None)
            .expect_success("failed to create event")
    };

    // workaround as uefi crate do not implement non-blocking call
    use core::ffi::c_void;
    use uefi::proto::pi::mp::Procedure;
    type StartupAllAps = extern "efiapi" fn(
        this: *const MPServices,
        procedure: Procedure,
        single_thread: bool,
        wait_event: *mut c_void,
        timeout_in_micro_seconds: usize,
        procedure_argument: *mut c_void,
        failed_cpu_list: *mut *mut usize,
    ) -> Status;
    let startup_all_aps = unsafe { *((mp as *const usize).add(2) as *const StartupAllAps) };
    let event_ptr = unsafe { core::mem::transmute(event) };
    let status = startup_all_aps(
        mp,
        ap_main,
        false,
        event_ptr,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if !status.is_success() {
        warn!(
            "failed to startup all application processors with {:?}",
            status
        );
    }
}

/// Main function for application processors
extern "efiapi" fn ap_main(_arg: *mut core::ffi::c_void) {
    unsafe {
        jump_to_entry(core::ptr::null(), 0);
    }
}

/// Jump to ELF entry according to global variable `ENTRY`
unsafe fn jump_to_entry(bootinfo: *const BootInfo, stacktop: u64) -> ! {
    llvm_asm!("call $0" :: "r"(ENTRY), "{rsp}"(stacktop), "{rdi}"(bootinfo) :: "intel");
    loop {}
}

/// The entry point of kernel, set by BSP.
static mut ENTRY: usize = 0;
