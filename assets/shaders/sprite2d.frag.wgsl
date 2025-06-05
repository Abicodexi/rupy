@binding(0) @group(1)
var sprite_tex: texture_2d<f32>;
@binding(1) @group(1)
var sprite_sampler: sampler;

struct FragmentOutput {
    @location(0) color_out: vec4<f32>,
};

@fragment
fn fs_main(
    @location(0) frag_uv: vec2<f32>,
    @location(1) frag_color: vec4<f32>,
) -> FragmentOutput {
    let texel = textureSample(sprite_tex, sprite_sampler, frag_uv);
    return FragmentOutput(texel * frag_color);
}
