#![no_std]
#![no_main]

use bootloader_api::config::{BootloaderConfig, Mapping};
use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;
use kernel::log;
use kernel::serial_println;
use kernel::{graphics, log_info, log_panic, log_trace};

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

#[allow(unconditional_panic)]
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    serial_println!("\n\nWelcome to Micfong OS!");
    graphics::painter_init(&mut boot_info.framebuffer);
    let screen_height = graphics::get_height();
    let screen_width = graphics::get_width();
    graphics::draw_rect(0, 0, screen_width, screen_height, 0x202020);
    log::logger_init(20, 4);
    log_info!("Welcome to Micfong OS!");
    log_info!("Logger initialized");
    log_trace!("Screen width: {}", screen_width);
    log_trace!("Screen height: {}", screen_height);
    loop {}
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
