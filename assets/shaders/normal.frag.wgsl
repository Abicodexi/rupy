
struct Light {
    position: vec3<f32>,
    color:    vec3<f32>,
};
@group(0) @binding(1) var<uniform> light: Light;

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
