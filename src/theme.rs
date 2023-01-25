use wgpu::Color;

pub const BG: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

pub const FILL_BRIGHTNESS: f32 = 0.01;
pub const NODE_FILL: [f32; 3] = [FILL_BRIGHTNESS, FILL_BRIGHTNESS, FILL_BRIGHTNESS];

pub const OUTLINE_BRIGHTNESS: f32 = 0.2;
pub const NODE_OUTLINE: [f32; 3] = [OUTLINE_BRIGHTNESS, OUTLINE_BRIGHTNESS, OUTLINE_BRIGHTNESS];

pub const NODE_BODY_WIDTH: f32 = 30.0;
pub const NODE_MIN_HEIGHT: f32 = 100.0;
pub const NODE_HEADER_HEIGHT: f32 = 10.0;
pub const NODE_OUTER_CORNER_SIZE: f32 = 6.0;
pub const NODE_INNER_CORNER_SIZE: f32 = 4.0;
pub const NODE_PADDING: f32 = 4.0;
