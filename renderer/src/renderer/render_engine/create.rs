use std::num::NonZeroU32;

use theme::{PREVIEW_WIDGET_SIZE, PREVIEW_TEXTURE_SIZE};
use wgpu::{
    util::StagingBelt, AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry,
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
    Extent3d, FilterMode, ImageCopyTexture, ImageDataLayout, Origin3d, SamplerBindingType,
    SamplerDescriptor, ShaderStages, Texture, TextureAspect, TextureDescriptor, TextureDimension,
    TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor, TextureViewDimension,
};
use winit::window::Window;

use super::{MutableResources, ReadOnlyResources, RenderEngine};
use crate::renderer::{
    coordinates::Size,
    fonts::Fonts,
    icon_data::IconInstance,
    pipeline_util::{create_render_pipeline, create_shader},
    rect_data::RectInstance,
    render_device::RenderDevice,
    render_target::RenderTarget,
    vertex_data::{create_rect_verts_buffer, Vertex},
};

fn create_icon_texture(device: &RenderDevice) -> (BindGroupLayout, BindGroup) {
    let image = image::load_from_memory(include_bytes!("icons.png")).unwrap();
    let image = image.to_luma8();
    let texture_size = Extent3d {
        width: 2048,
        height: 2048,
        ..Default::default()
    };
    let desc = TextureDescriptor {
        label: Some("Icon Texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::R8Unorm,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
    };
    let texture = device.device().create_texture(&desc);
    device.queue().write_texture(
        ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        &image,
        ImageDataLayout {
            offset: 0,
            bytes_per_row: NonZeroU32::new(2048),
            rows_per_image: NonZeroU32::new(2048),
        },
        texture_size,
    );
    let texture_view = texture.create_view(&TextureViewDescriptor::default());
    let texture_sampler = device.device().create_sampler(&SamplerDescriptor {
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        address_mode_w: AddressMode::ClampToEdge,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Nearest,
        mipmap_filter: FilterMode::Nearest,
        ..Default::default()
    });
    let desc = BindGroupLayoutDescriptor {
        label: Some("Icon Texture Bind Group Layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
        ],
    };
    let texture_bind_group_layout = device.device().create_bind_group_layout(&desc);
    let desc = BindGroupDescriptor {
        label: Some("Icon Texture Bind Group"),
        layout: &texture_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&texture_view),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::Sampler(&texture_sampler),
            },
        ],
    };
    let texture_bind_group = device.device().create_bind_group(&desc);
    (texture_bind_group_layout, texture_bind_group)
}

const NUM_IMAGES: usize = 16;

fn create_image_textures(
    device: &RenderDevice,
) -> (BindGroupLayout, [(Texture, BindGroup); NUM_IMAGES]) {
    let texture_size = Extent3d {
        width: PREVIEW_TEXTURE_SIZE,
        height: PREVIEW_TEXTURE_SIZE,
        ..Default::default()
    };
    let desc = BindGroupLayoutDescriptor {
        label: Some("Icon Texture Bind Group Layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
        ],
    };
    let texture_bind_group_layout = device.device().create_bind_group_layout(&desc);

    let mut images = Vec::new();
    for image_index in 0..NUM_IMAGES {
        let name = format!("Image Texture #{}", image_index);
        let desc = TextureDescriptor {
            label: Some(&name),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        };
        let texture = device.device().create_texture(&desc);
        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        let texture_sampler = device.device().create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });
        let name = format!("Image Texture Bind Group #{}", image_index);
        let desc = BindGroupDescriptor {
            label: Some(&name),
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&texture_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&texture_sampler),
                },
            ],
        };
        let bind_group = device.device().create_bind_group(&desc);
        images.push((texture, bind_group));
    }
    (texture_bind_group_layout, images.try_into().unwrap())
}

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
        let (icon_texture_bind_group_layout, icon_texture_bind_group) =
            create_icon_texture(&device);
        let icon_shader = create_shader("Icon Shader", include_str!("icon_shader.wgsl"), &device);
        let icon_pipeline = create_render_pipeline(
            "Icon Pipeline",
            &icon_shader,
            &[
                target.surface_geometry_bind_group_layout(),
                &icon_texture_bind_group_layout,
            ],
            &[Vertex::desc(), IconInstance::desc()],
            &device,
            &target,
        );
        let image_shader =
            create_shader("Image Shader", include_str!("image_shader.wgsl"), &device);
        let (image_texture_bind_group_layout, image_textures) = create_image_textures(&device);
        let image_pipeline = create_render_pipeline(
            "Image Pipeline",
            &image_shader,
            &[
                target.surface_geometry_bind_group_layout(),
                &image_texture_bind_group_layout,
            ],
            &[Vertex::desc(), IconInstance::desc()],
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
                icon_pipeline,
                icon_texture_bind_group,
                image_pipeline,
                image_textures,
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

    pub fn target_size(&self) -> Size {
        self.ror.target.size()
    }
}
