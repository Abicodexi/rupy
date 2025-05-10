// per-vertex inputs
struct VertexInput {
    @location(0) position:      vec3<f32>,
    @location(1) tex_coords:    vec2<f32>,
    @location(2) normal:        vec3<f32>,
    @location(3) tangent:       vec3<f32>,
    @location(4) bitangent:     vec3<f32>,

    // instance-data model matrix (packed as 4 vec4s)
    @location(5) model_0:       vec4<f32>,
    @location(6) model_1:       vec4<f32>,
    @location(7) model_2:       vec4<f32>,
    @location(8) model_3:       vec4<f32>,

    // instance-data normal matrix
    @location(9)  nrm_0:        vec3<f32>,
    @location(10) nrm_1:        vec3<f32>,
    @location(11) nrm_2:        vec3<f32>
}

// what we pass to the fragment
struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0)        uv:        vec2<f32>,
    @location(1)        world_pos: vec3<f32>,
    @location(2)        world_nrm: vec3<f32>,
    @location(3)        world_tan: vec3<f32>,
    @location(4)        world_bit: vec3<f32>
}

struct Camera {
    view_proj: mat4x4<f32>,
    inv_proj:  mat4x4<f32>,  
    inv_view:  mat4x4<f32>,
    world_pos: vec3<f32>,
    _pad:      f32,
};
@group(0) @binding(0) var<uniform> camera: Camera;

struct Light {
    position: vec3<f32>,
    color:    vec3<f32>
}
@group(0) @binding(1) var<uniform> light: Light;

@group(1) @binding(0) var env_map:    texture_cube<f32>;
@group(1) @binding(1) var env_sampler: sampler;

@group(2) @binding(0) var t_diffuse: texture_2d<f32>;
@group(2) @binding(1) var s_diffuse: sampler;
@group(2) @binding(2) var t_normal:  texture_2d<f32>;
@group(2) @binding(3) var s_normal:   sampler;



@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    // reconstruct model & normal matrices
    let model: mat4x4<f32> = mat4x4<f32>(
        in.model_0, in.model_1, in.model_2, in.model_3
    );
    let nrm_mat: mat3x3<f32> = mat3x3<f32>(
        in.nrm_0, in.nrm_1, in.nrm_2
    );

    // world‐space position
    let wp4: vec4<f32> = model * vec4<f32>(in.position, 1.0);

    var out: VertexOutput;
    out.clip_pos   = camera.view_proj * wp4;
    out.uv         = in.tex_coords;
    out.world_pos  = wp4.xyz;

    // world‐space TBN basis
    out.world_nrm  = normalize(nrm_mat * in.normal);
    out.world_tan  = normalize(nrm_mat * in.tangent);
    out.world_bit  = normalize(nrm_mat * in.bitangent);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // sample your diffuse + normal maps
    let albedo    = textureSample(t_diffuse, s_diffuse, in.uv).rgb;
    let nmap      = textureSample(t_normal,  s_normal,  in.uv).xyz * 2.0 - 1.0;

    // build TBN matrix in-frag for normal mapping
    let T = normalize(in.world_tan - dot(in.world_tan, in.world_nrm) * in.world_nrm);
    let B = cross(T, in.world_nrm);
    let N = normalize(mat3x3<f32>(T, B, in.world_nrm) * nmap);

    // lighting
    let L = normalize(light.position - in.world_pos);
    let V = normalize(camera.world_pos - in.world_pos);
    let H = normalize(L + V);

    let diff = max(dot(N, L), 0.0);
    let spec = pow(max(dot(N, H), 0.0), 32.0);

    let diffuse_col  = light.color * diff;
    let specular_col = light.color * spec;

    // environment reflection
    let reflect_dir  = reflect(-V, N);
    let env_col      = textureSample(env_map, env_sampler, reflect_dir).rgb;

    let shininess    = 0.1;
    let color        = (diffuse_col + specular_col) * albedo + env_col * shininess;

    return vec4<f32>(color, 1.0);
}