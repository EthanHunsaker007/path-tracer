struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let uv = vec2<f32>(f32(in_vertex_index & 2u), f32((in_vertex_index & 1u) << 1u));

    out.clip_position = vec4<f32>((uv * vec2(2.0, -2.0) + vec2(-1.0, 1.0)), 0.0, 1.0);
    out.uv = uv;

    return out;
}

@group(0) @binding(0)
var tex: texture_2d<f32>;
@group(0) @binding(1)
var samp: sampler;

@fragment
fn fs_main(
    @location(0) uv: vec2<f32>
) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, uv);
}