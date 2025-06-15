struct VertexInput {
    @location(0) position: vec2<f32>,   
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) texture_index: i32,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) frag_uv: vec2<f32>,
    @location(1) frag_color: vec4<f32>,
    @location(2) @interpolate(flat) tex_index: i32, 
};

@binding(0) @group(0)
var<uniform> u_ortho: mat4x4<f32>;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let pos = vec4<f32>(input.position.xy, 0.0, 1.0);
    out.clip_pos = u_ortho * pos;
    out.frag_uv = input.uv;
    out.frag_color = input.color;
    out.tex_index = input.texture_index;
    return out;
}
