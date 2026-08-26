use spin::Once;
use x86_64::structures::idt::{
    InterruptDescriptorTable,
    InterruptStackFrame,
};

static IDT: Once<InterruptDescriptorTable> = Once::new();

pub fn init() {
    IDT.call_once(|| {
        let mut idt = InterruptDescriptorTable::new();

        idt.breakpoint.set_handler_fn(breakpoint_handler);

        idt
    });

    IDT.get().unwrap().load();
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame,
) {
    let _ = stack_frame;

    loop {
        x86_64::instructions::hlt();
    }
}