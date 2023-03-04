use crate::{gdt, log_error, log_panic, log_warn, serial_println};
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
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt
    };
}

pub fn idt_init() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    log_warn!(
        "CPU Exception:    BREAKPOINT (int 0x3)

═╡ STACK FRAME ╞══════════════════════
{:#?}
══════════════════════════════════════",
        stack_frame
    );
    serial_println!(
        "CPU Exception:    BREAKPOINT (int 0x3)

═╡ STACK FRAME ╞══════════════════════
{:#?}
══════════════════════════════════════",
        stack_frame
    );
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    log_panic!(
        "CPU Exception:    #DF DOUBLE FAULT (int 0x8)

═╡ STACK FRAME ╞══════════════════════
{:#?}
══════════════════════════════════════",
        stack_frame
    );
    serial_println!(
        "CPU Exception:    #DF DOUBLE FAULT (int 0x8)

═╡ STACK FRAME ╞══════════════════════
{:#?}
══════════════════════════════════════",
        stack_frame
    );
    panic!("double fault occured")
}

use crate::hlt_loop;
use x86_64::structures::idt::PageFaultErrorCode;

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    log_error!(
        "CPU Exception:    #PF PAGE FAULT
Accessed Address: {:?}
Error Code:       {:?}

═╡ STACK FRAME ╞══════════════════════
{:#?}
══════════════════════════════════════",
        Cr2::read(),
        error_code,
        stack_frame
    );
    serial_println!(
        "CPU Exception:    #PF PAGE FAULT
Accessed Address: {:?}
Error Code:       {:?}

═╡ STACK FRAME ╞══════════════════════
{:#?}
══════════════════════════════════════",
        Cr2::read(),
        error_code,
        stack_frame
    );
    hlt_loop();
}
