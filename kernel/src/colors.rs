#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

impl Color {
    pub const fn new(hex: u32, alpha: f32) -> Self {
        let r = (hex >> 16) as u8;
        let g = (hex >> 8) as u8;
        let b = hex as u8;
        Self { r, g, b, a: alpha }
    }

    pub const fn new_rgba(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

pub const RED: Color = Color::new(0xFA4B4B, 1.0);
pub const ORANGE: Color = Color::new(0xFF7E35, 1.0);
pub const YELLOW: Color = Color::new(0xFCBB13, 1.0);
pub const GREEN: Color = Color::new(0x12B76A, 1.0);
pub const WHITE: Color = Color::new(0xFFFFFF, 1.0);
pub const BLACK: Color = Color::new(0x000000, 1.0);

pub const DESKTOP_BACKGROUND: Color = Color::new(0x202020, 1.0);
pub const BRIGHT_RED: Color = Color::new(0xFCA5A5, 1.0);
pub const BRIGHT_YELLOW: Color = Color::new(0xFDDD89, 1.0);
pub const TRACE_LOG: Color = Color::new(0xAAAAAA, 1.0);
pub const MOUSE_FILL: Color = Color::new(0x101010, 1.0);

pub const TRANSPARENT: Color = Color::new(0x000000, 0.0);
