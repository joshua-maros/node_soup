use bytemuck::{Pod, Zeroable};
use wgpu::{VertexAttribute, VertexBufferLayout, VertexStepMode};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ImageInstance {
    pub position: [f32; 2],
    pub size: f32,
    pub index: i32,
}

impl ImageInstance {
    pub fn desc() -> VertexBufferLayout<'static> {
        const ATTRS: [VertexAttribute; 3] = wgpu::vertex_attr_array![
            1 => Float32x2,
            2 => Float32,
            3 => Uint32,
        ];
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as _,
            step_mode: VertexStepMode::Instance,
            attributes: &ATTRS,
        }
    }
}
