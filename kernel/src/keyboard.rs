use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use spin::Mutex;
use x86_64::instructions::port::Port;

use crate::{colors, graphics, log_warn};

#[derive(Debug, PartialEq, Eq)]
pub enum MousePhase {
    Ack,
    Byte1,
    Byte2,
    Byte3,
}

pub struct MouseStatus {
    pub x_delta: i32,
    pub y_delta: i32,
    pub x_pos: i32,
    pub y_pos: i32,
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,
    pub buffer: [u8; 3],
    pub phase: MousePhase,
}

pub static KEYBOARD_SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
pub static MOUSE_SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
pub static MOUSE_STATUS: OnceCell<Mutex<MouseStatus>> = OnceCell::uninit();

pub fn add_keyboard_scancode(scancode: u8) {
    if let Ok(queue) = KEYBOARD_SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            log_warn!(
                "Keyboard scancode queue full; dropping keyboard input {:#X}",
                scancode
            );
        }
    } else {
        log_warn!("Keyboard scancode queue uninitialized");
    }
}

pub fn scancode_queues_empty() -> bool {
    KEYBOARD_SCANCODE_QUEUE
        .try_get()
        .ok()
        .map(|queue| queue.is_empty())
        .unwrap_or(true)
        && MOUSE_SCANCODE_QUEUE
            .try_get()
            .ok()
            .map(|queue| queue.is_empty())
            .unwrap_or(true)
}

pub fn get_keyboard_scancode() -> Option<u8> {
    KEYBOARD_SCANCODE_QUEUE
        .try_get()
        .ok()
        .and_then(|queue| queue.pop().ok())
}

pub fn add_mouse_scancode(scancode: u8) {
    if let Ok(queue) = MOUSE_SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            log_warn!(
                "Mouse scancode queue full; dropping mouse input {:#X}",
                scancode
            );
        }
    } else {
        log_warn!("Mouse scancode queue uninitialized");
    }
}

pub fn get_mouse_scancode() -> Option<u8> {
    MOUSE_SCANCODE_QUEUE
        .try_get()
        .ok()
        .and_then(|queue| queue.pop().ok())
}

pub fn init() {
    KEYBOARD_SCANCODE_QUEUE
        .try_init_once(|| ArrayQueue::new(64))
        .expect("Keyboard scancode queue already initialized");
    MOUSE_SCANCODE_QUEUE
        .try_init_once(|| ArrayQueue::new(128))
        .expect("Mouse scancode queue already initialized");
    MOUSE_STATUS
        .try_init_once(|| {
            Mutex::new(MouseStatus {
                x_delta: 0,
                y_delta: 0,
                x_pos: (graphics::get_width() / 2) as i32,
                y_pos: (graphics::get_height() / 2) as i32,
                left_button: false,
                right_button: false,
                middle_button: false,
                buffer: [0; 3],
                phase: MousePhase::Ack,
            })
        })
        .expect("Mouse status already initialized");
}

const PORT_KEYDAT: u16 = 0x0060;
const PORT_KEYSTA: u16 = 0x0064;
const PORT_KEYCMD: u16 = 0x0064;
const KEYCMD_WRITE_MODE: u8 = 0x60;
const KEYCMD_SENDTO_MOUSE: u8 = 0xd4;
const MOUSECMD_ENABLE: u8 = 0xf4;

const KBC_MODE: u8 = 0x47; // mode enabling PS/2 mouse

fn wait_kbc_isready() {
    let mut port = Port::new(PORT_KEYSTA);
    loop {
        let status: u8 = unsafe { port.read() };
        if (status & 0b10) == 0 {
            break;
        }
    }
}

pub fn init_kbc() {
    let mut port = Port::new(PORT_KEYCMD);
    unsafe {
        port.write(KEYCMD_WRITE_MODE);
    }
    wait_kbc_isready();
    let mut port = Port::new(PORT_KEYDAT);
    unsafe {
        port.write(KBC_MODE);
    }
    wait_kbc_isready();
}

pub fn enable_mouse() {
    wait_kbc_isready();
    let mut port = Port::new(PORT_KEYCMD);
    unsafe { port.write(KEYCMD_SENDTO_MOUSE) };
    let mut port = Port::new(PORT_KEYDAT);
    unsafe { port.write(MOUSECMD_ENABLE) };
}
