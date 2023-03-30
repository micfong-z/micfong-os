use alloc::{sync::Arc, vec, vec::Vec};

use conquer_once::spin::OnceCell;
use spin::Mutex;

use crate::{colors::Color, graphics};

pub struct Layer {
    framebuffer: Vec<Color>,
    width: u32,
    height: u32,
    x_pos: u32,
    y_pos: u32,
    z_index: u32,
    hidden: bool,
    index: u32, // index in the layer controller
}

impl Layer {
    pub fn new(width: u32, height: u32, x_pos: u32, y_pos: u32, z_index: u32) -> Self {
        Layer {
            framebuffer: vec![Color::new(0x000000, 0.0); (width * height) as usize],
            width,
            height,
            x_pos,
            y_pos,
            z_index,
            hidden: false,
            index: 0,
        }
    }

    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    fn set_z_index(&mut self, z_index: u32) {
        self.z_index = z_index;
    }

    pub fn set_pos(&mut self, x_pos: u32, y_pos: u32) {
        self.x_pos = x_pos;
        self.y_pos = y_pos;
    }

    pub fn get_pos_usize(&self) -> (usize, usize) {
        (self.x_pos as usize, self.y_pos as usize)
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn get_framebuffer(&self) -> &[Color] {
        &self.framebuffer
    }

    pub fn draw_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x >= self.width || y >= self.height || color.a == 0.0 {
            return;
        }

        self.framebuffer[(y * self.width + x) as usize] = color;
    }

    pub fn draw_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: Color) {
        for y in y..y + height {
            for x in x..x + width {
                self.draw_pixel(x, y, color);
            }
        }
    }

    pub fn draw_bitmap(&mut self, x_pos: u32, y_pos: u32, width: u32, height: u32, bitmap: &[Color]) {
        for y in 0..height {
            for x in 0..width {
                self.draw_pixel(x_pos + x, y_pos + y, bitmap[(y * width + x) as usize]);
            }
        }
    }
}

pub struct LayerController {
    layers: Vec<Arc<Mutex<Layer>>>,
}

impl LayerController {
    pub fn add_layer(&mut self, mut layer: Layer) -> Arc<Mutex<Layer>> {
        let mut left = 0;
        let mut right = self.layers.len();

        while left < right {
            let mid = (left + right) / 2;
            if self.layers[mid].lock().z_index > layer.z_index {
                right = mid;
            } else {
                left = mid + 1;
            }
        }

        layer.index = left as u32;

        let layer = Arc::new(Mutex::new(layer));
        self.layers.insert(left, layer.clone());
        layer
    }

    pub fn add_layer_arc(&mut self, layer: Arc<Mutex<Layer>>) {
        let mut left = 0;
        let mut right = self.layers.len();

        while left < right {
            let mid = (left + right) / 2;
            if self.layers[mid].lock().z_index > layer.lock().z_index {
                right = mid;
            } else {
                left = mid + 1;
            }
        }

        layer.lock().index = left as u32;

        self.layers.insert(left, layer.clone());
    }

    pub fn remove_layer(&mut self, layer: Arc<Mutex<Layer>>) -> Arc<Mutex<Layer>> {
        self.layers.remove(layer.lock().index as usize)
    }

    pub fn get_layer_count(&self) -> usize {
        self.layers.len()
    }

    pub fn set_layer_z_index(&mut self, layer: Arc<Mutex<Layer>>, z_index: u32) {
        let layer = self.remove_layer(layer);
        layer.lock().set_z_index(z_index);
        self.add_layer_arc(layer);
    }

    pub fn render(&self) {
        graphics::layer_controller_render(&self);
    }

    pub fn render_partial(&self, x: u32, y: u32, width: u32, height: u32) {
        graphics::layer_controller_render_partial(&self, x, y, width, height);
    }

    pub fn get_layers_iter(&self) -> impl Iterator<Item = &Arc<Mutex<Layer>>> {
        self.layers.iter()
    }

    pub fn get_layers_iter_rev(&self) -> impl Iterator<Item = &Arc<Mutex<Layer>>> {
        self.layers.iter().rev()
    }
}

pub static LAYER_CONTROLLER: OnceCell<Mutex<LayerController>> = OnceCell::uninit();

pub fn init() {
    LAYER_CONTROLLER
        .try_init_once(|| Mutex::new(LayerController { layers: vec![] }))
        .expect("Layer controller already initialized");
}

pub fn add_layer(layer: Layer) -> Arc<Mutex<Layer>> {
    LAYER_CONTROLLER
        .try_get()
        .expect("Layer controller not initialized")
        .lock()
        .add_layer(layer)
}
