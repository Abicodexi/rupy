struct VertexInput {
    // per‐vertex
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,

    // per‐instance model matrix
    @location(5) model_0: vec4<f32>,
    @location(6) model_1: vec4<f32>,
    @location(7) model_2: vec4<f32>,
    @location(8) model_3: vec4<f32>,
    @location(9) normal_matrix_0: vec3<f32>,
    @location(10) normal_matrix_1: vec3<f32>,
    @location(11) normal_matrix_2: vec3<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec3<f32>,
    @location(3) bitangent: vec3<f32>,
    @location(4) world_normal: vec3<f32>,
    @location(5) world_position: vec3<f32>,
    @location(6) world_tangent: vec3<f32>,
    @location(7) world_bitangent: vec3<f32>,
    @location(8) world_position: vec3<f32>,
    @location(9) world_view_position: vec3<f32>,
};

struct Camera {
    view_proj: mat4x4<f32>,
    inv_proj: mat4x4<f32>,
    inv_view: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
}
@group(0) @binding(1)
var<uniform> light: Light;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@group(1) @binding(2)
var t_normal: texture_2d<f32>;
@group(1) @binding(3)
var s_normal: sampler;

@group(2)
@binding(0)
var env_map: texture_cube<f32>;
@group(2)
@binding(1)
var env_sampler: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let model = mat4x4<f32>(
        input.model_0, input.model_1, input.model_2, input.model_3
    );
    let normal_matrix = mat3x3<f32>(
        input.normal_matrix_0,
        input.normal_matrix_1,
        input.normal_matrix_2,
    );

    let world4 = model * vec4<f32>(input.position, 1.0);
    out.clip_position = camera.view_proj * world4;
    out.tex_coords    = input.tex_coords;
    out.normal        = normalize((model * vec4<f32>(input.normal, 0.0)).xyz);
    out.tangent       = normalize((model * vec4<f32>(input.tangent,0.0)).xyz);
    out.bitangent     = normalize((model * vec4<f32>(input.bitangent,0.0)).xyz);

    out.clip_position = camera.view_proj * world4;
    out.world_normal = normalize(normal_matrix * input.normal);
    out.world_tangent = normalize(normal_matrix * input.tangent);
    out.world_bitangent = normalize(normal_matrix * input.bitangent);
    out.world_position = world4.xyz;
    let camera_pos: vec3<f32> = (camera.inv_view * vec4<f32>(0.0, 0.0, 0.0, 1.0)).xyz;
    out.world_view_position = camera_pos;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // sample textures
    let object_color  : vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let object_normal : vec4<f32> = textureSample(t_normal,  s_normal,  in.tex_coords);

    // build TBN matrix
    let world_tangent   = normalize(in.world_tangent - dot(in.world_tangent, in.world_normal) * in.world_normal);
    let world_bitangent = cross(world_tangent, in.world_normal);
    let TBN = mat3x3<f32>(
        world_tangent,
        world_bitangent,
        in.world_normal
    );

    // decode normal map (from [0,1] → [-1,1]) and transform into world-space
    let tangent_normal = object_normal.xyz * 2.0 - 1.0;
    let world_normal   = normalize(TBN * tangent_normal);

    // lighting vectors
    let light_dir = normalize(light.position.xyz - in.world_position);
    let view_dir  = normalize(in.world_view_position - in.world_position);
    let half_dir  = normalize(view_dir + light_dir);

    // Blinn-Phong
    let NdotL = max(dot(world_normal, light_dir), 0.0);
    let diffuse_strength  = NdotL;
    let diffuse_color     = light.color.rgb * diffuse_strength;

    let spec_angle        = max(dot(world_normal, half_dir), 0.0);
    let specular_strength = pow(spec_angle, 32.0);
    let specular_color    = light.color.rgb * specular_strength;

    // reflection from a cubemap environment
    let reflect_dir = reflect(-view_dir, world_normal);
    let environment = textureSample(env_map, env_sampler, reflect_dir).rgb;

    // combine
    let shininess = 0.1;
    let lit = (diffuse_color + specular_color) * object_color.rgb
            + environment * shininess;

    // apply per-instance vertex color and preserve alpha
    let alpha = object_color.a;

    return vec4<f32>(lit, alpha);
}