use wgpu_glyph::{ab_glyph::FontArc, GlyphBrush, GlyphBrushBuilder};

use super::{render_device::RenderDevice, render_target::RenderTarget};

pub struct Fonts {
    pub regular: GlyphBrush<()>,
}

impl Fonts {
    pub fn new(device: &RenderDevice, target: &RenderTarget) -> Self {
        let mut brushes = vec![];
        for data in &[include_bytes!("fonts/Ubuntu-Regular.ttf")] {
            let font = FontArc::try_from_slice(*data).unwrap();
            let brush = GlyphBrushBuilder::using_font(font).build(device.device(), target.format());
            brushes.push(brush);
        }

        Self {
            regular: brushes.pop().unwrap(),
        }
    }
}
