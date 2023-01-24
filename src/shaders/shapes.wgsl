struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct RectInstance {
    @location(1) position: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) fill_color: vec3<f32>,
    @location(4) outline_color: vec3<f32>,
    @location(5) corner_radius: f32,
};

struct Screen {
    width: f32,
    height: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec2<f32>,
    @location(1) fill_color: vec3<f32>,
    @location(2) outline_color: vec3<f32>,
    @location(3) corner_radius: f32,
    @location(4) bottom_left_corner_center: vec2<f32>,
    @location(5) top_right_corner_center: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> screen: Screen;

@vertex
fn vertex_shader(vert: VertexInput, rect: RectInstance) -> VertexOutput {
    var out: VertexOutput;

    out.fill_color = rect.fill_color;
    out.outline_color = rect.outline_color;
    out.corner_radius = rect.corner_radius;

    out.position = vert.position * rect.size + rect.position;
    let x = out.position.x / screen.width * 2.0 - 1.0;
    let y = out.position.y / screen.height * 2.0 - 1.0;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);

    out.bottom_left_corner_center = rect.position + rect.corner_radius;
    out.top_right_corner_center = rect.position + rect.size - rect.corner_radius;

    return out;
}

@fragment
fn fragment_shader(in: VertexOutput) -> @location(0) vec4<f32> {
    let start = in.bottom_left_corner_center - in.corner_radius;
    let end = in.top_right_corner_center + in.corner_radius;
    let x = in.position.x;
    let y = in.position.y;
    let thickness = 1.0;
    if y >= in.bottom_left_corner_center.y && y <= in.top_right_corner_center.y {
        if x <= start.x + thickness || x >= end.x - thickness {
            return vec4<f32>(in.outline_color, 1.0);
        }
        return vec4<f32>(in.fill_color, 1.0);
    } else if x >= in.bottom_left_corner_center.x && x <= in.top_right_corner_center.x {
        if y <= start.y + thickness || y >= end.y - thickness {
            return vec4<f32>(in.outline_color, 1.0);
        }
        return vec4<f32>(in.fill_color, 1.0);
    } else {
        let dx = select(in.bottom_left_corner_center.x - x, x - in.top_right_corner_center.x, x > in.bottom_left_corner_center.x);
        let dy = select(in.bottom_left_corner_center.y - y, y - in.top_right_corner_center.y, y > in.bottom_left_corner_center.y);
        let d = abs(dx) + abs(dy);
        if d <= in.corner_radius {
            if d >= in.corner_radius - thickness {
                return vec4<f32>(in.outline_color, 1.0);
            }
            return vec4<f32>(in.fill_color, 1.0);
        }
    }
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}

@fragment
fn fragment_shader_2(in: VertexOutput) -> @location(0) vec4<f32> {
    let x = in.position.x;
    let y = in.position.y;
    var dy: f32;
    if x < in.bottom_left_corner_center.x {
        dy = x - in.bottom_left_corner_center.x + in.corner_radius;
    } else if x > in.top_right_corner_center.x {
        dy = in.top_right_corner_center.x + in.corner_radius - x;
    } else {
        dy = in.corner_radius;
    }
    let top = in.top_right_corner_center.y + dy;
    let bottom = in.bottom_left_corner_center.y - in.corner_radius + dy;
    let thickness = 1.0;
    if y >= bottom && y <= top {
        if y <= bottom + thickness || y >= top - thickness 
            || x <= in.bottom_left_corner_center.x - in.corner_radius + thickness
            || x >= in.top_right_corner_center.x + in.corner_radius - thickness
        {
            return vec4<f32>(in.outline_color, 1.0);
        }
        return vec4<f32>(in.fill_color, 1.0);
    }
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}

@fragment
fn fragment_shader_3(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}
