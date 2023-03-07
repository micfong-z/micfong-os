#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;

use bootloader_api::{
    config::{BootloaderConfig, Mapping},
    entry_point, BootInfo,
};
use kernel::{
    allocator, gdt, graphics, hlt_loop, interrupts, log, log_info, log_panic, log_trace,
    memory::{self, BootInfoFrameAllocator},
    println, serial_println, log_ok,
};
use x86_64::VirtAddr;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    {
        // Initialization of the kernel

        gdt::init();
        graphics::painter_init(&mut boot_info.framebuffer);
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

        let phys_mem_offset =
            VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
        let mut mapper = unsafe { memory::init_mapper(phys_mem_offset) };
        log_info!("Memory mapper initialized");
        let mut frame_allocator =
            unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };
        log_info!("Frame allocator initialized");

        allocator::init_heap(&mut mapper, &mut frame_allocator)
            .expect("heap initialization failed");
        log_info!("Heap initialized");
        log_ok!("Kernel initialization done");
    }

    println!();
    log_ok!("Welcome to Micfong OS!");
    serial_println!("\n\nWelcome to Micfong OS!");

    hlt_loop();
}

/// This function is called when Rust panics.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if log::is_initialized() {
        log_panic!("{}", info);
    }
    serial_println!("[PANIC!] {}", info);

    loop {}
}
