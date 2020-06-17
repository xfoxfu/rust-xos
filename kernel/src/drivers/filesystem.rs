use super::{device, MutexIDE};
use fatpart::{Directory, Entry, FATPartition, File};

pub type OsDevice = FATPartition<'static, MutexIDE<'static>>;
pub type OsDir = Directory<'static, OsDevice>;
pub type OsFile = File<'static, OsDevice>;
pub type OsEntry = Entry<'static, OsDevice>;

pub static FS: spin::Once<OsDevice> = spin::Once::new();

pub fn fs() -> &'static OsDevice {
    FS.r#try().unwrap()
}

pub fn init() {
    let [p0, _p1, _p2, _p3] = fatpart::Disk::new(device()).partitions();
    FS.call_once(|| FATPartition::new(p0));
}
