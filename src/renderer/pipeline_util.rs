use wgpu::{
    BindGroupLayout, BlendState, ColorTargetState, ColorWrites, Face, FragmentState, FrontFace,
    MultisampleState, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology,
    RenderPipeline, RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource,
    VertexBufferLayout, VertexState,
};

use super::{render_device::RenderDevice, render_target::RenderTarget};

pub fn make_shader(label: &str, source: &str, device: &RenderDevice) -> ShaderModule {
    let desc = ShaderModuleDescriptor {
        label: Some(label),
        source: ShaderSource::Wgsl(source.into()),
    };
    device.device().create_shader_module(desc)
}

pub fn make_render_pipeline(
    label: &str,
    shader: &ShaderModule,
    bind_group_layouts: &[&BindGroupLayout],
    vertex_buffers: &[VertexBufferLayout<'static>],
    device: &RenderDevice,
    target: &RenderTarget,
) -> RenderPipeline {
    let layout_label = format!("{} Layout", label);
    let layout_desc = PipelineLayoutDescriptor {
        label: Some(&layout_label),
        bind_group_layouts,
        push_constant_ranges: &[],
    };
    let render_pipeline_layout = device.device().create_pipeline_layout(&layout_desc);
    let targets = [Some(ColorTargetState {
        format: target.format(),
        blend: Some(BlendState::ALPHA_BLENDING),
        write_mask: ColorWrites::ALL,
    })];
    let pipeline_desc = RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&render_pipeline_layout),
        vertex: VertexState {
            module: &shader,
            entry_point: "vertex_shader",
            buffers: &vertex_buffers,
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: "fragment_shader",
            targets: &targets,
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: Some(Face::Back),
            polygon_mode: PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    };
    device.device().create_render_pipeline(&pipeline_desc)
}
