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
    indent: usize,
}

impl Logger {
    pub fn new(line_height: usize, margin: usize) -> Self {
        Logger {
            line_height,
            margin,
            x: margin,
            y: margin,
            color: 0xFFFFFF,
            indent: 0,
        }
    }

    fn print(&mut self, s: &str) {
        for c in s.chars() {
            let newx = self.x + graphics::get_char_width(c);
            if newx > graphics::get_width() - self.margin {
                self.new_line();
            }
            if c == '\n' {
                self.new_line();
            } else {
                self.x += graphics::draw_char(self.x, self.y, c, self.color);
            }
        }
    }

    fn new_line(&mut self) {
        self.x = self.margin + self.indent * 8;
        graphics::draw_rect(
            self.x,
            self.y + self.line_height,
            graphics::get_width() - self.margin * 2,
            self.line_height,
            0x202020,
        );
        let newy = self.y + self.line_height * 3;
        if newy > graphics::get_height() - self.margin {
            self.scroll_up();
        } else {
            self.y += self.line_height;
        }
    }

    fn scroll_up(&mut self) {
        // move everything up one line, and clear the last line
        graphics::move_all_up(self.line_height);
        // clear the last line
        graphics::draw_rect(
            self.margin,
            graphics::get_height() - self.margin - self.line_height,
            graphics::get_width() - self.margin * 2,
            self.line_height,
            0x202020,
        );
        // set the cursor to the last line

        // self.y = self.margin;
        // graphics::draw_rect(self.x, self.margin, graphics::get_width() - self.margin * 2, self.line_height, 0x202020);
    }

    fn set_color(&mut self, color: u32) {
        self.color = color;
    }

    fn set_indent(&mut self, indent: usize) {
        self.indent = indent;
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

pub fn set_indent(indent: usize) {
    let mut logger = LOGGER.get().unwrap().0.lock();
    logger.set_indent(indent);
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
        $crate::print!("[-TRACE]\n");
    };
    ($($arg:tt)*) => {
        $crate::log::set_color(0xAAAAAA);
        $crate::print!("[-TRACE] ");
        $crate::log::set_indent(9);
        $crate::print!("{}", format_args!($($arg)*));
        $crate::log::set_indent(0);
        $crate::print!("\n");
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
        $crate::log::set_indent(9);
        $crate::print!("{}", format_args!($($arg)*));
        $crate::log::set_indent(0);
        $crate::print!("\n");
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
        $crate::log::set_indent(9);
        $crate::print!("{}", format_args!($($arg)*));
        $crate::log::set_indent(0);
        $crate::print!("\n");
    };
}

#[macro_export]
macro_rules! log_error {
    () => {
        $crate::log::set_color(0xFA4B4B);
        $crate::log::set_indent(9);
        $crate::print!("[ERROR!]\n");
        $crate::log::set_indent(0);
    };
    ($($arg:tt)*) => {
        $crate::log::set_color(0xFA4B4B);
        $crate::print!("[ERROR!] ");
        $crate::log::set_color(0xFCA5A5);
        $crate::log::set_indent(9);
        $crate::print!("{}", format_args!($($arg)*));
        $crate::log::set_indent(0);
        $crate::print!("\n");
    };
}

#[macro_export]
macro_rules! log_ok {
    () => {
        $crate::log::set_color(0x12B76A);
        $crate::print!("[  OK  ]\n");
    };
    ($($arg:tt)*) => {
        $crate::log::set_color(0x12B76A);
        $crate::print!("[  OK  ] ");
        $crate::log::set_color(0xFFFFFF);
        $crate::log::set_indent(9);
        $crate::print!("{}", format_args!($($arg)*));
        $crate::log::set_indent(0);
        $crate::print!("\n");
    };
}

#[macro_export]
macro_rules! log_panic {
    () => {
        $crate::log::set_color(0xFA4B4B);
        $crate::log::set_indent(9);
        $crate::print!("[PANIC!]\n");
        $crate::log::set_indent(0);
    };
    ($($arg:tt)*) => {
        $crate::log::set_color(0xFA4B4B);
        $crate::print!("[PANIC!] ");
        $crate::log::set_indent(9);
        $crate::print!("{}", format_args!($($arg)*));
        $crate::log::set_indent(0);
        $crate::print!("\n");
    };
}
