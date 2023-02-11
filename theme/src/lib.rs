//! All color values are linear unless otherwise noted.

pub const BG: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

pub const FILL_BRIGHTNESS: f32 = 0.005;
pub const NODE_FILL: [f32; 3] = [FILL_BRIGHTNESS, FILL_BRIGHTNESS, FILL_BRIGHTNESS];

pub const OUTLINE_BRIGHTNESS: f32 = 0.1;
pub const NODE_OUTLINE: [f32; 3] = [OUTLINE_BRIGHTNESS, OUTLINE_BRIGHTNESS, OUTLINE_BRIGHTNESS];

pub const NODE_WIDTH: f32 = 120.0;
pub const NODE_LABEL_HEIGHT: f32 = 24.0;
pub const NODE_CORNER_SIZE: f32 = 6.0;
pub const NODE_PARAMETER_PADDING: f32 = 2.0;
pub const NODE_LABEL_PADDING: f32 = 4.0;
pub const NODE_ICON_PADDING: f32 = NODE_LABEL_PADDING;
pub const NODE_ICON_SIZE: f32 = 16.0;
pub const INTER_NODE_PADDING: f32 = 6.0;
pub const NODE_GUTTER_WIDTH: f32 = INTER_NODE_PADDING * 2.0;
pub const INTER_PANEL_PADDING: f32 = 18.0;

pub const PREVIEW_WIDGET_SIZE: f32 = 360.0;
pub const TOOL_BUTTON_SIZE: f32 = 32.0;
pub const TOOL_ICON_SIZE: f32 = 24.0;
pub const TOOL_BUTTON_PADDING: f32 = (TOOL_BUTTON_SIZE - TOOL_ICON_SIZE) / 2.0;

pub const NODE_LABEL_COLOR_BRIGHTNESS_SRGB: f32 = 0.7;
pub const NODE_LABEL_COLOR_OPACITY: f32 = 1.0;
pub fn node_label_color() -> [f32; 4] {
    srgba_to_linear_rgba([
        NODE_LABEL_COLOR_BRIGHTNESS_SRGB,
        NODE_LABEL_COLOR_BRIGHTNESS_SRGB,
        NODE_LABEL_COLOR_BRIGHTNESS_SRGB,
        NODE_LABEL_COLOR_OPACITY,
    ])
}
pub const NODE_LABEL_SIZE: f32 = NODE_LABEL_HEIGHT - 2.0 * NODE_LABEL_PADDING;

pub const BIG_VALUE_SIZE: f32 = 32.0;
pub const BIG_VALUE_BRIGHTNESS_SRGB: f32 = 1.0;
pub fn big_value_color() -> [f32; 4] {
    srgba_to_linear_rgba([
        BIG_VALUE_BRIGHTNESS_SRGB,
        BIG_VALUE_BRIGHTNESS_SRGB,
        BIG_VALUE_BRIGHTNESS_SRGB,
        1.0,
    ])
}

macro_rules! hex_color {
    ($HEX_COLOR:expr) => {
        [
            (($HEX_COLOR >> 16) & 0xFF) as f32 / 255.0,
            (($HEX_COLOR >> 8) & 0xFF) as f32 / 255.0,
            (($HEX_COLOR >> 0) & 0xFF) as f32 / 255.0,
        ]
    };
}

pub fn srgb_transfer_function(x: f32) -> f32 {
    if x < 0.04045 {
        return x / 12.92;
    } else {
        return ((x + 0.055) / 1.055).powf(2.4);
    }
}

pub fn srgb_to_linear_rgb(srgb_color: [f32; 3]) -> [f32; 3] {
    [
        srgb_transfer_function(srgb_color[0]),
        srgb_transfer_function(srgb_color[1]),
        srgb_transfer_function(srgb_color[2]),
    ]
}

pub fn srgba_to_linear_rgba(srgba_color: [f32; 4]) -> [f32; 4] {
    [
        srgb_transfer_function(srgba_color[0]),
        srgb_transfer_function(srgba_color[1]),
        srgb_transfer_function(srgba_color[2]),
        // Alpha is usually encoded linearly, to avoid biasing towards the underlying color.
        srgba_color[3],
    ]
}

macro_rules! column_color {
    ($OUTLINE_COLOR:expr) => {
        [
            srgb_to_linear_rgb([
                0.2 * $OUTLINE_COLOR[0],
                0.2 * $OUTLINE_COLOR[1],
                0.2 * $OUTLINE_COLOR[2],
            ]),
            srgb_to_linear_rgb($OUTLINE_COLOR),
        ]
    };
}

pub fn column_colors() -> [[[f32; 3]; 2]; 6] {
    [
        column_color!([0.5, 0.5, 0.5]),
        column_color!(hex_color!(0x007BFF)),
        column_color!(hex_color!(0x8201D9)),
        column_color!(hex_color!(0xFF006E)),
        column_color!(hex_color!(0xFF5100)),
        column_color!(hex_color!(0xFFBE0B)),
    ]
}

// pub const INTEGER_TYPE_COLORS: [[f32; 3]; 2] = type_colors!([0.5, 1.0, 0.4]);
// pub const FLOAT_TYPE_COLORS: [[f32; 3]; 2] = type_colors!([0.5, 0.5, 0.5]);
// pub const STRING_TYPE_COLORS: [[f32; 3]; 2] = type_colors!([0.3, 0.5, 1.0]);
// pub const VECTOR_TYPE_COLORS: [[f32; 3]; 2] = type_colors!([0.3, 0.0, 1.0]);
