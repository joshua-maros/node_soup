use wgpu::{
    Backends, Device, DeviceDescriptor, Features, Instance, Limits, Queue, RequestAdapterOptions,
    Surface, TextureFormat,
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
        let device_desc = DeviceDescriptor {
            features: Features::empty(),
            limits: if cfg!(target_arch = "wasm32") {
                Limits::downlevel_webgl2_defaults()
            } else {
                Limits::default()
            },
            label: Some("UI Device"),
        };
        let (device, queue) = adapter.request_device(&device_desc, None).await.unwrap();
        (Self { device, queue }, surface.get_supported_formats(&adapter)[0])
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }
}
