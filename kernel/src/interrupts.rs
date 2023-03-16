use conquer_once::spin::Lazy;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::{colors, gdt, keyboard, log_error, log_panic, log_warn, serial_println};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    PIC2,
    SerialPort2,
    SerialPort1,
    ParallelPort2Or3,
    FloppyDisk,
    ParallelPort1,
    RTC,
    ACPI,
    Reserved1,
    Reserved2,
    Mouse,
    FPU,
    PrimaryATAHardDisk,
    SecondaryATAHardDisk,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    }
    idt.page_fault.set_handler_fn(page_fault_handler);
    idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
    idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
    idt[InterruptIndex::Mouse.as_usize()].set_handler_fn(mouse_interrupt_handler);
    idt
});

pub fn idt_init() {
    IDT.load();
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // way too noisy
    // log_trace!("Timer interrupt");
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    keyboard::add_keyboard_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn mouse_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    keyboard::add_mouse_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Mouse.as_u8());
    }
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

use x86_64::structures::idt::PageFaultErrorCode;

use crate::hlt_loop;

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
