use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, CommandEncoder, CommandEncoderDescriptor, LoadOp, Operations, RenderPass,
    RenderPassColorAttachment, RenderPassDescriptor, SurfaceError, SurfaceTexture, TextureView,
    TextureViewDescriptor,
};

use super::{ActiveRenderInfo, ReadOnlyResources};
use crate::{
    renderer::{shapes::Shapes, vertex_data::RECT_VERTS_LEN},
    theme::{self},
};

pub(super) fn clear(ror: &ReadOnlyResources, info: &mut ActiveRenderInfo) {
    let render_pass = start_render_pass(info);
    drop(render_pass);
}

fn start_render_pass<'a, 'b: 'a>(info: &'a mut ActiveRenderInfo<'b>) -> RenderPass<'a> {
    let render_pass_desc = RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: info.view,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(theme::BG),
                store: true,
            },
        })],
        depth_stencil_attachment: None,
    };
    info.encoder.begin_render_pass(&render_pass_desc)
}
