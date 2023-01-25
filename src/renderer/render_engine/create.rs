use wgpu::util::StagingBelt;
use winit::window::Window;

use super::{MutableResources, ReadOnlyResources, RenderEngine};
use crate::renderer::{
    coordinates::Size,
    fonts::Fonts,
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
        let staging_belt = StagingBelt::new(1024);
        let fonts = Fonts::new(&device, &target);
        Self {
            ror: ReadOnlyResources {
                device,
                target,
                rect_verts,
                rect_pipeline,
            },
            mr: MutableResources {
                staging_belt,
                fonts,
            },
        }
    }

    pub fn resize_target(&mut self, new_size: Size) {
        self.ror.target.resize(new_size, &self.ror.device)
    }

    pub fn refresh_target(&mut self) {
        self.ror.target.refresh(&self.ror.device)
    }
}
