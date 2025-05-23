struct Uniform {
    view_proj: mat4x4<f32>,
    inv_proj: mat4x4<f32>,
    inv_view: mat4x4<f32>,
    view_pos: vec3<f32>,
    _pad:      f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniform;

@group(1)
@binding(0)
var env_map: texture_cube<f32>;
@group(1)
@binding(1)
var env_sampler: sampler;

struct VertexOutput {
    @builtin(position) frag_position: vec4<f32>,
    @location(0) clip_position: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) id: u32,
) -> VertexOutput {
    let uv = vec2<f32>(vec2<u32>(
        id & 1u,
        (id >> 1u) & 1u,
    ));
    var out: VertexOutput;
    out.clip_position = vec4(uv * 4.0 - 1.0, 1.0, 1.0);
    out.frag_position = out.clip_position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let view_pos_h = uniforms.inv_proj * in.clip_position;
    let view_dir   = normalize(view_pos_h.xyz / view_pos_h.w);
    let world_dir  = normalize((uniforms.inv_view * vec4(view_dir, 0.0)).xyz);
    var scene_rgb = textureSample(env_map, env_sampler, world_dir).rgb;
    return vec4<f32>(scene_rgb, 1.0);
}