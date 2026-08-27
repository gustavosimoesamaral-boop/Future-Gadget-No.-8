use pic8259::ChainedPics;
use spin::{Mutex, Once};
use x86_64::structures::idt::{
    InterruptDescriptorTable,
    InterruptStackFrame,
};
use core::sync::atomic::{
    AtomicU8,
    AtomicU64,
    AtomicUsize,
    Ordering,
};
use crate::keyboard;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub const TIMER_INTERRUPT_ID: u8 = PIC_1_OFFSET;
pub const KEYBOARD_INTERRUPT_ID: u8 = PIC_1_OFFSET + 1;

static IDT: Once<InterruptDescriptorTable> = Once::new();

static PICS: Mutex<ChainedPics> = Mutex::new(unsafe {
    ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
});

static TIMER_TICKS: AtomicU64 = AtomicU64::new(0);
const KEYBOARD_BUFFER_SIZE: usize = 128;

static KEYBOARD_BUFFER: [AtomicU8; KEYBOARD_BUFFER_SIZE] =
    [const { AtomicU8::new(0) }; KEYBOARD_BUFFER_SIZE];

static KEYBOARD_HEAD: AtomicUsize = AtomicUsize::new(0);
static KEYBOARD_TAIL: AtomicUsize = AtomicUsize::new(0);

pub fn init() {
    IDT.call_once(|| {
        let mut idt = InterruptDescriptorTable::new();

        idt.breakpoint.set_handler_fn(breakpoint_handler);

        idt[TIMER_INTERRUPT_ID]
            .set_handler_fn(timer_interrupt_handler);

        idt[KEYBOARD_INTERRUPT_ID]
            .set_handler_fn(keyboard_interrupt_handler);

        idt
    });

    unsafe {
        PICS.lock().initialize();

        // Libera somente IRQ0 (timer).
        // Todas as outras IRQs continuam mascaradas.
        PICS.lock().write_masks(0b1111_1100, 0xFF);
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

pub fn keyboard_scancode() -> Option<u8> {
    let tail = KEYBOARD_TAIL.load(Ordering::Relaxed);
    let head = KEYBOARD_HEAD.load(Ordering::Acquire);

    if tail == head {
        return None;
    }

    let scancode = KEYBOARD_BUFFER[tail].load(Ordering::Relaxed);

    let next_tail = (tail + 1) % KEYBOARD_BUFFER_SIZE;

    KEYBOARD_TAIL.store(next_tail, Ordering::Release);

    Some(scancode)
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

extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: InterruptStackFrame,
) {
    let scancode = keyboard::read_hardware_scancode();

    push_keyboard_scancode(scancode);

    unsafe {
        PICS.lock().notify_end_of_interrupt(KEYBOARD_INTERRUPT_ID);
    }
}

fn push_keyboard_scancode(scancode: u8) {
    let head = KEYBOARD_HEAD.load(Ordering::Relaxed);
    let next_head = (head + 1) % KEYBOARD_BUFFER_SIZE;

    let tail = KEYBOARD_TAIL.load(Ordering::Acquire);

    // Buffer cheio: descarta o novo scancode.
    if next_head == tail {
        return;
    }

    KEYBOARD_BUFFER[head].store(scancode, Ordering::Relaxed);

    KEYBOARD_HEAD.store(next_head, Ordering::Release);
}