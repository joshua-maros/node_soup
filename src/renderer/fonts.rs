use ab_glyph::Font;
use wgpu_glyph::{ab_glyph::FontArc, GlyphBrush, GlyphBrushBuilder};

use super::{render_device::RenderDevice, render_target::RenderTarget};

pub struct Fonts {
    pub regular: GlyphBrush<()>,
}

pub const FONT_LIGHT: usize = 1;

macro_rules! fonts {
    ($($name:expr => $style:expr,)* ) => {
        [
            $((
                include_bytes!($name),
                $style,
            )),*
        ]
    }
}

const FONT_DATA: [(&[u8], usize); 2] = fonts!(
    "fonts/Ubuntu-Regular.ttf" => 0,
    "fonts/Ubuntu-Light.ttf" => FONT_LIGHT,
);

impl Fonts {
    pub fn new(device: &RenderDevice, target: &RenderTarget) -> Self {
        let mut fonts = vec![];
        for (index, (data, expected_style)) in FONT_DATA.into_iter().enumerate() {
            assert_eq!(index, expected_style);
            fonts.push(FontArc::try_from_slice(data).unwrap());
        }

        Self {
            regular: GlyphBrushBuilder::using_fonts(fonts).build(device.device(), target.format()),
        }
    }
}
