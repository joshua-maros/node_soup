use std::marker::PhantomData;

use bytemuck::{Pod, Zeroable};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, ShaderStages,
};

use super::render_device::RenderDevice;

pub struct UniformBuffer<T> {
    buffer: Buffer,
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
    data: PhantomData<T>,
}

impl<T: Pod + Zeroable> UniformBuffer<T> {
    pub fn new(label: &str, data: T, device: &RenderDevice) -> Self {
        let buffer = Self::buffer(label, data, device);
        let bind_group_layout = Self::create_bind_group_layout(label, device);
        let bind_group = Self::create_bind_group(label, &device, &bind_group_layout, &buffer);
        Self {
            buffer,
            bind_group_layout,
            bind_group,
            data: PhantomData,
        }
    }

    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn set(&mut self, data: T, device: &RenderDevice) {
        device
            .queue()
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[data]));
    }

    fn buffer(label: &str, data: T, device: &RenderDevice) -> Buffer {
        let data = [data];
        let label = format!("{} Uniform Buffer", label);
        let buffer_desc = BufferInitDescriptor {
            label: Some(&label),
            contents: bytemuck::cast_slice(&data),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        };
        device.device().create_buffer_init(&buffer_desc)
    }

    fn create_bind_group_layout(label: &str, device: &RenderDevice) -> BindGroupLayout {
        let label = format!("{} Bind Group Layout", label);
        let desc = BindGroupLayoutDescriptor {
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
            label: Some(&label),
        };
        device.device().create_bind_group_layout(&desc)
    }

    fn create_bind_group(
        label: &str,
        device: &RenderDevice,
        layout: &BindGroupLayout,
        buffer: &Buffer,
    ) -> BindGroup {
        let label = format!("{} Bind Group", label);
        let bind_group_desc = BindGroupDescriptor {
            layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some(&label),
        };
        device.device().create_bind_group(&bind_group_desc)
    }
}
