#![no_std]
#![no_main]

use bootloader_api::config::{BootloaderConfig, Mapping};
use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;
use kernel::{hlt_loop, log, log_info, log_panic, serial_println};

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    kernel::init(&mut boot_info.framebuffer);

    log_info!("Welcome to Micfong OS!");
    serial_println!("\n\nWelcome to Micfong OS!");
    
    hlt_loop();
}

/// This function is called when Rust panics.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if log::is_initialized() {
        log_panic!("{}", info);
    }
    serial_println!("[PANIC] {}", info);

    loop {}
}
