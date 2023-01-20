struct VertexData {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vertex_shader(@builtin(vertex_index) index: u32) -> VertexData {
    var vert: VertexData;
    let x = f32(1 - i32(index)) * 0.5;
    let y = f32(i32(index & 1u) * 2 - 1) * 0.5;
    vert.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    return vert;
}

@fragment
fn fragment_shader(in: VertexData) -> @location(0) vec4<f32> {
    return vec4<f32>(0.3, 0.2, 0.1, 1.0);
}
