mod create;
mod render;

use wgpu::{Buffer, RenderPipeline};

use super::{render_device::RenderDevice, render_target::RenderTarget};

pub struct RenderEngine {
    pub(super) device: RenderDevice,
    pub(super) target: RenderTarget,
    pub(super) rect_verts: Buffer,
    pub(super) rect_pipeline: RenderPipeline,
}
