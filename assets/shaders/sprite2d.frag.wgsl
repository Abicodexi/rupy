@group(1) @binding(0)
var sprite_tex: texture_2d_array<f32>;

@group(1) @binding(1)
var sprite_sampler: sampler;

struct FragmentInput {
    @location(0) frag_uv: vec2<f32>,
    @location(1) frag_color: vec4<f32>,
    @location(2) @interpolate(flat) tex_index: i32,
};

struct FragmentOutput {
    @location(0) color_out: vec4<f32>,
};

@fragment
fn fs_main(input: FragmentInput) -> FragmentOutput {
    let uv_layer = vec3<f32>(input.frag_uv, f32(input.tex_index));
    let texel = textureSampleLevel(sprite_tex, sprite_sampler, input.frag_uv, input.tex_index, 0.0);
    return FragmentOutput(texel * input.frag_color);
}
