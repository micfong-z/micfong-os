#![no_std]
#![no_main]

use bootloader_api::{entry_point, BootInfo};
use bootloader_api::config::{BootloaderConfig, Mapping};
use kernel::graphics;
use kernel::serial_println;
use core::panic::PanicInfo;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    serial_println!("\n\nWelcome to Micfong OS!");
    graphics::painter_init(&mut boot_info.framebuffer);
 
    let screen_height = graphics::get_height();
    let screen_width = graphics::get_width();

    graphics::draw_rect(0, 0, screen_width, screen_height, 0x202020);
    graphics::draw_str(10, 10, "Hello, world!", 0xFFFFFF);
    graphics::draw_str(10, 30, "你好, 世界!", 0xFFFFFF);
    loop {}
}

/// This function is called when Rust panics.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[PANIC] {}", info);
    loop {}
}
