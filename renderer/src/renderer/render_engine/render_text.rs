use itertools::Itertools;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt, StagingBelt},
    Buffer, BufferUsages, CommandEncoder, CommandEncoderDescriptor, LoadOp, Operations, RenderPass,
    RenderPassColorAttachment, RenderPassDescriptor, SurfaceError, SurfaceTexture, TextureView,
    TextureViewDescriptor,
};
use wgpu_glyph::{Extra, FontId, HorizontalAlign, Layout, VerticalAlign};

use super::{ActiveRenderInfo, MutableResources, ReadOnlyResources};
use crate::{
    renderer::{fonts::Fonts, shapes::Shapes, Text},
};

pub(super) fn render_text(
    ror: &ReadOnlyResources,
    mr: &mut MutableResources,
    info: &mut ActiveRenderInfo,
) {
    let screen_height = ror.target.size().height;
    let texts = info.shapes.texts.iter();
    let wgpu_sections = texts.map(|text| text.as_wgpu_section(screen_height));
    for section in wgpu_sections {
        mr.fonts.regular.queue(section);
    }
    mr.fonts
        .regular
        .draw_queued(
            ror.device.device(),
            &mut mr.staging_belt,
            info.encoder,
            info.view,
            ror.target.size().width as u32,
            ror.target.size().height as u32,
        )
        .unwrap();
}
