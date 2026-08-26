use pic8259::ChainedPics;
use spin::{Mutex, Once};
use x86_64::structures::idt::{
    InterruptDescriptorTable,
    InterruptStackFrame,
};
use core::sync::atomic::{AtomicU64, Ordering};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub const TIMER_INTERRUPT_ID: u8 = PIC_1_OFFSET;

static IDT: Once<InterruptDescriptorTable> = Once::new();

static PICS: Mutex<ChainedPics> = Mutex::new(unsafe {
    ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
});

static TIMER_TICKS: AtomicU64 = AtomicU64::new(0);

pub fn init() {
    IDT.call_once(|| {
        let mut idt = InterruptDescriptorTable::new();

        idt.breakpoint.set_handler_fn(breakpoint_handler);

        idt[TIMER_INTERRUPT_ID]
            .set_handler_fn(timer_interrupt_handler);

        idt
    });

    unsafe {
        PICS.lock().initialize();

        // Libera somente IRQ0 (timer).
        // Todas as outras IRQs continuam mascaradas.
        PICS.lock().write_masks(0b1111_1110, 0xFF);
    }

    unsafe {
        IDT.get().unwrap().load();
    }
}

pub fn enable() {
    x86_64::instructions::interrupts::enable();
}

pub fn timer_ticks() -> u64 {
    TIMER_TICKS.load(Ordering::Relaxed)
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame,
) {
    let _ = stack_frame;

    loop {
        x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: InterruptStackFrame,
) {
    TIMER_TICKS.fetch_add(1, Ordering::Relaxed);

    unsafe {
        PICS.lock().notify_end_of_interrupt(TIMER_INTERRUPT_ID);
    }
}