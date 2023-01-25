use wgpu::{
    util::{BufferInitDescriptor, DeviceExt, StagingBelt},
    Buffer, BufferUsages, CommandEncoder, CommandEncoderDescriptor, LoadOp, Operations, RenderPass,
    RenderPassColorAttachment, RenderPassDescriptor, SurfaceError, SurfaceTexture, TextureView,
    TextureViewDescriptor,
};
use wgpu_glyph::{HorizontalAlign, Layout, Section, Text, VerticalAlign};

use super::{ActiveRenderInfo, MutableResources, ReadOnlyResources};
use crate::{
    renderer::{fonts::Fonts, shapes::Shapes},
    theme::{self},
};

pub(super) fn render_text(
    ror: &ReadOnlyResources,
    mr: &mut MutableResources,
    info: &mut ActiveRenderInfo,
) {
    let sections = info.shapes.text.iter().map(|text| Section {
        text: vec![Text::new(&text.text)
            .with_color(text.color)
            .with_scale(text.size)],
        screen_position: (
            text.position[0],
            ror.target.size().height - text.position[1],
        ),
        bounds: (text.bounds[0], text.bounds[1]),
        layout: Layout::default_single_line()
            .h_align(HorizontalAlign::Center)
            .v_align(VerticalAlign::Center),
    });
    for section in sections {
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
