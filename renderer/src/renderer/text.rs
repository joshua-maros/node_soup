use itertools::Itertools;
use theme::{BIG_VALUE_COLOR, BIG_VALUE_SIZE, NODE_LABEL_COLOR, NODE_LABEL_SIZE};
use wgpu_glyph::{Extra, FontId, HorizontalAlign, Layout, VerticalAlign};

use super::fonts::{FONT_BOLD, FONT_LIGHT};

#[derive(Clone, Debug)]
pub struct Text {
    pub sections: Vec<Section>,
    pub center: [f32; 2],
    pub bounds: [f32; 2],
    pub horizontal_align: HorizontalAlign,
    pub vertical_align: VerticalAlign,
}

impl Text {
    pub fn as_wgpu_section(&self, screen_height: f32) -> wgpu_glyph::Section {
        wgpu_glyph::Section {
            text: self
                .sections
                .iter()
                .map(Section::as_wgpu_text)
                .collect_vec(),
            screen_position: (self.center[0], screen_height - self.center[1]),
            bounds: (self.bounds[0], self.bounds[1]),
            layout: Layout::default_single_line()
                .h_align(self.horizontal_align)
                .v_align(self.vertical_align),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Section {
    pub text: String,
    pub color: [f32; 4],
    pub size: f32,
    pub style: usize,
}

impl Section {
    pub fn node_label(text: String) -> Self {
        Self {
            text,
            color: NODE_LABEL_COLOR,
            size: NODE_LABEL_SIZE,
            style: FONT_LIGHT,
        }
    }

    pub fn big_value_text(text: String) -> Self {
        Self {
            text,
            color: BIG_VALUE_COLOR,
            size: BIG_VALUE_SIZE,
            style: FONT_BOLD,
        }
    }

    pub(super) fn as_wgpu_text(&self) -> wgpu_glyph::Text {
        wgpu_glyph::Text {
            text: &self.text,
            scale: self.size.into(),
            font_id: FontId(self.style),
            extra: Extra {
                color: self.color,
                z: 0.0,
            },
        }
    }
}
