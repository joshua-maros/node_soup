mod clear;
mod create;
mod render;
mod render_icons;
mod render_images;
mod render_rects;
mod render_text;

use std::num::NonZeroU32;

use wgpu::{
    util::StagingBelt, BindGroup, Buffer, CommandEncoder, ImageCopyTexture, ImageDataLayout,
    Origin3d, RenderPipeline, Texture, TextureAspect, TextureView,
};

use super::{fonts::Fonts, render_device::RenderDevice, render_target::RenderTarget, Shapes};

struct ReadOnlyResources {
    device: RenderDevice,
    target: RenderTarget,
    rect_verts: Buffer,
    rect_pipeline: RenderPipeline,
    icon_pipeline: RenderPipeline,
    icon_texture_bind_group: BindGroup,
    image_pipeline: RenderPipeline,
    image_textures: [(Texture, BindGroup); 16],
}

struct MutableResources {
    staging_belt: StagingBelt,
    fonts: Fonts,
}

struct ActiveRenderInfo<'a> {
    shapes: &'a Shapes,
    view: &'a TextureView,
    encoder: &'a mut CommandEncoder,
}

pub struct RenderEngine {
    ror: ReadOnlyResources,
    mr: MutableResources,
}

impl RenderEngine {
    // Data is assumed to be in sRGB format.
    pub fn upload_image(&self, index: usize, data: &[[u8; 4]]) {
        self.ror.device.queue().write_texture(
            ImageCopyTexture {
                texture: &self.ror.image_textures[index].0,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            bytemuck::cast_slice(data),
            ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * 360),
                rows_per_image: NonZeroU32::new(360),
                ..Default::default()
            },
            wgpu::Extent3d {
                width: 360,
                height: 360,
                depth_or_array_layers: 1,
            },
        );
    }
}
