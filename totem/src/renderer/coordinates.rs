use std::ops::{Add, Sub};

use bytemuck::{Pod, Zeroable};
use winit::dpi::PhysicalSize;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}
impl Position {
    pub fn componentwise_min(&self, other: Position) -> Position {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }

    pub fn componentwise_max(&self, other: Position) -> Position {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }

    pub fn zero() -> Position {
        Self { x: 0.0, y: 0.0 }
    }
}

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

    pub fn is_negative(&self) -> bool {
        self.width < 0.0 || self.height < 0.0
    }
}

impl Add<Size> for Position {
    type Output = Position;

    fn add(self, rhs: Size) -> Self::Output {
        Position {
            x: self.x + rhs.width,
            y: self.y + rhs.height,
        }
    }
}

impl Sub for Position {
    type Output = Size;

    fn sub(self, rhs: Position) -> Self::Output {
        assert!(self.x >= rhs.x);
        assert!(self.y >= rhs.y);
        Size {
            width: self.x - rhs.x,
            height: self.y - rhs.y,
        }
    }
}
