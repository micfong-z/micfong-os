#![no_std]
#![no_main]

use bootloader::{BootInfo, entry_point};
use micfong_os::serial_println;

entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    serial_println!("\n\nWelcome to Micfong OS!");
    loop {}
}

use core::panic::PanicInfo;

/// This function is called when Rust panics.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[PANIC] {}", info);
    loop {}
}
