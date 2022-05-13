use x86_64::instructions::port::Port;

once_mutex!(pub DRIVE: IDE);
pub static DRIVE_WRAP: spin::Once<super::MutexIDE> = spin::Once::new();

/// 初始化键盘输入设备
///
/// 需要确保内存已经初始化；在键盘中断初始化完成前，无法获得输入
pub unsafe fn init() {
    init_DRIVE(IDE::from_id(0));
    drive().unwrap().init().unwrap();
    DRIVE_WRAP.call_once(|| super::MutexIDE(&DRIVE.get().unwrap()));
    debug!("block device initialized");
}

guard_access_fn!(pub drive(DRIVE: IDE));

pub fn device() -> &'static super::MutexIDE<'static> {
    DRIVE_WRAP.get().unwrap()
}

pub struct IDE {
    is_slave: bool,
    ports: IDEPorts,
}

impl IDE {
    pub fn new(is_slave: bool, ports: IDEPorts) -> Self {
        Self { is_slave, ports }
    }

    pub fn from_id(id: u8) -> Self {
        assert!(/* id >= 0 && */ id <= 3, "id should be in range 0 - 3");
        IDE::new(
            match id {
                0 | 2 => false,
                1 | 3 => true,
                _ => unreachable!(),
            },
            match id {
                // IDE Channel 1
                0 | 1 => IDEPorts::new(0x1F0, 0x3F4),
                // IDE Channel 2
                2 | 3 => IDEPorts::new(0x170, 0x374),
                _ => unreachable!(),
            },
        )
    }
}

impl IDE {
    pub fn init(&mut self) -> Result<(), IDEStatus> {
        unsafe {
            self.ports
                .drive_head
                .write(0xA0 | ((self.is_slave as u8) << 4));
            self.ports.sector_count.write(0);
            self.ports.sector_number.write(0);
            self.ports.cylinder_low.write(0);
            self.ports.cylinder_high.write(0);
            self.ports.command.write(ATACommand::Identify as u8);

            while IDEStatus::from_bits_truncate(self.ports.status.read()).contains(IDEStatus::BSY) {
            }

            // Read 256 16-bit values, and store them.
            for _ in 0..256 {
                self.ports.data.read();
            }
        }

        // TODO: check for errors
        Ok(())
    }

    pub fn read_lba(&mut self, sector: u32, count: u8, target: &mut [u8]) -> Result<(), IDEStatus> {
        assert_eq!(
            (count as usize) * 512,
            target.len(),
            "target length {} should equal {}",
            target.len(),
            count as usize * 512
        );

        unsafe {
            self.ports
                .drive_head
                .write(0xE0 | ((self.is_slave as u8) << 4) | (((sector >> 24) as u8) & 0x0F));
            self.ports.features.write(0x00);
            self.ports.sector_count.write(count);
            self.ports.sector_number.write((sector & 0xFF) as u8);
            self.ports.cylinder_low.write(((sector >> 8) & 0xFF) as u8);
            self.ports
                .cylinder_high
                .write(((sector >> 16) & 0xFF) as u8);
            self.ports.command.write(ATACommand::ReadSectors as u8);

            for i in 0..count {
                loop {
                    let status = IDEStatus::from_bits_truncate(self.ports.status.read());
                    if !status.contains(IDEStatus::BSY) && status.contains(IDEStatus::DRQ) {
                        break;
                    }
                    if status.contains(IDEStatus::ERR) || status.contains(IDEStatus::DF) {
                        return Err(status);
                    }
                }

                // This line is required to read properly
                // TODO: find out why
                debug!("read offset = {}", i as usize * 512usize);

                // Transfer 256 16-bit values, a uint16_t at a time, into your buffer from I/O port 0x1F0.
                // (In assembler, REP INSW works well for this.)
                core::arch::asm!("rep insw",
                    in("dx") self.ports.io_base,
                    in("rdi") &target[i as usize * 512usize ],
                    in("cx") 256usize,
                    lateout("rdi") _
                );
            }
        }

        // TODO: check for errors
        Ok(())
    }
}

pub struct IDEPorts {
    pub io_base: u16,
    pub ctrl_base: u16,
    pub data: Port<u16>,
    pub error: Port<u8>,
    pub features: Port<u8>,
    pub sector_count: Port<u8>,
    pub sector_number: Port<u8>,
    pub cylinder_low: Port<u8>,
    pub cylinder_high: Port<u8>,
    pub drive_head: Port<u8>,
    pub status: Port<u8>,
    pub command: Port<u8>,
}

impl IDEPorts {
    pub fn new(io_base: u16, ctrl_base: u16) -> Self {
        Self {
            io_base,
            ctrl_base,
            data: Port::<u16>::new(io_base),
            error: Port::<u8>::new(io_base + 1),
            features: Port::<u8>::new(io_base + 1),
            sector_count: Port::<u8>::new(io_base + 2),
            sector_number: Port::<u8>::new(io_base + 3),
            cylinder_low: Port::<u8>::new(io_base + 4),
            cylinder_high: Port::<u8>::new(io_base + 5),
            drive_head: Port::<u8>::new(io_base + 6),
            status: Port::<u8>::new(io_base + 7),
            command: Port::<u8>::new(io_base + 7),
        }
    }
}

#[repr(u8)]
pub enum ATACommand {
    NOP = 0x00,
    Identify = 0xEC,
    ReadSectors = 0x20,
    WriteSectors = 0x30,
}

bitflags! {
    pub struct IDEStatus: u8 {
        #[doc = "Indicates an error occurred. Send a new command to clear it (or nuke it with a Software Reset)."]
        const ERR = 0x01;
        #[doc = "Index. Always set to zero."]
        const IDX = 0x02;
        #[doc = "Corrected data. Always set to zero."]
        const CORR = 0x04;
        #[doc = "Set when the drive has PIO data to transfer, or is ready to accept PIO data."]
        const DRQ = 0x08;
        #[doc = "Overlapped Mode Service Request."]
        const SRV = 0x10;
        #[doc = "Drive Fault Error (does not set ERR)."]
        const DF = 0x20;
        #[doc = "Bit is clear when drive is spun down, or after an error. Set otherwise."]
        const RDY = 0x40;
        #[doc = "Indicates the drive is preparing to send/receive data (wait for it to clear). In case of 'hang' (it never clears), do a software reset."]
        const BSY = 0x80;
    }
}
