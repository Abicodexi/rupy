struct Uniform {
    view_proj: mat4x4<f32>,
    inv_proj: mat4x4<f32>,
    inv_view: mat4x4<f32>,
    world_pos: vec3<f32>,
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

    let plane_y = uniforms.world_pos.y - 0.1;
    let t1      = (plane_y - uniforms.world_pos.y) / world_dir.y;

    let center_clip     = vec4<f32>(0.0, 0.0, 1.0, 1.0);
    let center_view_h   = uniforms.inv_proj * center_clip;
    let center_view_dir = normalize(center_view_h.xyz / center_view_h.w);
    let center_world_dir= normalize((uniforms.inv_view * vec4(center_view_dir,0.0)).xyz);
    let t2              = (plane_y - uniforms.world_pos.y) / center_world_dir.y;
    let forward_hit     = uniforms.world_pos + center_world_dir * t2;

    let ndc = in.clip_position.xy / in.clip_position.w;          // in [-1,1]
    let uv  = ndc * 0.5 + vec2<f32>(0.5, 0.5);                  // in [0,1]

    let forward_clip = uniforms.view_proj * vec4<f32>(forward_hit, 1.0);
    let f_ndc        = forward_clip.xy / forward_clip.w;
    let f_uv         = f_ndc * 0.5 + vec2<f32>(0.5, 0.5);

    var scene_rgb = textureSample(env_map, env_sampler, world_dir).rgb;

    if (t1 > 0.0) {
        let hit1   = uniforms.world_pos + world_dir * t1;
        let h_fac  = clamp(uniforms.world_pos.y / 10.0, 0.0, 1.0);
        let radius = mix(0.0, 0.05, h_fac);
        let edge   = abs(distance(hit1.xz, uniforms.world_pos.xz) - radius);
        if (edge < 0.001) {
            let a = smoothstep(0.001, 0.0, edge);
            scene_rgb = mix(scene_rgb, vec3<f32>(1.0, 0.0, 0.0), a);
        }
    }

    let d_uv = distance(uv, f_uv);
    let marker_radius = 0.02;   // adjust to taste
    let marker_thick  = 0.005;
    if (d_uv < (marker_radius + marker_thick)) {
        let m = smoothstep(marker_radius + marker_thick, marker_radius, d_uv);
        scene_rgb = mix(scene_rgb, vec3<f32>(0.0, 1.0, 0.0), m);
    }

    return vec4<f32>(scene_rgb, 1.0);
}