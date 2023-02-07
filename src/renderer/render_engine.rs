mod create;
mod render;
mod render_rects;
mod render_text;
mod clear;
mod render_icons;

use wgpu::{util::StagingBelt, Buffer, RenderPipeline, TextureView, CommandEncoder, BindGroup};

use super::{fonts::Fonts, render_device::RenderDevice, render_target::RenderTarget, Shapes};

struct ReadOnlyResources {
    device: RenderDevice,
    target: RenderTarget,
    rect_verts: Buffer,
    rect_pipeline: RenderPipeline,
    icon_pipeline: RenderPipeline,
    icon_texture_bind_group: BindGroup,
}

struct MutableResources {
    staging_belt: StagingBelt,
    fonts: Fonts,
}

struct ActiveRenderInfo<'a> {
    shapes: &'a Shapes,
    view: &'a TextureView,
    encoder: &'a mut CommandEncoder,
}

pub struct RenderEngine {
    ror: ReadOnlyResources,
    mr: MutableResources,
}
