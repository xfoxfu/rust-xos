use apic::*;
use x86_64::structures::idt::InterruptDescriptorTable;

mod apic;
mod consts;
mod handlers;
mod keyboard;
mod syscall;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        handlers::reg_idt(&mut idt);
        keyboard::reg_idt(&mut idt);
        idt
    };
}

/// 初始化中断及硬件系统
pub unsafe fn init() {
    IDT.load();
    keyboard::init();
    info!("xapic support = {}", apic::XApic::support());
    // info!("x2apic support = {}", apic::X2Apic::support());
    let mut xapic = unsafe { XApic::new(crate::memory::physical_to_virtual(0xfee00000)) };
    xapic.cpu_init();

    x86_64::instructions::interrupts::enable();
}

#[inline(always)]
pub fn enable_irq(irq: u8) {
    let mut ioapic =
        unsafe { IoApic::new(crate::memory::physical_to_virtual(IOAPIC_ADDR as usize)) };
    ioapic.enable(irq, 0);
}

#[inline(always)]
pub fn ack(_irq: u8) {
    let mut lapic = unsafe { XApic::new(crate::memory::physical_to_virtual(LAPIC_ADDR)) };
    lapic.eoi();
}
