use bootloader_api::info::{FrameBuffer, FrameBufferInfo, Optional, PixelFormat};
use conquer_once::spin::OnceCell;
use spin::Mutex;

use crate::{
    colors::{self, Color},
    layer, log_error, unifont,
};

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

    pub fn get_height(&self) -> u32 {
        self.info.height as u32
    }

    pub fn get_width(&self) -> u32 {
        self.info.width as u32
    }

    pub fn move_all_up(&mut self, y: u32) {
        let y = y as usize;
        let bytes_per_row = self.info.stride * self.info.bytes_per_pixel;
        let bytes_per_move = y * bytes_per_row;
        let bytes_to_move = (self.info.height - y) * bytes_per_row;

        // this unsafe block should be fine since PAINTER is locked with a Mutex
        unsafe {
            let src = self.framebuffer.as_ptr().add(bytes_per_move);
            let dst = self.framebuffer.as_mut_ptr();
            core::ptr::copy(src, dst, bytes_to_move);
        }
    }

    pub fn draw_pixel(&mut self, x: u32, y: u32, color: Color) {
        let x = x as usize;
        let y = y as usize;
        if x >= self.info.width || y >= self.info.height || color.a == 0.0 {
            return;
        }
        let offset = y * self.info.stride + x;
        let r = color.r;
        let g = color.g;
        let b = color.b;
        let bytes_per_pixel = self.info.bytes_per_pixel;
        match self.info.pixel_format {
            PixelFormat::Bgr => {
                self.framebuffer[offset * bytes_per_pixel] = b;
                self.framebuffer[offset * bytes_per_pixel + 1] = g;
                self.framebuffer[offset * bytes_per_pixel + 2] = r;
            }
            PixelFormat::Rgb => {
                self.framebuffer[offset * bytes_per_pixel] = r;
                self.framebuffer[offset * bytes_per_pixel + 1] = g;
                self.framebuffer[offset * bytes_per_pixel + 2] = b;
            }
            other => panic!("Unsupported pixel format: {:?}", other),
        };
    }

    pub fn draw_rect(&mut self, x: u32, y: u32, mut width: u32, mut height: u32, color: Color) {
        if color.a == 0.0 {
            return;
        }
        if x + width >= (self.info.width as u32) {
            width = (self.info.width as u32) - x;
        }
        if y + height >= (self.info.height as u32) {
            height = (self.info.height as u32) - y;
        }
        let x = x as usize;
        let y = y as usize;
        let width = width as usize;
        let height = height as usize;
        let r = color.r;
        let g = color.g;
        let b = color.b;
        let bytes_per_pixel = self.info.bytes_per_pixel;
        match self.info.pixel_format {
            PixelFormat::Bgr => {
                for y in y..(y + height) {
                    let offset = y * self.info.stride;
                    for x in x..(x + width) {
                        let byte_offset = (offset + x) * bytes_per_pixel;
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
                        let byte_offset = (offset + x) * bytes_per_pixel;
                        self.framebuffer[byte_offset] = r;
                        self.framebuffer[byte_offset + 1] = g;
                        self.framebuffer[byte_offset + 2] = b;
                    }
                }
            }
            other => panic!("Unsupported pixel format: {:?}", other),
        };
    }

    pub fn layer_controller_render(&mut self, layer_controller: &layer::LayerController) {
        for layer in layer_controller.get_layers_iter() {
            let layer = layer.lock();
            if layer.is_hidden() {
                continue;
            }

            let (x_pos, y_pos) = layer.get_pos_usize();
            let width = layer.get_width() as usize;
            let height = layer.get_height() as usize;
            let framebuffer = layer.get_framebuffer();

            match self.info.pixel_format {
                PixelFormat::Bgr => {
                    for y in 0..height {
                        let offset = (y_pos + y) * self.info.stride;
                        if  y_pos + y >= self.info.height {
                            break;
                        }
                        for x in 0..width {
                            if (framebuffer[y * width + x].a as f32) == 0.0 || x_pos + x >= self.info.width {
                                continue;
                            }
                            let byte_offset = (offset + x_pos + x) * self.info.bytes_per_pixel;
                            let color = framebuffer[y * width + x];
                            self.framebuffer[byte_offset] = color.b;
                            self.framebuffer[byte_offset + 1] = color.g;
                            self.framebuffer[byte_offset + 2] = color.r;
                        }
                    }
                }
                PixelFormat::Rgb => {
                    for y in 0..height {
                        let offset = (y_pos + y) * self.info.stride;
                        if  y_pos + y >= self.info.height {
                            break;
                        }
                        for x in 0..width {
                            if (framebuffer[y * width + x].a as f32) == 0.0 || x_pos + x >= self.info.width {
                                continue;
                            }
                            let byte_offset = (offset + x_pos + x) * self.info.bytes_per_pixel;
                            let color = framebuffer[y * width + x];
                            self.framebuffer[byte_offset] = color.r;
                            self.framebuffer[byte_offset + 1] = color.g;
                            self.framebuffer[byte_offset + 2] = color.b;
                        }
                    }
                }
                other => panic!("Unsupported pixel format: {:?}", other),
            };
        }
    }

    pub fn layer_controller_render_partial(
        &mut self,
        layer_controller: &layer::LayerController,
        viewport_x: u32,
        viewport_y: u32,
        width: u32,
        height: u32,
    ) {
        for layer in layer_controller.get_layers_iter() {
            let layer = layer.lock();
            if layer.is_hidden() {
                continue;
            }

            let (x_pos, y_pos) = layer.get_pos_usize();
            let crop_x = viewport_x as usize;
            let crop_y = viewport_y as usize;
            let crop_width = width as usize;
            let crop_height = height as usize;

            let width = layer.get_width() as usize;
            let height = layer.get_height() as usize;
            let framebuffer = layer.get_framebuffer();

            match self.info.pixel_format {
                PixelFormat::Bgr => {
                    for y in 0..height {
                        let offset = (y_pos + y) * self.info.stride;
                        let screen_y = y + y_pos;
                        if screen_y < crop_y || screen_y >= crop_y + crop_height {
                            continue;
                        }
                        if screen_y >= self.info.height {
                            break;
                        }
                        for x in 0..width {
                            let screen_x = x + x_pos;
                            if screen_x < crop_x || screen_x >= crop_x + crop_width {
                                continue;
                            }
                            if (framebuffer[y * width + x].a as f32) == 0.0 || screen_x >= self.info.width {
                                continue;
                            }
                            let byte_offset = (offset + x_pos + x) * self.info.bytes_per_pixel;
                            let color = framebuffer[y * width + x];
                            self.framebuffer[byte_offset] = color.b;
                            self.framebuffer[byte_offset + 1] = color.g;
                            self.framebuffer[byte_offset + 2] = color.r;
                        }
                    }
                }
                PixelFormat::Rgb => {
                    for y in 0..height {
                        let offset = (y_pos + y) * self.info.stride;
                        let screen_y = y + y_pos;
                        if screen_y < crop_y || screen_y >= crop_y + crop_height {
                            continue;
                        }
                        if screen_y >= self.info.height {
                            break;
                        }
                        for x in 0..width {
                            let screen_x = x + x_pos;
                            if screen_x < crop_x || screen_x >= crop_x + crop_width {
                                continue;
                            }
                            if (framebuffer[y * width + x].a as f32) == 0.0 || screen_x >= self.info.width {
                                continue;
                            }
                            let byte_offset = (offset + x_pos + x) * self.info.bytes_per_pixel;
                            let color = framebuffer[y * width + x];
                            self.framebuffer[byte_offset] = color.r;
                            self.framebuffer[byte_offset + 1] = color.g;
                            self.framebuffer[byte_offset + 2] = color.b;
                        }
                    }
                }
                other => panic!("Unsupported pixel format: {:?}", other),
            };
        }
    }
}

