#![no_std]
#![feature(abi_x86_interrupt)]

use bootloader_api::info::{FrameBuffer, Optional};
use x86_64::instructions;
pub mod gdt;
pub mod graphics;
pub mod interrupts;
pub mod log;
pub mod serial;
pub mod unifont;

pub fn init(framebuffer: &'static mut Optional<FrameBuffer>) {
    gdt::init();
    graphics::painter_init(framebuffer);
    let screen_width = graphics::get_width();
    let screen_height = graphics::get_height();
    graphics::draw_rect(0, 0, screen_width, screen_height, 0x202020);
    log::logger_init(20, 4);
    log_info!("(done before logger init) GDT reloaded");
    log_info!("(done before logger init) Graphics initialized");
    log_trace!(
        "-   Screen width: {}
-   Screen height: {}",
        screen_width,
        screen_height
    );
    log_info!("Logger initialized");
    interrupts::idt_init();
    log_info!("IDT reloaded");
    log_info!("Initialization completed\n");
}

pub fn hlt_loop() -> ! {
    loop {
        instructions::hlt();
    }
}
