pub enum Glyph {
    HalfWidth([u8; 16]),
    FullWidth([u16; 16]),
}

impl Glyph {
    pub fn get_pixel(&self, x: usize, y: usize) -> bool {
        y < 16
            && match self {
                Glyph::HalfWidth(rows) => x < 8 && rows[y] & (0x80 >> x) != 0,
                Glyph::FullWidth(rows) => x < 16 && rows[y] & (0x8000 >> x) != 0,
            }
    }

    pub fn get_width(&self) -> usize {
        match self {
            Glyph::HalfWidth(_) => 8,
            Glyph::FullWidth(_) => 16,
        }
    }
}

pub fn get_glyph(c: char) -> Option<&'static Glyph> {
    let code_point = c as usize;
    let mut offset: usize = 0;
    let mut result = None;
    for (start, end) in CODE_POINT_RANGES.iter() {
        if *start <= code_point && code_point < *end {
            result = Some(&GLYPH_TABLE[offset + code_point - start]);
            break;
        } else {
            offset += end - start;
        }
    }
    result
}

include!("glyph_table.rs");
