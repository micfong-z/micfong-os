use conquer_once::spin::OnceCell;
use spin::Mutex;

use crate::{colors::{self, Color}, graphics};

pub static LOGGER: OnceCell<LockedLogger> = OnceCell::uninit();

pub struct LockedLogger(Mutex<Logger>);

impl LockedLogger {
    pub fn new(line_height: u32, margin: u32) -> Self {
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
    line_height: u32,
    margin: u32,
    x: u32,
    y: u32,
    color: Color,
    indent: u32,
}

impl Logger {
    pub fn new(line_height: u32, margin: u32) -> Self {
        Logger {
            line_height,
            margin,
            x: margin,
            y: margin,
            color: colors::WHITE,
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
            colors::DESKTOP_BACKGROUND,
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
            colors::DESKTOP_BACKGROUND,
        );
        // set the cursor to the last line

        // self.y = self.margin;
        // graphics::draw_rect(self.x, self.margin, graphics::get_width() - self.margin * 2, self.line_height, 0x202020);
    }

    fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    fn set_indent(&mut self, indent: u32) {
        self.indent = indent;
    }

    fn backspace(&mut self) {
        if self.x > self.margin {
            self.x -= 8;
            graphics::draw_rect(
                self.x,
                self.y,
                8,
                self.line_height,
                colors::DESKTOP_BACKGROUND,
            );
        }
    }
}

pub fn backspace() {
    let mut logger = LOGGER.get().unwrap().0.lock();
    logger.backspace();
}

pub fn logger_init(line_height: u32, margin: u32) {
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

pub fn set_color(color: Color) {
    let mut logger = LOGGER.get().unwrap().0.lock();
    logger.set_color(color);
}

pub fn set_indent(indent: u32) {
    let mut logger = LOGGER.get().unwrap().0.lock();
    logger.set_indent(indent);
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        LOGGER.get().unwrap().0.lock().write_fmt(args).unwrap();
    });
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
        $crate::log::set_color(colors::TRACE_LOG);
        $crate::print!("[-TRACE]\n");
    };
    ($($arg:tt)*) => {
        $crate::log::set_color(colors::TRACE_LOG);
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
        $crate::log::set_color(colors::WHITE);
        $crate::print!("[ INFO ]\n");
    };
    ($($arg:tt)*) => {
        $crate::log::set_color(colors::WHITE);
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
        $crate::log::set_color(colors::YELLOW);
        $crate::print!("[ WARN ]\n");
    };
    ($($arg:tt)*) => {
        $crate::log::set_color(colors::YELLOW);
        $crate::print!("[ WARN ] ");
        $crate::log::set_color(colors::BRIGHT_YELLOW);
        $crate::log::set_indent(9);
        $crate::print!("{}", format_args!($($arg)*));
        $crate::log::set_indent(0);
        $crate::print!("\n");
    };
}

#[macro_export]
macro_rules! log_error {
    () => {
        $crate::log::set_color(colors::RED);
        $crate::log::set_indent(9);
        $crate::print!("[ERROR!]\n");
        $crate::log::set_indent(0);
    };
    ($($arg:tt)*) => {
        $crate::log::set_color(colors::RED);
        $crate::print!("[ERROR!] ");
        $crate::log::set_color(colors::BRIGHT_RED);
        $crate::log::set_indent(9);
        $crate::print!("{}", format_args!($($arg)*));
        $crate::log::set_indent(0);
        $crate::print!("\n");
    };
}

#[macro_export]
macro_rules! log_ok {
    () => {
        $crate::log::set_color(colors::GREEN);
        $crate::print!("[  OK  ]\n");
    };
    ($($arg:tt)*) => {
        $crate::log::set_color(colors::GREEN);
        $crate::print!("[  OK  ] ");
        $crate::log::set_color(colors::WHITE);
        $crate::log::set_indent(9);
        $crate::print!("{}", format_args!($($arg)*));
        $crate::log::set_indent(0);
        $crate::print!("\n");
    };
}

#[macro_export]
macro_rules! log_panic {
    () => {
        $crate::log::set_color(colors::RED);
        $crate::log::set_indent(9);
        $crate::print!("[PANIC!]\n");
        $crate::log::set_indent(0);
    };
    ($($arg:tt)*) => {
        $crate::log::set_color(colors::RED);
        $crate::print!("[PANIC!] ");
        $crate::log::set_indent(9);
        $crate::print!("{}", format_args!($($arg)*));
        $crate::log::set_indent(0);
        $crate::print!("\n");
    };
}
