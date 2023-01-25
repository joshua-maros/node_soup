struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct RectInstance {
    @location(1) position: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) fill_color: vec3<f32>,
    @location(4) outline_color: vec3<f32>,
    @location(5) border_modes: u32,
};

struct Screen {
    width: f32,
    height: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec2<f32>,
    @location(1) start: vec2<f32>,
    @location(2) end: vec2<f32>,
    @location(3) fill_color: vec3<f32>,
    @location(4) outline_color: vec3<f32>,
    @location(5) border_modes: u32,
};

@group(0) @binding(0)
var<uniform> screen: Screen;

@vertex
fn vertex_shader(vert: VertexInput, rect: RectInstance) -> VertexOutput {
    var out: VertexOutput;

    out.fill_color = rect.fill_color;
    out.outline_color = rect.outline_color;
    out.border_modes = rect.border_modes;

    out.position = vert.position * rect.size + rect.position;
    let x = out.position.x / screen.width * 2.0 - 1.0;
    let y = out.position.y / screen.height * 2.0 - 1.0;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);

    out.start = rect.position;
    out.end = rect.position + rect.size;

    return out;
}

@fragment
fn fragment_shader(in: VertexOutput) -> @location(0) vec4<f32> {
    let x = in.position.x;
    let y = in.position.y;

    let left_border_mode = in.border_modes & 0x3u;
    let right_border_mode = (in.border_modes >> 2u) & 0x3u;
    let bottom_border_mode = (in.border_modes >> 4u) & 0x1u;
    let top_border_mode = (in.border_modes >> 5u) & 0x1u;

    var left = in.start.x;
    var right = in.end.x;

    if left_border_mode == 2u {
        // Diagonal mode.
        left += y - in.start.y;
    } else if left_border_mode == 3u {
        // Antidiagonal mode.
        left += in.end.y - y - 1.0;
    }

    if right_border_mode == 2u {
        // Diagonal mode.
        right -= in.end.y - y;
    } else if right_border_mode == 3u {
        // Antidiagonal mode.
        right -= y - in.start.y;
    }

    let thickness = 1.0;

    if x >= left && x <= right {
        if (left_border_mode > 0u && x <= left + thickness)
            || (right_border_mode > 0u && x >= right - thickness)
            || (bottom_border_mode > 0u && y <= in.start.y + thickness)
            || (top_border_mode > 0u && y >= in.end.y - thickness) {
            return vec4<f32>(in.outline_color, 1.0);
        } else {
            return vec4<f32>(in.fill_color, 1.0);
        }
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}
