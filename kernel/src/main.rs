#![no_std]
#![no_main]

extern crate alloc;

use core::{panic::PanicInfo, u32::MAX};

use bootloader_api::{
    config::{BootloaderConfig, Mapping},
    entry_point, BootInfo,
};
use kernel::{
    allocator, bitmap, colors, gdt, graphics, interrupts,
    keyboard::{self, MousePhase, MOUSE_STATUS},
    layer::{self, Layer, LAYER_CONTROLLER},
    log, log_info, log_ok, log_panic, log_trace,
    memory::{self, BootInfoFrameAllocator},
    print, serial_println
};
use x86_64::{instructions, VirtAddr};

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
        graphics::draw_rect(
            0,
            0,
            screen_width,
            screen_height,
            colors::DESKTOP_BACKGROUND,
        );
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

        unsafe { interrupts::PICS.lock().initialize() };
        log_info!("PICs initialized");
        x86_64::instructions::interrupts::enable();
        log_info!("Interrupts enabled");

        let phys_mem_offset =
            VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
        let mut mapper = unsafe { memory::init_mapper(phys_mem_offset) };
        log_info!("Memory mapper initialized");
        let mut frame_allocator =
            unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };
        log_info!("Frame allocator initialized");

        log_info!("Initializing heap...");
        allocator::init_heap(&mut mapper, &mut frame_allocator)
            .expect("heap initialization failed");
        log_info!("Heap initialized");

        keyboard::init();
        keyboard::init_kbc();
        log_info!("Keyboard initialized");

        keyboard::enable_mouse();
        log_info!("Mouse initialized");

        layer::init();
        log_info!("Layer manager initialized");

        log_ok!("Kernel initialization done");
    }

    log_ok!("Welcome to Micfong OS!");
    serial_println!("\n\nWelcome to Micfong OS!");

    let screen_width = graphics::get_width();
    let screen_height = graphics::get_height();

    let mut background_layer = Layer::new(screen_width, screen_height, 0, 0, 0);
    background_layer.draw_rect(
        0,
        0,
        screen_width,
        screen_height,
        colors::DESKTOP_BACKGROUND,
    );
    background_layer.draw_rect(20, 20, 120, 120, colors::ORANGE);

    let mut mouse_cursor_layer = Layer::new(13, 19, screen_width / 2, screen_height / 2, MAX);
    mouse_cursor_layer.draw_bitmap(0, 0, 13, 19, &bitmap::MOUSE_CURSOR);

    let mut test_window_layer = Layer::new(200, 100, 120, 80, 1);
    test_window_layer.draw_window("Test Window");

    layer::add_layer(background_layer);
    layer::add_layer(test_window_layer);
    let mouse_cursor_layer = layer::add_layer(mouse_cursor_layer);

    let layer_controller = LAYER_CONTROLLER.get().unwrap().lock();

    layer_controller.render();

    loop {
        instructions::interrupts::disable();
        if keyboard::scancode_queues_empty() {
            instructions::interrupts::enable_and_hlt();
        } else {
            if let Some(scancode) = keyboard::get_keyboard_scancode() {
                instructions::interrupts::enable();
                log::set_color(colors::YELLOW);
                print!("{:02X} ", scancode);
            }
            if let Some(scancode) = keyboard::get_mouse_scancode() {
                instructions::interrupts::enable();

                let mut mouse_status = MOUSE_STATUS.get().unwrap().lock();
                match mouse_status.phase {
                    MousePhase::Ack => {
                        if scancode == 0xfa {
                            mouse_status.phase = MousePhase::Byte1;
                        }
                    }
                    MousePhase::Byte1 => {
                        // Check if this is a valid byte
                        if scancode & 0b1100_1000 == 0b0000_1000 {
                            mouse_status.buffer[0] = scancode;
                            mouse_status.phase = MousePhase::Byte2;
                        }
                    }
                    MousePhase::Byte2 => {
                        mouse_status.buffer[1] = scancode;
                        mouse_status.phase = MousePhase::Byte3;
                    }
                    MousePhase::Byte3 => {
                        mouse_status.buffer[2] = scancode;
                        mouse_status.phase = MousePhase::Byte1;

                        mouse_status.left_button = mouse_status.buffer[0] & 0b0000_0001 != 0;
                        mouse_status.right_button = mouse_status.buffer[0] & 0b0000_0010 != 0;
                        mouse_status.middle_button = mouse_status.buffer[0] & 0b0000_0100 != 0;

                        mouse_status.x_delta = mouse_status.buffer[1] as i32;
                        mouse_status.y_delta = mouse_status.buffer[2] as i32;
                        if mouse_status.buffer[0] & 0b0001_0000 != 0 {
                            mouse_status.x_delta -= 256;
                        }
                        if mouse_status.buffer[0] & 0b0010_0000 != 0 {
                            mouse_status.y_delta -= 256;
                        }
                        mouse_status.y_delta = -mouse_status.y_delta;

                        let old_x = mouse_status.x_pos as u32;
                        let old_y = mouse_status.y_pos as u32;

                        mouse_status.x_pos += mouse_status.x_delta;
                        mouse_status.y_pos += mouse_status.y_delta;
                        mouse_status.x_pos = mouse_status
                            .x_pos
                            .max(0)
                            .min((graphics::get_width() - 1) as i32);
                        mouse_status.y_pos = mouse_status
                            .y_pos
                            .max(0)
                            .min((graphics::get_height() - 1) as i32);

                        mouse_cursor_layer
                            .lock()
                            .set_pos(mouse_status.x_pos as u32, mouse_status.y_pos as u32);
                        layer_controller.render_partial(old_x, old_y, 13, 19);
                        layer_controller.render_partial(
                            mouse_status.x_pos as u32,
                            mouse_status.y_pos as u32,
                            13,
                            19,
                        );
                    }
                }
            }
        }
    }
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
