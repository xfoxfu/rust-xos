use super::{device, MutexIDE};
use alloc::vec;
use fatpart::{Device, Directory, Entry, FATPartition, File, Partition};

pub type OsDevice = FATPartition<'static, MutexIDE<'static>>;
pub type OsDir = Directory<'static, OsDevice>;
pub type OsFile = File<'static, OsDevice>;
pub type OsEntry = Entry<'static, OsDevice>;

pub static FS: spin::Once<OsDevice> = spin::Once::new();

pub fn fs() -> &'static OsDevice {
    FS.r#try().unwrap()
}

pub fn init() {
    let mut sector = vec![0; 512];
    device().read_block(0, 1, &mut sector).unwrap();
    let partition = fatpart::MBRPartitionTable::parse_sector(&sector)
        .unwrap()
        .partition0;
    info!(
        "parsed partition, begin={}, total={}",
        partition.begin_lba, partition.total_lba
    );

    FS.call_once(|| FATPartition::new(Partition::new(device(), partition)));
}
