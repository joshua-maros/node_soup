#[derive(Clone, Debug)]
pub struct Text {
    pub text: String,
    pub position: [f32; 2],
    pub bounds: [f32; 2],
    pub color: [f32; 4],
    pub size: f32,
}
