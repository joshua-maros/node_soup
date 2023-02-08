use wgpu::{
    Device, DeviceDescriptor, Features, Instance, Limits, Queue, RequestAdapterOptions, Surface,
    TextureFormat,
};

pub struct RenderDevice {
    device: Device,
    queue: Queue,
}

impl RenderDevice {
    pub async fn new_for_surface(instance: &Instance, surface: &Surface) -> (Self, TextureFormat) {
        let options = RequestAdapterOptions {
            compatible_surface: Some(surface),
            ..Default::default()
        };
        let adapter = instance.request_adapter(&options).await.unwrap();
        let (device, queue) = Self::create_device_and_queue(&adapter).await;
        (
            Self { device, queue },
            surface.get_supported_formats(&adapter)[0],
        )
    }

    async fn create_device_and_queue(adapter: &wgpu::Adapter) -> (Device, Queue) {
        let desc = DeviceDescriptor {
            features: Features::empty(),
            limits: Self::limits(),
            label: Some("UI Device"),
        };
        adapter.request_device(&desc, None).await.unwrap()
    }

    fn limits() -> Limits {
        if cfg!(target_arch = "wasm32") {
            Limits::downlevel_webgl2_defaults()
        } else {
            Limits::default()
        }
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }
}
