use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, CommandEncoder, CommandEncoderDescriptor, LoadOp, Operations, RenderPass,
    RenderPassColorAttachment, RenderPassDescriptor, SurfaceError, SurfaceTexture, TextureView,
    TextureViewDescriptor,
};

use super::RenderEngine;
use crate::{
    renderer::{shapes::Shapes, vertex_data::RECT_VERTS_LEN},
    theme::{self},
};

impl RenderEngine {
    pub fn render(&self, shapes: &Shapes) -> Result<(), SurfaceError> {
        let (target, view, mut encoder) = self.start_rendering()?;
        self.render_rects(shapes, view, &mut encoder);
        self.finish_rendering(encoder, target);
        Ok(())
    }

    fn render_rects(&self, shapes: &Shapes, view: TextureView, encoder: &mut CommandEncoder) {
        let (instance_buffer, len) = self.upload_rects(shapes);
        let mut render_pass = Self::start_render_pass(&view, encoder);
        self.render_rect_instructions(&mut render_pass, &instance_buffer, len);
    }

    fn render_rect_instructions<'a>(
        &'a self,
        render_pass: &mut RenderPass<'a>,
        instance_buffer: &'a Buffer,
        len: usize,
    ) {
        render_pass.set_pipeline(&self.rect_pipeline);
        render_pass.set_vertex_buffer(0, self.rect_verts.slice(..));
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        render_pass.set_bind_group(0, self.target.surface_geometry_bind_group(), &[]);
        render_pass.draw(0..RECT_VERTS_LEN as _, 0..len as _);
    }

    fn upload_rects(&self, shapes: &Shapes) -> (Buffer, usize) {
        let contents = &shapes.rects;
        let buffer_desc = BufferInitDescriptor {
            label: Some("Node Geometry Buffer"),
            contents: bytemuck::cast_slice(contents),
            usage: BufferUsages::VERTEX,
        };
        let instance_buffer = self.device.device().create_buffer_init(&buffer_desc);
        (instance_buffer, contents.len())
    }

    fn start_rendering(
        &self,
    ) -> Result<(SurfaceTexture, TextureView, CommandEncoder), SurfaceError> {
        let target = self.target.surface().get_current_texture()?;
        let view_desc = TextureViewDescriptor {
            ..Default::default()
        };
        let view = target.texture.create_view(&view_desc);
        let encoder_desc = CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        };
        let encoder = self.device.device().create_command_encoder(&encoder_desc);
        Ok((target, view, encoder))
    }

    fn start_render_pass<'a>(
        view: &'a TextureView,
        encoder: &'a mut CommandEncoder,
    ) -> RenderPass<'a> {
        let render_pass_desc = RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(theme::BG),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        };
        encoder.begin_render_pass(&render_pass_desc)
    }

    fn finish_rendering(&self, encoder: CommandEncoder, target: SurfaceTexture) {
        self.device.queue().submit([encoder.finish()]);
        target.present();
    }
}
