use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;

use crate::{log_warn, colors};

pub static KEYBOARD_SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

pub fn add_scancode(scancode: u8) {
    if let Ok(queue) = KEYBOARD_SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            log_warn!("Keyboard scancode queue full; dropping keyboard input {:#X}", scancode);
        }
    } else {
        log_warn!("Keyboard scancode queue uninitialized");
    }
}

pub fn get_scancode() -> Option<u8> {
    KEYBOARD_SCANCODE_QUEUE
        .try_get()
        .ok()
        .and_then(|queue| queue.pop().ok())
}

pub fn init() {
    KEYBOARD_SCANCODE_QUEUE
        .try_init_once(|| ArrayQueue::new(64))
        .expect("Keyboard scancode queue already initialized");
}
