use bytemuck::{Pod, Zeroable};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, VertexAttribute, VertexBufferLayout, VertexStepMode,
};

use super::render_device::RenderDevice;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
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
pub const RECT_VERTS_LEN: usize = RECT_VERTS.len();

pub fn create_rect_verts_buffer(device: &RenderDevice) -> Buffer {
    let desc = BufferInitDescriptor {
        label: Some("Rectangle Vertices"),
        contents: bytemuck::cast_slice(RECT_VERTS),
        usage: BufferUsages::VERTEX,
    };
    device.device().create_buffer_init(&desc)
}
