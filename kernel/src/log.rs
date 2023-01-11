use conquer_once::spin::OnceCell;
use spin::Mutex;

use crate::graphics;

pub static LOGGER: OnceCell<LockedLogger> = OnceCell::uninit();

pub struct LockedLogger(Mutex<Logger>);

impl LockedLogger {
    pub fn new(line_height: usize, margin: usize) -> Self {
        LockedLogger(Mutex::new(Logger::new(line_height, margin)))
    }

    /// Force-unlocks the logger to prevent deadlocks.
    /// 
    /// ## Safety
    /// This method is unsafe and usage of it should be avoided.
    pub unsafe fn force_unlock(&self) {
        unsafe { self.0.force_unlock() };
    }
}


struct Logger {
    line_height: usize,
    margin: usize,
    x: usize,
    y: usize,
    color: u32,
}

impl Logger {
    pub fn new(line_height: usize, margin: usize) -> Self {
        Logger {
            line_height,
            margin,
            x: margin,
            y: margin,
            color: 0xFFFFFF,
        }
    }

    fn print(&mut self, s: &str) {
        for c in s.chars() {
            let newx = self.x + graphics::get_char_width(c);
            if newx > graphics::get_width() - self.margin {
                self.x = self.margin;
                self.y += self.line_height;
            }
            if c == '\n' {
                self.new_line();
            } else {
                self.x += graphics::draw_char(self.x, self.y, c, self.color);
            }
        }
    }

    fn new_line(&mut self) {
        self.x = self.margin;
        self.y += self.line_height;
    }

    fn set_color(&mut self, color: u32) {
        self.color = color;
    }
}

pub fn logger_init(line_height: usize, margin: usize) {
    LOGGER.init_once(|| LockedLogger::new(line_height, margin));
}

pub fn is_initialized() -> bool {
    LOGGER.is_initialized()
}

use core::fmt::{self, Write};
impl fmt::Write for Logger {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.print(s);
        Ok(())
    }
}

pub fn set_color(color: u32) {
    let mut logger = LOGGER.get().unwrap().0.lock();
    logger.set_color(color);
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    LOGGER.get().unwrap().0.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::log::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! log_trace {
    () => {
        $crate::log::set_color(0xAAAAAA);
        $crate::print!("[TRACE:]\n");
    };
    ($($arg:tt)*) => {
        $crate::log::set_color(0xAAAAAA);
        $crate::print!("[TRACE:] {}\n", format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_info {
    () => {
        $crate::log::set_color(0xFFFFFF);
        $crate::print!("[ INFO ]\n");
    };
    ($($arg:tt)*) => {
        $crate::log::set_color(0xFFFFFF);
        $crate::print!("[ INFO ] ");
        $crate::print!("{}\n", format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_warn {
    () => {
        $crate::log::set_color(0xFCBB13);
        $crate::print!("[ WARN ]\n");
    };
    ($($arg:tt)*) => {
        $crate::log::set_color(0xFCBB13);
        $crate::print!("[ WARN ] ");
        $crate::log::set_color(0xFDDD89);
        $crate::print!("{}\n", format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_error {
    () => {
        $crate::log::set_color(0xFA4B4B);
        $crate::print!("[ERROR!]\n");
    };
    ($($arg:tt)*) => {
        $crate::log::set_color(0xFA4B4B);
        $crate::print!("[ERROR!] ");
        $crate::log::set_color(0xFCA5A5);
        $crate::print!("{}\n", format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_panic {
    () => {
        $crate::log::set_color(0xFA4B4B);
        $crate::print!("[PANIC!]\n");
    };
    ($($arg:tt)*) => {
        $crate::log::set_color(0xFA4B4B);
        $crate::print!("[PANIC!] {}\n", format_args!($($arg)*));
    };
}
