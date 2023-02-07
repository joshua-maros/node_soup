struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct IconInstance {
    @location(1) position: vec2<f32>,
    @location(2) size: f32,
    @location(3) index: u32,
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
var icon_texture: texture_2d<f32>;
@group(1) @binding(2)
var icon_sampler: sampler;

@vertex
fn vertex_shader(vert: VertexInput, icon: IconInstance) -> VertexOutput {
    var out: VertexOutput;

    let icons_per_row = 2048u / 64u;
    let uv = vec2<f32>(
        vert.position.x + f32(icon.index % icons_per_row),
        1.0 - vert.position.y + f32(icon.index / icons_per_row),
    );
    out.uv = uv / f32(icons_per_row);
    let position = vert.position * icon.size + icon.position;
    let x = position.x / screen.width * 2.0 - 1.0;
    let y = position.y / screen.height * 2.0 - 1.0;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);

    return out;
}

@fragment
fn fragment_shader(in: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = textureSample(icon_texture, icon_sampler, in.uv).r;
    return vec4<f32>(1.0, 1.0, 1.0, alpha);
}
