pub mod colors {
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
}
