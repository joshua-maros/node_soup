mod create;
mod render;
mod render_rects;
mod render_text;

use wgpu::{util::StagingBelt, Buffer, RenderPipeline, TextureView, CommandEncoder};

use super::{fonts::Fonts, render_device::RenderDevice, render_target::RenderTarget, Shapes};

struct ReadOnlyResources {
    device: RenderDevice,
    target: RenderTarget,
    rect_verts: Buffer,
    rect_pipeline: RenderPipeline,
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
