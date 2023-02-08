use bytemuck::{Pod, Zeroable};
use wgpu::{
    Backends, BindGroup, BindGroupLayout, Instance, Surface, SurfaceConfiguration, TextureFormat,
};
use winit::window::Window;

use super::{coordinates::Size, render_device::RenderDevice, uniform_buffer::UniformBuffer};

pub struct RenderTarget {
    surface: Surface,
    config: SurfaceConfiguration,
    surface_geometry_buffer: UniformBuffer<SurfaceGeometry>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SurfaceGeometry {
    size: Size,
}

impl RenderTarget {
    pub async fn new_for_window(window: &Window) -> (Self, RenderDevice) {
        let instance = Instance::new(Backends::all());
        let (surface, config, device) = Self::create_surface_from_window(&instance, window).await;
        (Self::new_for_surface(surface, config, &device), device)
    }

    pub fn surface(&self) -> &Surface {
        &self.surface
    }

    pub fn format(&self) -> TextureFormat {
        self.config.format
    }

    pub fn surface_geometry_bind_group_layout(&self) -> &BindGroupLayout {
        self.surface_geometry_buffer.bind_group_layout()
    }

    pub fn surface_geometry_bind_group(&self) -> &BindGroup {
        self.surface_geometry_buffer.bind_group()
    }

    pub fn resize(&mut self, new_size: Size, device: &RenderDevice) {
        self.config.width = new_size.width as u32;
        self.config.height = new_size.height as u32;
        self.surface_geometry_buffer
            .set(Self::surface_geometry(&self.config), device);
        self.surface.configure(device.device(), &self.config);
    }

    pub fn refresh(&mut self, device: &RenderDevice) {
        self.resize(Self::surface_geometry(&self.config).size, device);
    }

    async fn create_surface_from_window(
        instance: &Instance,
        window: &Window,
    ) -> (Surface, SurfaceConfiguration, RenderDevice) {
        let surface = unsafe { instance.create_surface(window) };
        let (device, format) = RenderDevice::new_for_surface(&instance, &surface).await;

        let config = Self::config_from_window(format, window);
        surface.configure(device.device(), &config);

        (surface, config, device)
    }

    fn config_from_window(format: TextureFormat, window: &Window) -> SurfaceConfiguration {
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        config
    }

    pub fn new_for_surface(
        surface: Surface,
        config: SurfaceConfiguration,
        device: &RenderDevice,
    ) -> Self {
        let surface_geometry_buffer = Self::surface_geometry_buffer(&config, device);

        Self {
            surface,
            config,
            surface_geometry_buffer,
        }
    }

    fn surface_geometry(config: &SurfaceConfiguration) -> SurfaceGeometry {
        SurfaceGeometry {
            size: Size {
                width: config.width as _,
                height: config.height as _,
            },
        }
    }

    fn surface_geometry_buffer(
        config: &SurfaceConfiguration,
        device: &RenderDevice,
    ) -> UniformBuffer<SurfaceGeometry> {
        UniformBuffer::new("Surface Geometry", Self::surface_geometry(config), device)
    }

    pub(crate) fn size(&self) -> Size {
        Self::surface_geometry(&self.config).size
    }
}
