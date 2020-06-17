#[derive(Debug, Eq, PartialEq)]
pub struct MBRPartitionTable {
    pub partition0: PartitionMeta,
    pub partition1: PartitionMeta,
    pub partition2: PartitionMeta,
    pub partition3: PartitionMeta,
}

impl MBRPartitionTable {
    pub fn parse(data: &[u8]) -> Result<Self, usize> {
        if data.len() != 16 * 4 {
            Err(data.len())?
        }

        let partition0 = PartitionMeta::parse(&data[0..16])?;
        let partition1 = PartitionMeta::parse(&data[16..32])?;
        let partition2 = PartitionMeta::parse(&data[32..48])?;
        let partition3 = PartitionMeta::parse(&data[48..64])?;

        Ok(Self {
            partition0,
            partition1,
            partition2,
            partition3,
        })
    }

    pub fn parse_sector(data: &[u8]) -> Result<Self, usize> {
        if data.len() != 512 {
            Err(data.len())?
        }

        Ok(Self::parse(&data[0x1BE..0x1FE])?)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct PartitionMeta {
    pub is_active: bool,
    pub begin_head: u8,
    pub begin_sector: u8,
    pub begin_cylinder: u16,
    pub fs: u8,
    pub end_head: u8,
    pub end_sector: u8,
    pub end_cylinder: u16,
    pub begin_lba: u32,
    pub total_lba: u32,
}

impl PartitionMeta {
    pub fn parse(data: &[u8]) -> Result<Self, usize> {
        if data.len() != 16 {
            return Err(data.len());
        }

        // 00H 1 分区状态：00-->非活动分区；0x80-->活动分区；
        //       其它数值没有意义
        let is_active = data[0x0] == 0x80;
        // 01H 1 分区起始磁头号（HEAD），用到全部8位
        let begin_head = data[0x1];
        // 02H 2 分区起始扇区号（SECTOR），占据02H的位0－5；
        //       该分区的起始磁柱号（CYLINDER），占据
        //       02H的位6－7和03H的全部8位
        let begin_sector = data[0x2] & 0x3F;
        let begin_cylinder = ((data[0x2] as u16 & 0xC0) << 2) | data[0x3] as u16;
        // 04H 1 文件系统标志位
        let fs = data[0x4];
        // 05H 1 分区结束磁头号（HEAD），用到全部8位
        let end_head = data[0x5];
        // 06H 2 分区结束扇区号（SECTOR），占据06H的位0－5；
        //       该分区的结束磁柱号（CYLINDER），占据
        //       06H的位6－7和07H的全部8位
        let end_sector = data[0x6] & 0x3F;
        let end_cylinder = ((data[0x6] as u16) << 2) | data[0x7] as u16;
        // 08H 4 分区起始相对扇区号
        let begin_lba = ((data[0xB] as u32) << 24)
            | ((data[0xA] as u32) << 16)
            | ((data[0x9] as u32) << 8)
            | (data[0x8] as u32);
        // 0CH 4 分区总的扇区数
        let total_lba = ((data[0xF] as u32) << 24)
            | ((data[0xE] as u32) << 16)
            | ((data[0xD] as u32) << 8)
            | (data[0xC] as u32);

        Ok(Self {
            is_active,
            begin_head,
            begin_sector,
            begin_cylinder,
            fs,
            end_head,
            end_sector,
            end_cylinder,
            begin_lba,
            total_lba,
        })
    }

    pub fn from_zero() -> Self {
        Self {
            is_active: false,
            begin_head: 0,
            begin_sector: 0,
            begin_cylinder: 0,
            fs: 0,
            end_head: 0,
            end_sector: 0,
            end_cylinder: 0,
            begin_lba: 0,
            total_lba: 0,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_meta() {
        let meta0 = PartitionMeta::parse(&[
            0x80, 0x01, 0x01, 0x00, 0x0B, 0xFE, 0xBF, 0xFC, 0x3F, 0x00, 0x00, 0x00, 0x7E, 0x86,
            0xBB, 0x00,
        ])
        .unwrap();

        assert_eq!(
            meta0,
            PartitionMeta {
                is_active: true,
                begin_head: 1,
                begin_sector: 1,
                begin_cylinder: 0,
                fs: 0x0B,
                end_head: 254,
                end_sector: 63,
                end_cylinder: 764,
                begin_lba: 63,
                total_lba: 12289662,
            }
        );
    }

    #[test]
    fn mbr_part() {
        let mut part = vec![
            0x80, 0x01, 0x01, 0x00, 0x0B, 0xFE, 0xBF, 0xFC, 0x3F, 0x00, 0x00, 0x00, 0x7E, 0x86,
            0xBB, 0x00,
        ];
        part.resize(64, 0);
        let meta0 = MBRPartitionTable::parse(&part).unwrap();

        assert_eq!(
            meta0,
            MBRPartitionTable {
                partition0: PartitionMeta {
                    is_active: true,
                    begin_head: 1,
                    begin_sector: 1,
                    begin_cylinder: 0,
                    fs: 0x0B,
                    end_head: 254,
                    end_sector: 63,
                    end_cylinder: 764,
                    begin_lba: 63,
                    total_lba: 12289662,
                },
                partition1: PartitionMeta::from_zero(),
                partition2: PartitionMeta::from_zero(),
                partition3: PartitionMeta::from_zero(),
            }
        );
    }

    #[test]
    fn mbr_sector() {
        let mut part = [0u8; 512];
        part[0x1BE] = 0x80;
        part[0x1BF] = 0x01;
        part[0x1C0] = 0x01;
        part[0x1C1] = 0x00;
        part[0x1C2] = 0x0B;
        part[0x1C3] = 0xFE;
        part[0x1C4] = 0xBF;
        part[0x1C5] = 0xFC;
        part[0x1C6] = 0x3F;
        part[0x1C7] = 0x00;
        part[0x1C8] = 0x00;
        part[0x1C9] = 0x00;
        part[0x1CA] = 0x7E;
        part[0x1CB] = 0x86;
        part[0x1CC] = 0xBB;
        part[0x1CD] = 0x00;
        let meta0 = MBRPartitionTable::parse_sector(&part).unwrap();

        assert_eq!(
            meta0,
            MBRPartitionTable {
                partition0: PartitionMeta {
                    is_active: true,
                    begin_head: 1,
                    begin_sector: 1,
                    begin_cylinder: 0,
                    fs: 0x0B,
                    end_head: 254,
                    end_sector: 63,
                    end_cylinder: 764,
                    begin_lba: 63,
                    total_lba: 12289662,
                },
                partition1: PartitionMeta::from_zero(),
                partition2: PartitionMeta::from_zero(),
                partition3: PartitionMeta::from_zero(),
            }
        );
    }
}
