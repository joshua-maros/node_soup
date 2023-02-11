use super::{image_data::ImageInstance, IconInstance, RectInstance, Text};

#[derive(Clone, Debug)]
pub struct Shapes {
    pub rects: Vec<RectInstance>,
    pub texts: Vec<Text>,
    pub icons: Vec<IconInstance>,
    pub images: Vec<ImageInstance>,
}

impl Shapes {
    pub fn new() -> Self {
        Self {
            rects: vec![],
            texts: vec![],
            icons: vec![],
            images: vec![],
        }
    }

    pub fn push_rect(&mut self, rect: RectInstance) {
        self.rects.push(rect)
    }

    pub fn push_text(&mut self, text: Text) {
        self.texts.push(text)
    }

    pub fn push_icon(&mut self, icon: IconInstance) {
        self.icons.push(icon)
    }

    pub fn push_image(&mut self, image: ImageInstance) {
        self.images.push(image)
    }

    pub fn append(&mut self, other: Self) {
        self.rects.append(&mut { other.rects });
        self.texts.append(&mut { other.texts });
        self.icons.append(&mut { other.icons });
        self.images.append(&mut { other.images });
    }
}
