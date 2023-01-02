#![no_std]
#![no_main]

use bootloader_api::{entry_point, BootInfo};
use kernel::serial_println;
use core::panic::PanicInfo;

entry_point!(kernel_main);

#[allow(unconditional_panic)]
fn kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    serial_println!("\n\nWelcome to Micfong OS!");
    let zero = 0;
    serial_println!("\n\nWelcome to Micfong OS!{}", 1/zero);
    loop {}
}

/// This function is called when Rust panics.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[PANIC] {}", info);
    loop {}
}
