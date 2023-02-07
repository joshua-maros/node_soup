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

pub(super) fn render_icons(ror: &ReadOnlyResources, info: &mut ActiveRenderInfo) {
    let (instance_buffer, len) = upload_icons(ror, info);
    let mut render_pass = start_render_pass(info);
    render_icon_instructions(&mut render_pass, &instance_buffer, len, ror);
}

fn render_icon_instructions<'a>(
    render_pass: &mut RenderPass<'a>,
    instance_buffer: &'a Buffer,
    len: usize,
    ror: &'a ReadOnlyResources,
) {
    render_pass.set_pipeline(&ror.icon_pipeline);
    render_pass.set_vertex_buffer(0, ror.rect_verts.slice(..));
    render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
    render_pass.set_bind_group(0, ror.target.surface_geometry_bind_group(), &[]);
    render_pass.set_bind_group(1, &ror.icon_texture_bind_group, &[]);
    render_pass.draw(0..RECT_VERTS_LEN as _, 0..len as _);
}

fn upload_icons(ror: &ReadOnlyResources, info: &mut ActiveRenderInfo) -> (Buffer, usize) {
    let contents = &info.shapes.icons;
    let buffer_desc = BufferInitDescriptor {
        label: Some("Icon Instance Buffer"),
        contents: bytemuck::cast_slice(contents),
        usage: BufferUsages::VERTEX,
    };
    let instance_buffer = ror.device.device().create_buffer_init(&buffer_desc);
    (instance_buffer, contents.len())
}

fn start_render_pass<'a, 'b: 'a>(info: &'a mut ActiveRenderInfo<'b>) -> RenderPass<'a> {
    let render_pass_desc = RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: info.view,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Load,
                store: true,
            },
        })],
        depth_stencil_attachment: None,
    };
    info.encoder.begin_render_pass(&render_pass_desc)
}
