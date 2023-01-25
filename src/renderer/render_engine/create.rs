use winit::window::Window;

use super::RenderEngine;
use crate::renderer::{
    coordinates::Size,
    pipeline_util::{create_render_pipeline, create_shader},
    rect_data::RectInstance,
    render_target::RenderTarget,
    vertex_data::{create_rect_verts_buffer, Vertex},
};

impl RenderEngine {
    pub async fn new_for_window(window: &Window) -> Self {
        let (target, device) = RenderTarget::new_for_window(window).await;
        let rect_shader = create_shader("Rect Shader", include_str!("rect_shader.wgsl"), &device);
        let rect_verts = create_rect_verts_buffer(&device);
        let rect_pipeline = create_render_pipeline(
            "Rect Pipeline",
            &rect_shader,
            &[target.surface_geometry_bind_group_layout()],
            &[Vertex::desc(), RectInstance::desc()],
            &device,
            &target,
        );
        Self {
            device,
            target,
            rect_verts,
            rect_pipeline,
        }
    }

    pub fn resize_target(&mut self, new_size: Size) {
        self.target.resize(new_size, &self.device)
    }

    pub fn refresh_target(&mut self) {
        self.target.refresh(&self.device)
    }
}
