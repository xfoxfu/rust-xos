use boot::RuntimeServices;
use spin::Mutex;
use xklibr::UefiClock;

once_mutex!(pub UEFI_CLOCK: UefiClock);

pub fn initialize(st: &'static RuntimeServices) {
    init_UEFI_CLOCK(UefiClock::new(st))
}

guard_access_fn! {
    #[doc = "获得UEFI时钟"]
    pub get_clock(UEFI_CLOCK: UefiClock)
}
