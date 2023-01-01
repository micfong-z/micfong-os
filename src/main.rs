#![no_std]
#![no_main]

mod serial;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_println!("\n\nWelcome to Micfong OS!");
    let zero: i32 = 0;
    serial_println!("Hello World{} {}", "!", 0/zero);
    loop {}
}

use core::panic::PanicInfo;

/// This function is called when Rust panics.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[PANIC] {}", info);
    loop {}
}
