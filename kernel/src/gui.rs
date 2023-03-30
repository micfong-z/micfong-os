use crate::{layer, colors, bitmap};

impl layer::Layer {
    /// Draw a window with the given title, filling the entire layer.
    pub fn draw_window(&mut self, _title: &str) {
        let layer_width = self.get_width();
        let layer_height = self.get_height();
        if layer_width < 56 || layer_height < 30 {
            panic!("Layer is too small to draw a window");
        }

        self.draw_rect(0, 0, layer_width, 25, colors::WINDOW_BORDER);
        self.draw_rect(0, 25, 1, layer_height - 25, colors::WINDOW_BORDER);
        self.draw_rect(layer_width - 1, 25, 1, layer_height - 25, colors::WINDOW_BORDER);
        self.draw_rect(0, layer_height - 1, layer_width, 1, colors::WINDOW_BORDER);
        self.draw_bitmap(9, 7, 11, 11, &bitmap::MINIMIZE_ICON);
        self.draw_bitmap(layer_width - 20, 7, 11, 11, &bitmap::CLOSE_ICON);
        
        self.draw_rect(1, 25, layer_width - 2, layer_height - 26, colors::DESKTOP_BACKGROUND);
    }
}