pub fn move_all_up(y: u32) {
    PAINTER.get().unwrap().0.lock().move_all_up(y);
}

pub fn painter_init(framebuffer: &'static mut Optional<FrameBuffer>) {
    if let Optional::Some(framebuffer) = framebuffer {
        let info = framebuffer.info();
        let framebuffer = framebuffer.buffer_mut();
        PAINTER.init_once(move || LockedPainter::new(framebuffer, info));
    }
}

pub fn draw_rect(x: u32, y: u32, width: u32, height: u32, color: Color) {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        let painter = PAINTER.get().unwrap();
        let mut painter = painter.0.lock();
        painter.draw_rect(x, y, width, height, color);
    });
}

pub fn draw_pixel(x: u32, y: u32, color: Color) {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        let painter = PAINTER.get().unwrap();
        let mut painter = painter.0.lock();
        painter.draw_pixel(x, y, color);
    });
}

pub fn draw_char(x: u32, y: u32, c: char, color: Color) -> u32 {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        let painter = PAINTER.get().unwrap();
        let mut painter = painter.0.lock();
        if let Some(glyph) = unifont::get_glyph(c) {
            let glyph_width = glyph.get_width() as u32;
            for i in 0..glyph_width {
                for j in 0..16u32 {
                    if glyph.get_pixel(i as usize, j as usize) {
                        painter.draw_pixel(x + i, y + j, color);
                    }
                }
            }
            return glyph_width;
        }
        return 0;
    })
}

pub fn draw_str(x: u32, y: u32, s: &str, color: Color) {
    let mut x = x;
    for c in s.chars() {
        x += draw_char(x, y, c, color);
    }
}

fn draw_line_low(x0: u32, y0: u32, x1: u32, y1: u32, color: Color) {
    let painter = PAINTER.get().unwrap();
    let mut painter = painter.0.lock();
    let dx = x1 - x0;
    let dy: u32;
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

fn draw_line_high(x0: u32, y0: u32, x1: u32, y1: u32, color: Color) {
    let painter = PAINTER.get().unwrap();
    let mut painter = painter.0.lock();
    let dy = y1 - y0;
    let dx: u32;
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

pub fn draw_line(x0: u32, y0: u32, x1: u32, y1: u32, color: Color) {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
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
    });
}

pub fn get_height() -> u32 {
    PAINTER.get().unwrap().0.lock().get_height()
}

pub fn get_width() -> u32 {
    PAINTER.get().unwrap().0.lock().get_width()
}

pub fn get_char_width(c: char) -> u32 {
    if let Some(glyph) = unifont::get_glyph(c) {
        return glyph.get_width() as u32;
    }
    return 0;
}

pub fn draw_bitmap(x: u32, y: u32, width: u32, height: u32, bitmap: &[Color]) {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        let painter = PAINTER.get().unwrap();
        let mut painter = painter.0.lock();
        if width * height != bitmap.len() as u32 {
            log_error!("Bitmap size does not match width and height");
            return;
        }
        for i in 0..width {
            for j in 0..height {
                painter.draw_pixel(x + i, y + j, bitmap[(j * width + i) as usize]);
            }
        }
    });
}

pub fn layer_controller_render(layer_controller: &layer::LayerController) {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        let painter = PAINTER.get().unwrap();
        let mut painter = painter.0.lock();
        painter.layer_controller_render(layer_controller);
    });
}

pub fn layer_controller_render_partial(
    layer_controller: &layer::LayerController,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        let painter = PAINTER.get().unwrap();
        let mut painter = painter.0.lock();
        painter.layer_controller_render_partial(layer_controller, x, y, width, height);
    });
}
