#![no_std]
#![feature(abi_x86_interrupt)]

use bootloader_api::{info::FrameBuffer, info::Optional};
use x86_64::instructions;
pub mod gdt;
pub mod graphics;
pub mod interrupts;
pub mod log;
pub mod serial;
pub mod unifont;


pub fn init(framebuffer: &'static mut Optional<FrameBuffer>) {
    gdt::init();
    interrupts::idt_init();
    graphics::painter_init(framebuffer);
    let screen_width = graphics::get_width();
    let screen_height = graphics::get_height();
    graphics::draw_rect(0, 0, screen_width, screen_height, 0x202020);
    log::logger_init(20, 4);
    log_info!("Initialization completed");
}

pub fn hlt_loop() -> ! {
    loop {
        instructions::hlt();
    }
}
