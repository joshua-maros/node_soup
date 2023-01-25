use bytemuck::{Pod, Zeroable};
use winit::dpi::PhysicalSize;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl From<PhysicalSize<u32>> for Size {
    fn from(other: PhysicalSize<u32>) -> Self {
        Self {
            width: other.width as _,
            height: other.height as _,
        }
    }
}

impl Size {
    pub fn zero() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
        }
    }

    pub fn componentwise_max(self, other: Self) -> Self {
        Self {
            width: self.width.max(other.width),
            height: self.height.max(other.height),
        }
    }
}
