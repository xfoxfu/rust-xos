use apic::*;
use x86_64::structures::idt::InterruptDescriptorTable;

mod consts;
mod handlers;
mod keyboard;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        handlers::reg_idt(&mut idt);
        keyboard::reg_idt(&mut idt);
        idt
    };
}

pub fn init() {
    IDT.load();
    keyboard::init();
    x86_64::instructions::interrupts::enable();
}

#[inline(always)]
pub fn enable_irq(irq: u8) {
    let mut ioapic = unsafe { IoApic::new(IOAPIC_ADDR as usize + 0xFFFF800000000000) };
    ioapic.enable(irq, 0);
}

#[inline(always)]
pub fn ack(_irq: u8) {
    let mut lapic = unsafe { XApic::new(LAPIC_ADDR + 0xFFFF800000000000) };
    lapic.eoi();
}
