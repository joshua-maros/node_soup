use bytemuck::{Pod, Zeroable};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendState, Buffer, BufferBindingType, BufferUsages,
    ColorTargetState, ColorWrites, CommandEncoderDescriptor, Device, Face, FragmentState,
    FrontFace, Instance, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor,
    PolygonMode, PrimitiveState, PrimitiveTopology, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, Surface, SurfaceConfiguration,
    SurfaceError, TextureViewDescriptor, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexState, VertexStepMode,
};
use winit::{
    dpi::PhysicalSize,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use crate::constants::colors::{self, NODE_FILL, NODE_OUTLINE};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 2],
}

impl Vertex {
    pub fn desc() -> VertexBufferLayout<'static> {
        const ATTRS: [VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as _,
            step_mode: VertexStepMode::Vertex,
            attributes: &ATTRS,
        }
    }
}

const RECT_VERTS: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, 0.0],
    },
    Vertex {
        position: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0],
    },
    Vertex {
        position: [0.0, 1.0],
    },
];

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct RectInstance {
    position: [f32; 2],
    size: [f32; 2],
    fill_color: [f32; 3],
    border_color: [f32; 3],
    corner_radius: f32,
}

impl RectInstance {
    pub fn desc() -> VertexBufferLayout<'static> {
        const ATTRS: [VertexAttribute; 5] = wgpu::vertex_attr_array![
            1 => Float32x2,
            2 => Float32x2,
            3 => Float32x3,
            4 => Float32x3,
            5 => Float32
        ];
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as _,
            step_mode: VertexStepMode::Instance,
            attributes: &ATTRS,
        }
    }
}

const RECTS1: &[RectInstance] = &[
    RectInstance {
        position: [300.0, 100.0],
        size: [128.0, 128.0],
        fill_color: NODE_FILL,
        border_color: NODE_OUTLINE,
        corner_radius: 8.0,
    },
    RectInstance {
        position: [300.0 - 128.0 + 4.0, 100.0 - 24.0 - 4.0],
        size: [128.0 - 8.0, 128.0],
        fill_color: NODE_FILL,
        border_color: NODE_OUTLINE,
        corner_radius: 8.0,
    },
];

const RECTS2: &[RectInstance] = &[RectInstance {
    position: [300.0 - 128.0, 100.0 + 128.0 - 32.0],
    size: [128.0, 32.0],
    fill_color: NODE_FILL,
    border_color: NODE_OUTLINE,
    corner_radius: 8.0,
}];

const RECTS3: &[RectInstance] = &[RectInstance {
    position: [300.0, 100.0],
    size: [128.0, 128.0],
    fill_color: NODE_FILL,
    border_color: NODE_OUTLINE,
    corner_radius: 8.0,
}];

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ScreenUniformData {
    width: f32,
    height: f32,
}

pub struct RenderTarget {
    surface: Surface,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: PhysicalSize<u32>,
    window: Window,
    shape1_pipeline: RenderPipeline,
    shape2_pipeline: RenderPipeline,
    shape3_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    instance_buffer_1: Buffer,
    instance_buffer_2: Buffer,
    instance_buffer_3: Buffer,
    screen_uniform_buffer: Buffer,
    screen_bind_group: BindGroup,
}

impl RenderTarget {
    pub async fn new(event_loop: &EventLoop<()>) -> Self {
        let window = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(1280, 720))
            .build(event_loop)
            .unwrap();

