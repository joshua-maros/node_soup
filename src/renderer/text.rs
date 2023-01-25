use itertools::Itertools;
use wgpu_glyph::{Extra, FontId, HorizontalAlign, Layout, VerticalAlign};

use super::fonts::FONT_LIGHT;
use crate::theme::{
    NODE_LABEL_COLOR, NODE_LABEL_SIZE, PARAMETER_LABEL_COLOR, PARAMETER_LABEL_SIZE,
};

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
    pub fn parameter_label(text: String) -> Self {
        Self {
            text,
            color: PARAMETER_LABEL_COLOR,
            size: PARAMETER_LABEL_SIZE,
            style: 0,
        }
    }

    pub fn node_label(text: String) -> Self {
        Self {
            text,
            color: NODE_LABEL_COLOR,
            size: NODE_LABEL_SIZE,
            style: FONT_LIGHT,
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
