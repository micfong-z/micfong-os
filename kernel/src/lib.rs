#![no_std]
#![feature(abi_x86_interrupt)]

use bootloader_api::BootInfo;
pub mod serial;
pub mod graphics;
pub mod unifont;
pub mod log;
pub mod interrupts;
pub mod gdt;

pub fn init(boot_info: &'static mut BootInfo) {
    gdt::init();
    interrupts::idt_init();
    graphics::painter_init(&mut boot_info.framebuffer);
    let screen_width = graphics::get_width();
    let screen_height = graphics::get_height();
    graphics::draw_rect(0, 0, screen_width, screen_height, 0x202020);
    log::logger_init(20, 4);
    log_info!("Initialization completed");
}

