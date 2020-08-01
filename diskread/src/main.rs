#![no_std]
#![no_main]
#![feature(asm)]

extern crate alloc;
extern crate rlibc;
#[macro_use]
extern crate xlibr;

use fatpart::{Device, Entry, FATPartition, FatDevice};
use sys_device::SysDevice;

mod sys_device;

#[export_name = "_start"]
pub extern "C" fn __impl_start() -> ! {
    println!("DISK DEBUGGER");
    let dev = SysDevice;

    println!("--------------------------------------------------------------------------------");
    println!("                                VIEW SECTOR DATA                                ");
    print!("sector id> ");
    let id = xlibr::read_u64() as usize;
    println!();

    let mut buf = alloc::vec::Vec::with_capacity(512);
    buf.resize(512, 0xFF);
    dev.read_block(id, 1, &mut buf);

    for i in 0..(512 / 32) {
        for j in 0..8 {
            for k in 0..4 {
                print!("{:02x}", buf[i * 32 + j * 4 + k]);
            }
            print!(" ");
        }
        println!();
    }

    println!("--------------------------------------------------------------------------------");
    println!("                             GET FILE CLUSTER INFO                              ");

    let [p0, _, _, _] = fatpart::Disk::new(&dev).partitions();
    println!("part info = {:?}", p0.meta());
    let part = FATPartition::new(p0);
    println!("part meta = {:?}", part.fat_meta());

    let mut root = part.root_directory();
    let files = root.load_childs().unwrap();

    for (v, p) in files.iter().enumerate() {
        if let Entry::File(f) = p {
            println!(
                "{} - {}.{} {}B",
                v,
                f.entry.stem(),
                f.entry.ext(),
                f.entry.size
            );
        }
    }

    print!("type file id to view cluster info> ");
    let id = xlibr::read_u64() as usize;
    if id < files.len() {
        if let Entry::File(f) = &files[id] {
            println!("\nfile = {}.{}", f.entry.stem(), f.entry.ext());
            let mut count = 0;
            for (c, s) in f.cluster_sectors().into_iter() {
                print!("{} =>", c);
                for elem in s.into_iter() {
                    if count * 512 <= f.entry.size {
                        print!(" {}", elem);
                    }
                    count += 1;
                }
                println!();
            }
        } else {
            unreachable!()
        }
    } else {
        println!("invalid id.");
    }

    xlibr::sys_exit()
}
