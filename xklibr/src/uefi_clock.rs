use boot::RuntimeServices;
use chrono::naive::*;
use chrono::Duration;

pub struct UefiClock(&'static RuntimeServices);

impl UefiClock {
    pub fn new(st: &'static RuntimeServices) -> Self {
        UefiClock(st)
    }
}

impl UefiClock {
    fn rt(&self) -> &RuntimeServices {
        self.0
    }

    pub fn now(&self) -> NaiveDateTime {
        let uefi_time = self.rt().get_time().unwrap().unwrap();
        NaiveDate::from_ymd(
            uefi_time.year() as i32,
            uefi_time.month() as u32,
            uefi_time.day() as u32,
        )
        .and_hms_nano(
            uefi_time.hour() as u32,
            uefi_time.minute() as u32,
            uefi_time.second() as u32,
            uefi_time.nanosecond(),
        )
    }

    pub fn spin_wait_until(&self, time: &NaiveDateTime) {
        while &self.now() < time {}
    }

    pub fn spin_wait_for_ns(&self, ns: usize) {
        self.spin_wait_until(&(self.now() + Duration::nanoseconds(ns as i64)))
    }
}
