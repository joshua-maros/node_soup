use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, CommandEncoder, CommandEncoderDescriptor, LoadOp, Operations, RenderPass,
    RenderPassColorAttachment, RenderPassDescriptor, SurfaceError, SurfaceTexture, TextureView,
    TextureViewDescriptor,
};

use super::{
    clear::clear, render_icons::render_icons, render_rects::render_rects, render_text::render_text,
    ActiveRenderInfo, MutableResources, ReadOnlyResources, RenderEngine,
};
use crate::{
    renderer::{fonts::Fonts, shapes::Shapes, vertex_data::RECT_VERTS_LEN},
    theme::{self},
};

impl RenderEngine {
    pub fn render(&mut self, layers: &[&Shapes]) -> Result<(), SurfaceError> {
        let (target, view, mut encoder) = start_rendering(&self.ror)?;
        let mut info = ActiveRenderInfo {
            shapes: &Shapes::new(),
            view: &view,
            encoder: &mut encoder,
        };
        clear(&self.ror, &mut info);
        for shapes in layers {
            let mut info = ActiveRenderInfo {
                shapes,
                view: &view,
                encoder: &mut encoder,
            };
            render_rects(&self.ror, &mut info);
            render_text(&self.ror, &mut self.mr, &mut info);
            render_icons(&self.ror, &mut info);
        }
        finish_rendering(&self.ror, &mut self.mr, encoder, target);
        Ok(())
    }
}

fn start_rendering(
    ror: &ReadOnlyResources,
) -> Result<(SurfaceTexture, TextureView, CommandEncoder), SurfaceError> {
    let target = ror.target.surface().get_current_texture()?;
    let view_desc = TextureViewDescriptor {
        ..Default::default()
    };
    let view = target.texture.create_view(&view_desc);
    let encoder_desc = CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    };
    let encoder = ror.device.device().create_command_encoder(&encoder_desc);
    Ok((target, view, encoder))
}

fn finish_rendering(
    ror: &ReadOnlyResources,
    mr: &mut MutableResources,
    encoder: CommandEncoder,
    target: SurfaceTexture,
) {
    mr.staging_belt.finish();
    ror.device.queue().submit([encoder.finish()]);
    target.present();
    mr.staging_belt.recall();
}
