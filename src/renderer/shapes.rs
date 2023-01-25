use super::rect_data::RectInstance;

#[derive(Clone, Debug)]
pub struct Shapes {
    pub rects: Vec<RectInstance>,
}

impl Shapes {
    pub fn new() -> Self {
        Self { rects: vec![] }
    }

    pub fn push_rect(&mut self, rect: RectInstance) {
        self.rects.push(rect)
    }

    pub fn append(&mut self, other: Self) {
        self.rects.append(&mut { other.rects });
    }
}
