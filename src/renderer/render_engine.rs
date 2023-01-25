use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroupLayout, BlendState, Buffer, BufferUsages, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, Face, FragmentState, FrontFace, LoadOp, MultisampleState, Operations,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    ShaderModel, ShaderModule, ShaderModuleDescriptor, ShaderSource, SurfaceError,
    TextureViewDescriptor, VertexBufferLayout, VertexState,
};
use winit::window::Window;

use super::{
    pipeline_util::{make_render_pipeline, make_shader},
    rect_data::RectInstance,
    render_device::RenderDevice,
    render_target::RenderTarget,
    size::Size,
    vertex_data::{rect_verts_buffer, Vertex, RECT_VERTS_LEN},
};
use crate::{
    constants::{self},
    visuals::Shapes,
};

pub struct RenderEngine {
    device: RenderDevice,
    target: RenderTarget,
    rect_verts: Buffer,
    main_pipeline: RenderPipeline,
}

impl RenderEngine {
    pub async fn new_for_window(window: &Window) -> Self {
        let (target, device) = RenderTarget::new_for_window(window).await;
        let main_shader = make_shader(
            "Main Shader",
            include_str!("../shaders/shapes.wgsl"),
            &device,
        );
        let rect_verts = rect_verts_buffer(&device);
        let main_pipeline = make_render_pipeline(
            "Main Pipeline",
            &main_shader,
            &[target.surface_geometry_bind_group_layout()],
            &[Vertex::desc(), RectInstance::desc()],
            &device,
            &target,
        );
        Self {
            device,
            target,
            rect_verts,
            main_pipeline,
        }
    }

    pub fn resize_target(&mut self, new_size: Size) {
        self.target.resize(new_size, &self.device)
    }

    pub fn refresh_target(&mut self) {
        self.target.refresh(&self.device)
    }

    pub fn render_shapes(&self, shapes: &Shapes) -> Result<(), SurfaceError> {
        let target = self.target.surface().get_current_texture()?;
        let view_desc = TextureViewDescriptor {
            ..Default::default()
        };
        let view = target.texture.create_view(&view_desc);
        let encoder_desc = CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        };
        let mut encoder = self.device.device().create_command_encoder(&encoder_desc);

        let contents = &shapes.rects;
        let buffer_desc = BufferInitDescriptor {
            label: Some("Node Geometry Buffer"),
            contents: bytemuck::cast_slice(contents),
            usage: BufferUsages::VERTEX,
        };
        let instance_buffer = self.device.device().create_buffer_init(&buffer_desc);

        let render_pass_desc = RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(constants::BG),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        };
        let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

        render_pass.set_pipeline(&self.main_pipeline);
        render_pass.set_vertex_buffer(0, self.rect_verts.slice(..));
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        render_pass.set_bind_group(0, self.target.surface_geometry_bind_group(), &[]);
        render_pass.draw(0..RECT_VERTS_LEN as _, 0..contents.len() as _);

        drop(render_pass);
        self.device.queue().submit([encoder.finish()]);
        target.present();
        Ok(())
    }
}
