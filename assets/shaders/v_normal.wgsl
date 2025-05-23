// --------------------------------------------------
// Uniforms
// --------------------------------------------------

struct Camera {
    view_proj: mat4x4<f32>,
    inv_proj:  mat4x4<f32>,
    inv_view:  mat4x4<f32>,
    view_pos:  vec3<f32>,
};
@group(0) @binding(0) var<uniform> camera: Camera;

struct Light {
    position: vec3<f32>,
    color:    vec3<f32>,
};
@group(0) @binding(1) var<uniform> light: Light;

// --------------------------------------------------
// Vertex inputs
// --------------------------------------------------

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) normal: vec3<f32>,
    @location(4) tangent: vec3<f32>,
};
struct InstanceInput {
    @location(5)  model_0: vec4<f32>,
    @location(6)  model_1: vec4<f32>,
    @location(7)  model_2: vec4<f32>,
    @location(8)  model_3: vec4<f32>,
    @location(9)  color: vec3<f32>,
    @location(10) translation: vec3<f32>,
    @location(11) uv_offset: vec2<f32>,
    @location(12) normal: vec3<f32>,
    @location(13) tangent: vec3<f32>,
    @location(14) material_id: u32,
};

struct VertexOutput {
    @builtin(position) clip_position:      vec4<f32>,
    @location(0) tex_coords:        vec2<f32>,
    @location(1) world_position:    vec3<f32>,
    @location(2) world_view_pos:    vec3<f32>,
    @location(3) world_normal:      vec3<f32>,
    @location(4) world_tangent:     vec3<f32>,
    @location(5) tint_color:        vec3<f32>,
    @location(6) material_id:       u32,
};

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput
) -> VertexOutput {
    // Reconstruct matrices
    let model_matrix = mat4x4<f32>(
        instance.model_0,
        instance.model_1,
        instance.model_2,
        instance.model_3,
    );
    let normal_matrix = mat3x3<f32>(
        instance.tangent,
        cross(instance.normal, instance.tangent),
        instance.normal,
    );

    // World space position
    let world_pos4 = model_matrix * vec4<f32>(vertex.position, 1.0);
    let world_pos = world_pos4.xyz + instance.translation;

    // Transform normals and tangent
    let wn = normalize(normal_matrix * vertex.normal);
    let wt = normalize(normal_matrix * vertex.tangent);

    var out: VertexOutput;
    out.clip_position   = camera.view_proj * world_pos4;
    out.tex_coords      = vertex.tex_coords + instance.uv_offset;
    out.world_position  = world_pos;
    out.world_view_pos  = camera.view_pos;
    out.world_normal    = wn;
    out.world_tangent   = wt;
    out.tint_color      = vertex.color * instance.color;
    out.material_id     = instance.material_id;

    return out;
}

// --------------------------------------------------
// Fragment inputs & bindings
// --------------------------------------------------


@group(1) @binding(0) var env_map:    texture_cube<f32>;
@group(1) @binding(1) var env_samp:   sampler;

struct Material {
    ambient:   vec3<f32>,
    diffuse:   vec3<f32>,
    specular:  vec3<f32>,
    shininess: f32,
};
@group(2) @binding(0) var<storage, read> materials: array<Material>;


@group(3) @binding(0) var t_diffuse: texture_2d<f32>;
@group(3) @binding(1) var s_diffuse: sampler;
@group(3) @binding(2) var t_normal:  texture_2d<f32>;
@group(3) @binding(3) var s_normal:  sampler;



@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let material = materials[in.material_id];

    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let object_normal: vec4<f32> = textureSample(t_normal, s_normal, in.tex_coords);

    // TBN
    let world_tangent = normalize(in.world_tangent - dot(in.world_tangent, in.world_normal) * in.world_normal);
    let world_bitangent = cross(in.world_normal, world_tangent);
    let TBN = mat3x3(world_tangent, world_bitangent, in.world_normal);

    let tangent_normal = object_normal.xyz * 2.0 - 1.0;
    let world_normal = normalize(TBN * tangent_normal);

    let light_dir = normalize(light.position - in.world_position);
    let view_dir = normalize(in.world_view_pos - in.world_position);
    let half_dir = normalize(view_dir + light_dir);

    let diffuse_strength = max(dot(world_normal, light_dir), 0.0);
    let diffuse_color = material.diffuse * light.color * diffuse_strength;

    let specular_strength = pow(max(dot(world_normal, half_dir), 0.0), material.shininess);
    let specular_color =  material.specular * light.color * specular_strength;

    let world_reflect = reflect(-view_dir, world_normal);
    let reflection = textureSample(env_map, env_samp, world_reflect).rgb;

    let final_color = (diffuse_color + specular_color) * (object_color.xyz * in.tint_color.rgb) + reflection * material.shininess;

    return vec4<f32>(final_color, object_color.a);
}
