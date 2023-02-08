use wgpu::Color;

pub const BG: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

pub const FILL_BRIGHTNESS: f32 = 0.005;
pub const NODE_FILL: [f32; 3] = [FILL_BRIGHTNESS, FILL_BRIGHTNESS, FILL_BRIGHTNESS];

pub const OUTLINE_BRIGHTNESS: f32 = 0.1;
pub const NODE_OUTLINE: [f32; 3] = [OUTLINE_BRIGHTNESS, OUTLINE_BRIGHTNESS, OUTLINE_BRIGHTNESS];

pub const NODE_WIDTH: f32 = 140.0;
pub const NODE_HEIGHT: f32 = 24.0;
pub const NODE_CORNER_SIZE: f32 = 6.0;
pub const NODE_PARAMETER_PADDING: f32 = 2.0;
pub const NODE_LABEL_PADDING: f32 = 4.0;
pub const NODE_ICON_PADDING: f32 = NODE_LABEL_PADDING;
pub const NODE_ICON_SIZE: f32 = NODE_HEIGHT - 2.0 * NODE_ICON_PADDING;
pub const INTER_NODE_PADDING: f32 = 6.0;
pub const NODE_GUTTER_WIDTH: f32 = INTER_NODE_PADDING * 2.0;
pub const INTER_PANEL_PADDING: f32 = 18.0;

pub const PREVIEW_WIDGET_SIZE: f32 = 512.0;
pub const TOOL_BUTTON_SIZE: f32 = 48.0;
pub const TOOL_ICON_SIZE: f32 = 32.0;
pub const TOOL_BUTTON_PADDING: f32 = (TOOL_BUTTON_SIZE - TOOL_ICON_SIZE) / 2.0;

pub const NODE_LABEL_COLOR_BRIGHTNESS: f32 = 0.5;
pub const NODE_LABEL_COLOR_OPACITY: f32 = 1.0;
pub const NODE_LABEL_COLOR: [f32; 4] = [
    NODE_LABEL_COLOR_BRIGHTNESS,
    NODE_LABEL_COLOR_BRIGHTNESS,
    NODE_LABEL_COLOR_BRIGHTNESS,
    NODE_LABEL_COLOR_OPACITY,
];
pub const NODE_LABEL_SIZE: f32 = 16.0;

pub const BIG_VALUE_SIZE: f32 = 32.0;
pub const BIG_VALUE_BRIGHTNESS: f32 = 1.0;
pub const BIG_VALUE_COLOR: [f32; 4] = [
    BIG_VALUE_BRIGHTNESS,
    BIG_VALUE_BRIGHTNESS,
    BIG_VALUE_BRIGHTNESS,
    1.0,
];

macro_rules! outline_to_fill {
    ($OUTLINE_COLOR:expr) => {
        [
            0.1 * $OUTLINE_COLOR[0],
            0.1 * $OUTLINE_COLOR[1],
            0.1 * $OUTLINE_COLOR[2],
        ]
    };
}

pub const FLOAT_TYPE_OUTLINE_COLOR: [f32; 3] = [0.5, 0.5, 0.5];
pub const FLOAT_TYPE_FILL_COLOR: [f32; 3] = outline_to_fill!(FLOAT_TYPE_OUTLINE_COLOR);

pub const VECTOR_TYPE_OUTLINE_COLOR: [f32; 3] = [0.3, 0.0, 1.0];
pub const VECTOR_TYPE_FILL_COLOR: [f32; 3] = outline_to_fill!(VECTOR_TYPE_OUTLINE_COLOR);
