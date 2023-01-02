use bootloader_api::info::{FrameBufferInfo, PixelFormat, Optional, FrameBuffer};
use conquer_once::spin::OnceCell;
use spin::Mutex;
use crate::unifont;

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
        let color = match self.info.pixel_format {
            PixelFormat::Rgb => [r, g, b, 0],
            PixelFormat::Bgr => [b, g, r, 0],
            other => panic!("Unsupported pixel format: {:?}", other),
        };
        let bytes_per_pixel = self.info.bytes_per_pixel;
        let byte_offset = offset * bytes_per_pixel;
        self.framebuffer[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
    }
}

pub fn painter_init(framebuffer: &'static mut Optional<FrameBuffer>) {
    if let Optional::Some(framebuffer) = framebuffer {
        let info = framebuffer.info();
        let framebuffer = framebuffer.buffer_mut();
        let painter = PAINTER.get_or_init(move || LockedPainter::new(framebuffer, info));
        painter.0.lock().draw_pixel(0, 0, 0xFF0000);
    }
}

pub fn draw_rect(x: usize, y: usize, width: usize, height: usize, color: u32) {
    let painter = PAINTER.get().unwrap();
    let mut painter = painter.0.lock();
    for x in x..(x + width) {
        for y in y..(y + height) {
            painter.draw_pixel(x, y, color);
        }
    }
}

fn draw_char(x: usize, y: usize, c: char, color: u32) -> usize {
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

pub fn get_height() -> usize {
    PAINTER.get().unwrap().0.lock().get_height()
}

pub fn get_width() -> usize {
    PAINTER.get().unwrap().0.lock().get_width()
}
