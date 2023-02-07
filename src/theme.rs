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

pub const NODE_WIDTH: f32 = 140.0;
pub const NODE_HEIGHT: f32 = 24.0;
pub const NODE_GUTTER_WIDTH: f32 = 10.0;
pub const NODE_CORNER_SIZE: f32 = 6.0;
pub const NODE_PARAMETER_PADDING: f32 = 2.0;
pub const NODE_LABEL_PADDING: f32 = 4.0;
pub const INTER_NODE_PADDING: f32 = 6.0;
pub const INTER_PANEL_PADDING: f32 = 18.0;

pub const NODE_LABEL_COLOR_BRIGHTNESS: f32 = 1.0;
pub const NODE_LABEL_COLOR_OPACITY: f32 = 1.0;
pub const NODE_LABEL_COLOR: [f32; 4] = [
    NODE_LABEL_COLOR_BRIGHTNESS,
    NODE_LABEL_COLOR_BRIGHTNESS,
    NODE_LABEL_COLOR_BRIGHTNESS,
    NODE_LABEL_COLOR_OPACITY,
];
pub const NODE_LABEL_SIZE: f32 = 16.0;
