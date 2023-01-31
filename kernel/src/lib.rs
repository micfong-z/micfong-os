#![no_std]
#![feature(abi_x86_interrupt)]

use bootloader_api::BootInfo;
use x86_64::instructions;
pub mod gdt;
pub mod graphics;
pub mod interrupts;
pub mod log;
pub mod serial;
pub mod unifont;

pub unsafe fn init(boot_info: &'static mut BootInfo) {
    gdt::init();
    interrupts::idt_init();
    graphics::painter_init(&mut boot_info.framebuffer);
    let screen_width = graphics::get_width();
    let screen_height = graphics::get_height();
    graphics::draw_rect(0, 0, screen_width, screen_height, 0x202020);
    log::logger_init(20, 4);
    log_info!("Initialization completed");
}

pub fn hlt_loop() {
    loop {
        instructions::hlt();
    }
}
