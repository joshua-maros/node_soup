use wgpu_glyph::Section;

use super::{Position, RectInstance, Size, Text};

#[derive(Clone, Debug)]
pub struct Shapes {
    pub rects: Vec<RectInstance>,
    pub texts: Vec<Text>,
}

impl Shapes {
    pub fn new() -> Self {
        Self {
            rects: vec![],
            texts: vec![],
        }
    }

    pub fn push_rect(&mut self, rect: RectInstance) {
        self.rects.push(rect)
    }

    pub fn push_text(&mut self, text: Text) {
        self.texts.push(text)
    }

    pub fn append(&mut self, other: Self) {
        self.rects.append(&mut { other.rects });
        self.texts.append(&mut { other.texts });
    }
}
