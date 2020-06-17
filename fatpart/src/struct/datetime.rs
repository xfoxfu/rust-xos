use bit_field::BitField;

#[derive(Eq, PartialEq)]
pub struct FatDate {
    pub year: u8,
    pub month: u8,
    pub day: u8,
}

impl FatDate {
    pub fn new(year: u8, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    pub fn parse_u16(value: u16) -> Self {
        let year = value.get_bits(10..16);
        let month = value.get_bits(5..10);
        let day = value.get_bits(0..5);
        Self::new(year as u8, month as u8, day as u8)
    }
}

impl core::fmt::Display for FatDate {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:02}-{:02}-{:02}", self.year, self.month, self.day)
    }
}
impl core::fmt::Debug for FatDate {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:02}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[derive(Eq, PartialEq)]
pub struct FatTime {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl FatTime {
    pub fn new(hour: u8, minute: u8, second: u8) -> Self {
        Self {
            hour,
            minute,
            second,
        }
    }

    pub fn parse_u16(value: u16) -> Self {
        let hour = value.get_bits(11..16);
        let minute = value.get_bits(5..11);
        let second = value.get_bits(0..5) as u8 * 2u8;
        Self::new(hour as u8, minute as u8, second as u8)
    }
}

impl core::fmt::Display for FatTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }
}
impl core::fmt::Debug for FatTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }
}
