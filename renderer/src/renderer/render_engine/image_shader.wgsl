struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct ImageInstance {
    @location(1) position: vec2<f32>,
    @location(2) size: f32,
};

struct Screen {
    width: f32,
    height: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> screen: Screen;

@group(1) @binding(1)
var tex: texture_2d<f32>;
@group(1) @binding(2)
var sam: sampler;

@vertex
fn vertex_shader(vert: VertexInput, image: ImageInstance) -> VertexOutput {
    var out: VertexOutput;

    out.uv = vert.position.xy;
    let position = vert.position * image.size + image.position;
    let x = position.x / screen.width * 2.0 - 1.0;
    let y = position.y / screen.height * 2.0 - 1.0;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);

    return out;
}

@fragment
fn fragment_shader(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(tex, sam, in.uv);
}
