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

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) fill_color: vec3<f32>,
    @location(1) bottom_left_corner_center: vec2<f32>,
    @location(2) top_right_corner_center: vec2<f32>,
    @location(3) position: vec2<f32>,
    @location(4) corner_radius: f32,
};

@vertex
fn vertex_shader(vert: VertexInput, rect: RectInstance) -> VertexOutput {
    var out: VertexOutput;
    out.fill_color = rect.fill_color;
    out.position = vert.position * rect.size + rect.position;
    out.clip_position = vec4<f32>(out.position, 0.0, 1.0);
    out.bottom_left_corner_center = rect.position + rect.corner_radius;
    out.top_right_corner_center = rect.position + rect.size - rect.corner_radius;
    out.corner_radius = rect.corner_radius;
    return out;
}

@fragment
fn fragment_shader(in: VertexOutput) -> @location(0) vec4<f32> {
    var dist: f32;
    var dx: f32;
    var dy: f32;
    if in.position.x < in.bottom_left_corner_center.x {
        dx = in.bottom_left_corner_center.x - in.position.x;
    } else if in.position.x > in.top_right_corner_center.x {
        dx = in.position.x - in.top_right_corner_center.x;
    } else {
        dx = 0.0;
    }
    if in.position.y < in.bottom_left_corner_center.y {
        dy = in.bottom_left_corner_center.y - in.position.y;
    } else if in.position.y > in.top_right_corner_center.y {
        dy = in.position.y - in.top_right_corner_center.y;
    } else {
        dy = 0.0;
    }
    let radius = sqrt(dx * dx + dy * dy);
    if radius < in.corner_radius {
        return vec4<f32>(in.fill_color, 1.0);
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}
