#![no_std]
#![no_main]

use bootloader_api::config::{BootloaderConfig, Mapping};
use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;
use kernel::{log, log_trace};
use kernel::serial_println;
use kernel::{log_info, log_panic};

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    unsafe {
    kernel::init(boot_info);
    }
    
    log_info!("Welcome to Micfong OS!");
    serial_println!("\n\nWelcome to Micfong OS!");
    unsafe {
        let start = x86::time::rdtsc();
        for i in 0..1000 {
            log_info!("It {}", i);
        }
        let end = x86::time::rdtsc();
        log_trace!("Cycles: {} M", (end - start) / 1000000);
    }

    use x86_64::registers::control::Cr3;

    let (level_4_page_table, _) = Cr3::read();
    log_trace!("Current active level 4 page table @ {:?}", level_4_page_table.start_address());

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
