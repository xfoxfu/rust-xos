#![no_std]
#![no_main]
#![feature(llvm_asm, abi_efiapi)]
#![deny(warnings)]

#[macro_use]
extern crate log;

// link rlibc for memcpy, etc.
// see https://github.com/rust-lang/wg-cargo-std-aware/issues/53#issuecomment-576242978
extern crate rlibc;

use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::console::text::Input;

const COLORS: [u32; 19] = [
    0x00f44336, 0x00e91e63, 0x009c27b0, 0x00673ab7, 0x003f51b5, 0x002196f3, 0x0003a9f4, 0x0000bcd4,
    0x00009688, 0x004caf50, 0x008bc34a, 0x00cddc39, 0x00ffeb3b, 0x00ffc107, 0x00ff9800, 0x00ff5722,
    0x00795548, 0x009e9e9e, 0x00607d8b,
];

#[entry]
fn efi_main(_image: uefi::Handle, st: SystemTable<Boot>) -> Status {
    uefi_services::init(&st).expect_success("failed to initialize utilities");

    info!("bootloader is running");
    let bs = st.boot_services();

    let gop = bs
        .locate_protocol::<GraphicsOutput>()
        .expect_success("failed to get GraphicsOutput");
    let gop = unsafe { &mut *gop.get() };

    let input = bs
        .locate_protocol::<Input>()
        .expect_success("failed to get Input");
    let input = unsafe { &mut *input.get() };

    let mode = gop.current_mode_info();
    let (display_x, display_y) = mode.resolution();
    let (display_x, display_y) = (display_x as isize, display_y as isize);
    let fb_addr = gop.frame_buffer().as_mut_ptr() as u64;
    let fb_size = gop.frame_buffer().size() as u64;

    info!("fb_addr={} fb_size={}", fb_addr, fb_size);

    for i in 0..display_x * display_y {
        unsafe {
            *(fb_addr as *mut u32).offset(i).as_mut().unwrap() = 0x000F0F0F;
        }
    }

    info!("PRESS ESC TO SHUTDOWN");
    bs.stall(10_000); // wait for 10ms

    let base_x: isize = 0;
    let base_y: isize = 0;
    let max_x: isize = display_x;
    let max_y: isize = display_y;

    let mut row = base_x;
    let mut col = base_y;

    let mut row_incr = 2;
    let mut col_incr = 1;
    let mut color = 0;
    while input.read_key().unwrap().unwrap()
        != Some(uefi::proto::console::text::Key::Special(
            uefi::proto::console::text::ScanCode::ESCAPE,
        ))
    {
        if col >= base_x && col < max_x && row >= base_y && row < max_y {
            unsafe {
                *(fb_addr as *mut u32)
                    .offset(row * display_x + col)
                    .as_mut()
                    .unwrap() = COLORS[color];
            }
        } else {
            // warn!("invalid position ({}, {})", col, row);
        }

        row += row_incr;
        col += col_incr;

        if col <= base_x || col > max_x {
            col_incr = -col_incr;
        }
        if row <= base_y || row > max_y {
            row_incr = -row_incr;
        }

        if col <= base_x || col > max_x || row <= base_y || row > max_y {
            color += 1;
        }

        if color >= 19 {
            color = 0;
        }

        bs.stall(10_000); // wait for 10ms
    }

    // shutdown
    info!("Shut down ......");
    bs.stall(1000_000); // wait for 100ms
    let rs = st.runtime_services();
    rs.reset(
        uefi::table::runtime::ResetType::Shutdown,
        Status::SUCCESS,
        None,
    );
}
