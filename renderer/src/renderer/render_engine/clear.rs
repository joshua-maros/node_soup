use theme::BG;
use wgpu::{
    Color, LoadOp, Operations, RenderPass, RenderPassColorAttachment, RenderPassDescriptor,
};

use super::{ActiveRenderInfo, ReadOnlyResources};

const BG_WGPU: Color = Color {
    r: BG[0] as f64,
    g: BG[1] as f64,
    b: BG[2] as f64,
    a: BG[3] as f64,
};

pub(super) fn clear(_ror: &ReadOnlyResources, info: &mut ActiveRenderInfo) {
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
                load: LoadOp::Clear(BG_WGPU),
                store: true,
            },
        })],
        depth_stencil_attachment: None,
    };
    info.encoder.begin_render_pass(&render_pass_desc)
}
