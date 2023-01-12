use crate::{log_warn, serial_println, log_panic, gdt};
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX); // new
        }
        idt
    };
}

pub fn idt_init() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    log_warn!("CPU Exception: BREAKPOINT (int 0x3)\n{:#?}", stack_frame);
    serial_println!("CPU Exception: #BP BREAKPOINT (int 0x3)\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    log_panic!("CPU Exception: #DF DOUBLE FAULT (int 0x8)\n{:#?}", stack_frame);
    serial_println!("CPU Exception: #DF DOUBLE FAULT (int 0x8)\n{:#?}", stack_frame);
    panic!("double fault occured")
}
