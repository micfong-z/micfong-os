#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)] // at the top of the file
#![feature(once_cell)]

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("alloc error: {:?}", layout)
}

extern crate alloc;

use x86_64::instructions;
pub mod allocator;
pub mod bitmap;
pub mod colors;
pub mod gdt;
pub mod graphics;
pub mod interrupts;
pub mod keyboard;
pub mod layer;
pub mod log;
pub mod memory;
pub mod serial;
pub mod unifont;
pub mod gui;

pub fn hlt_loop() -> ! {
    loop {
        instructions::hlt();
    }
}
