use crate::unifont;
use bootloader_api::info::{FrameBuffer, FrameBufferInfo, Optional, PixelFormat};
use conquer_once::spin::OnceCell;
use spin::Mutex;

pub static PAINTER: OnceCell<LockedPainter> = OnceCell::uninit();
pub struct LockedPainter(Mutex<Painter>);

impl LockedPainter {
    pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        LockedPainter(Mutex::new(Painter::new(framebuffer, info)))
    }

    /// Force-unlocks the painter to prevent deadlocks.
    ///
    /// ## Safety
    /// This method is unsafe and usage of it should be avoided.
    pub unsafe fn force_unlock(&self) {
        unsafe { self.0.force_unlock() };
    }
}

pub struct Painter {
    framebuffer: &'static mut [u8],
    info: FrameBufferInfo,
}

impl Painter {
    pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        Painter { framebuffer, info }
    }

    pub fn get_height(&self) -> usize {
        self.info.height
    }

    pub fn get_width(&self) -> usize {
        self.info.width
    }
    
    pub fn draw_pixel(&mut self, x: usize, y: usize, color: u32) {
        let offset = y * self.info.stride + x;
        let r = (color >> 16) as u8;
        let g = (color >> 8) as u8;
        let b = color as u8;
        match self.info.pixel_format {
            PixelFormat::Bgr => {
                self.framebuffer[offset * 4] = b;
                self.framebuffer[offset * 4 + 1] = g;
                self.framebuffer[offset * 4 + 2] = r;
            }
            PixelFormat::Rgb => {
                self.framebuffer[offset * 4] = r;
                self.framebuffer[offset * 4 + 1] = g;
                self.framebuffer[offset * 4 + 2] = b;
            }
            other => panic!("Unsupported pixel format: {:?}", other),
        };
    }

    pub fn draw_rect(&mut self, x: usize, y: usize, width: usize, height: usize, color: u32) {
        let r = (color >> 16) as u8;
        let g = (color >> 8) as u8;
        let b = color as u8;
        match self.info.pixel_format {
            PixelFormat::Bgr => {
                for y in y..(y + height) {
                    let offset = y * self.info.stride;
                    for x in x..(x + width) {
                        let byte_offset = (offset + x) * 4;
                        self.framebuffer[byte_offset] = b;
                        self.framebuffer[byte_offset + 1] = g;
                        self.framebuffer[byte_offset + 2] = r;
                    }
                }
            }
            PixelFormat::Rgb => {
                for y in y..(y + height) {
                    let offset = y * self.info.stride;
                    for x in x..(x + width) {
                        let byte_offset = (offset + x) * 4;
                        self.framebuffer[byte_offset] = r;
                        self.framebuffer[byte_offset + 1] = g;
                        self.framebuffer[byte_offset + 2] = b;
                    }
                }
            }
            other => panic!("Unsupported pixel format: {:?}", other),
        };
    }
}

pub fn painter_init(framebuffer: &'static mut Optional<FrameBuffer>) {
    if let Optional::Some(framebuffer) = framebuffer {
        let info = framebuffer.info();
        let framebuffer = framebuffer.buffer_mut();
        PAINTER.get_or_init(move || LockedPainter::new(framebuffer, info));
    }
}

pub fn draw_rect(x: usize, y: usize, width: usize, height: usize, color: u32) {
    let painter = PAINTER.get().unwrap();
    let mut painter = painter.0.lock();
    painter.draw_rect(x, y, width, height, color);
}

pub fn draw_pixel(x: usize, y: usize, color: u32) {
    let painter = PAINTER.get().unwrap();
    let mut painter = painter.0.lock();
    painter.draw_pixel(x, y, color);
}

pub fn draw_char(x: usize, y: usize, c: char, color: u32) -> usize {
    let painter = PAINTER.get().unwrap();
    let mut painter = painter.0.lock();
    if let Some(glyph) = unifont::get_glyph(c) {
        let glyph_width = glyph.get_width();
        for i in 0..glyph_width {
            for j in 0..16 {
                if glyph.get_pixel(i, j) {
                    painter.draw_pixel(x + i, y + j, color);
                }
            }
        }
        return glyph_width;
    }
    return 0;
}

pub fn draw_str(x: usize, y: usize, s: &str, color: u32) {
    let mut x = x;
    for c in s.chars() {
        x += draw_char(x, y, c, color);
    }
}

fn draw_line_low(x0: usize, y0: usize, x1: usize, y1: usize, color: u32) {
    let painter = PAINTER.get().unwrap();
    let mut painter = painter.0.lock();
    let dx = x1 - x0;
    let dy: usize;
    let inverse: bool;
    if y1 > y0 {
        dy = y1 - y0;
        inverse = false;
    } else {
        dy = y0 - y1;
        inverse = true;
    }
    let mut d: i32 = (2 * dy as i32) - dx as i32;
    let mut y = y0;
    for x in x0..=x1 {
        painter.draw_pixel(x, y, color);
        if d > 0 {
            if inverse == true {
                y -= 1;
            } else {
                y += 1;
            }
            d += 2 * (dy as i32 - dx as i32);
        } else {
            d += 2 * dy as i32;
        }
    }
}

fn draw_line_high(x0: usize, y0: usize, x1: usize, y1: usize, color: u32) {
    let painter = PAINTER.get().unwrap();
    let mut painter = painter.0.lock();
    let dy = y1 - y0;
    let dx: usize;
    let inverse: bool;
    if x1 > x0 {
        dx = x1 - x0;
        inverse = false;
    } else {
        dx = x0 - x1;
        inverse = true;
    }
    let mut d = (2 * dx as i32) - dy as i32;
    let mut x = x0;
    for y in y0..=y1 {
        painter.draw_pixel(x, y, color);
        if d > 0 {
            if inverse == true {
                x -= 1;
            } else {
                x += 1;
            }
            d += 2 * (dx as i32 - dy as i32);
        } else {
            d += 2 * dx as i32;
        }
    }
}

pub fn draw_line(x0: usize, y0: usize, x1: usize, y1: usize, color: u32) {
    let dx = if x0 > x1 { x0 - x1 } else { x1 - x0 };
    let dy = if y0 > y1 { y0 - y1 } else { y1 - y0 };
    if x0 == x1 {
        draw_rect(x0, y0, 1, dy + 1, color);
    } else if y0 == y1 {
        draw_rect(x0, y0, dx + 1, 1, color);
    } else {
        if dy < dx {
            if x0 > x1 {
                draw_line_low(x1, y1, x0, y0, color);
            } else {
                draw_line_low(x0, y0, x1, y1, color);
            }
        } else {
            if y0 > y1 {
                draw_line_high(x1, y1, x0, y0, color);
            } else {
                draw_line_high(x0, y0, x1, y1, color);
            }
        }
    }
}

pub fn get_height() -> usize {
    PAINTER.get().unwrap().0.lock().get_height()
}

pub fn get_width() -> usize {
    PAINTER.get().unwrap().0.lock().get_width()
}

pub fn get_char_width(c: char) -> usize {
    if let Some(glyph) = unifont::get_glyph(c) {
        return glyph.get_width();
    }
    return 0;
}
