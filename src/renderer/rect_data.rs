use bytemuck::{Pod, Zeroable};
use wgpu::{VertexBufferLayout, VertexAttribute, VertexStepMode};


#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct RectInstance {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub fill_color: [f32; 3],
    pub outline_color: [f32; 3],
    pub outline_modes: u32,
}

const OUTLINE_MODE_NONE: u32 = 0;
const OUTLINE_MODE_FLAT: u32 = 1;
const OUTLINE_MODE_DIAGONAL: u32 = 2;
const OUTLINE_MODE_ANTIDIAGONAL: u32 = 3;

pub const TOP_OUTLINE_NONE: u32 = OUTLINE_MODE_NONE << 0;
pub const TOP_OUTLINE_FLAT: u32 = OUTLINE_MODE_FLAT << 0;
pub const TOP_OUTLINE_DIAGONAL: u32 = OUTLINE_MODE_DIAGONAL << 0;
pub const TOP_OUTLINE_ANTIDIAGONAL: u32 = OUTLINE_MODE_ANTIDIAGONAL << 0;
pub const BOTTOM_OUTLINE_NONE: u32 = OUTLINE_MODE_NONE << 2;
pub const BOTTOM_OUTLINE_FLAT: u32 = OUTLINE_MODE_FLAT << 2;
pub const BOTTOM_OUTLINE_DIAGONAL: u32 = OUTLINE_MODE_DIAGONAL << 2;
pub const BOTTOM_OUTLINE_ANTIDIAGONAL: u32 = OUTLINE_MODE_ANTIDIAGONAL << 2;
pub const LEFT_OUTLINE_NONE: u32 = OUTLINE_MODE_NONE << 4;
pub const LEFT_OUTLINE_FLAT: u32 = OUTLINE_MODE_FLAT << 4;
pub const RIGHT_OUTLINE_NONE: u32 = OUTLINE_MODE_NONE << 5;
pub const RIGHT_OUTLINE_FLAT: u32 = OUTLINE_MODE_FLAT << 5;

impl RectInstance {
    pub fn desc() -> VertexBufferLayout<'static> {
        const ATTRS: [VertexAttribute; 5] = wgpu::vertex_attr_array![
            1 => Float32x2,
            2 => Float32x2,
            3 => Float32x3,
            4 => Float32x3,
            5 => Uint32,
        ];
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as _,
            step_mode: VertexStepMode::Instance,
            attributes: &ATTRS,
        }
    }
}