        let size = window.inner_size();
        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .unwrap();
        let device_desc = wgpu::DeviceDescriptor {
            features: wgpu::Features::empty(),
            limits: if cfg!(target_arch = "wasm32") {
                wgpu::Limits::downlevel_webgl2_defaults()
            } else {
                wgpu::Limits::default()
            },
            label: None,
        };
        let (device, queue) = adapter.request_device(&device_desc, None).await.unwrap();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        let buffer_desc = BufferInitDescriptor {
            label: Some("Screen Uniform Buffer"),
            contents: bytemuck::cast_slice(&[ScreenUniformData {
                width: 1280.0,
                height: 720.0,
            }]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        };
        let screen_uniform_buffer = device.create_buffer_init(&buffer_desc);

        let layout_desc = BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("Screen Bind Group Layout"),
        };
        let screen_bind_group_layout = device.create_bind_group_layout(&layout_desc);
        let bind_group_desc = BindGroupDescriptor {
            layout: &screen_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: screen_uniform_buffer.as_entire_binding(),
            }],
            label: Some("Screen Bind Group"),
        };
        let screen_bind_group = device.create_bind_group(&bind_group_desc);

        let shader_desc = ShaderModuleDescriptor {
            label: Some("Basic Shader"),
            source: ShaderSource::Wgsl(include_str!("../shaders/shapes.wgsl").into()),
        };
        let shader = device.create_shader_module(shader_desc);
        let layout_desc = PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&screen_bind_group_layout],
            push_constant_ranges: &[],
        };
        let render_pipeline_layout = device.create_pipeline_layout(&layout_desc);
        let targets = [Some(ColorTargetState {
            format: config.format,
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: ColorWrites::ALL,
        })];
        let mut pipeline_desc = RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vertex_shader",
                buffers: &[Vertex::desc(), RectInstance::desc()],
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
        let shape1_pipeline = device.create_render_pipeline(&pipeline_desc);
        pipeline_desc.fragment.as_mut().unwrap().entry_point = "fragment_shader_2";
        let shape2_pipeline = device.create_render_pipeline(&pipeline_desc);
        pipeline_desc.fragment.as_mut().unwrap().entry_point = "fragment_shader_3";
        let shape3_pipeline = device.create_render_pipeline(&pipeline_desc);

        let buffer_desc = BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&RECT_VERTS),
            usage: BufferUsages::VERTEX,
        };
        let vertex_buffer = device.create_buffer_init(&buffer_desc);

        let buffer_desc = BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&RECTS1),
            usage: BufferUsages::VERTEX,
        };
        let instance_buffer_1 = device.create_buffer_init(&buffer_desc);

        let buffer_desc = BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&RECTS2),
            usage: BufferUsages::VERTEX,
        };
        let instance_buffer_2 = device.create_buffer_init(&buffer_desc);

        let buffer_desc = BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&RECTS3),
            usage: BufferUsages::VERTEX,
        };
        let instance_buffer_3 = device.create_buffer_init(&buffer_desc);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            shape1_pipeline,
            shape2_pipeline,
            shape3_pipeline,
            vertex_buffer,
            instance_buffer_1,
            instance_buffer_2,
            instance_buffer_3,
            screen_uniform_buffer,
            screen_bind_group,
        }
    }

    pub fn render(&self) -> Result<(), SurfaceError> {
        let target = self.surface.get_current_texture()?;
        let view_desc = TextureViewDescriptor {
            ..Default::default()
        };
        let view = target.texture.create_view(&view_desc);
        let encoder_desc = CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        };
        let mut encoder = self.device.create_command_encoder(&encoder_desc);
        let render_pass_desc = RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(colors::BG),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        };
        let mut render_pass = encoder.begin_render_pass(&render_pass_desc);
        render_pass.set_pipeline(&self.shape1_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer_1.slice(..));
        render_pass.set_bind_group(0, &self.screen_bind_group, &[]);
        render_pass.draw(0..RECT_VERTS.len() as _, 0..RECTS1.len() as _);

        render_pass.set_pipeline(&self.shape2_pipeline);
        render_pass.set_vertex_buffer(1, self.instance_buffer_2.slice(..));
        render_pass.draw(0..RECT_VERTS.len() as _, 0..RECTS2.len() as _);

        render_pass.set_pipeline(&self.shape3_pipeline);
        render_pass.set_vertex_buffer(1, self.instance_buffer_3.slice(..));
        render_pass.draw(0..RECT_VERTS.len() as _, 0..RECTS3.len() as _);

        drop(render_pass);
        self.queue.submit([encoder.finish()]);
        target.present();
        Ok(())
    }

    pub fn window_id(&self) -> winit::window::WindowId {
        self.window.id()
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw()
    }

    pub fn resize_surface(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn refresh_surface(&mut self) {
        self.resize_surface(self.size)
    }
}
