struct Camera {
    view_proj: mat4x4<f32>,
    inv_proj:  mat4x4<f32>,
    inv_view:  mat4x4<f32>,
    view_pos:  vec3<f32>,
    _pad:      f32,
};

@group(0) @binding(0) var<uniform> camera: Camera;


struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
}

@group(0) @binding(1) var<uniform> light: Light;


struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
};

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) normal_matrix_0: vec3<f32>,
    @location(10) normal_matrix_1: vec3<f32>,
    @location(11) normal_matrix_2: vec3<f32>,
    @location(12) color:          vec3<f32>,
    @location(13) position:          vec3<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) world_view_position: vec3<f32>,
    @location(3) world_light_position: vec3<f32>,
    @location(4) world_normal: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2,
    );

    let world_normal = normalize(normal_matrix * model.normal);
    let world_position = model_matrix * vec4<f32>(model.position, 1.0);

    var out: VertexOutput;
    out.clip_position = camera.view_proj * world_position;
    out.world_normal = world_normal;
    out.world_position = world_position.xyz;
    out.world_view_position = camera.view_pos.xyz;
    out.color = model.color;

    return out;
}



@group(1) @binding(0) var env_map:    texture_cube<f32>;
@group(1) @binding(1) var env_sampler: sampler;


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(light.position - in.world_position);
    let view_dir = normalize(in.world_view_position - in.world_position);
    let half_dir = normalize(view_dir + light_dir);

    let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse_color = light.color * diffuse_strength;

    let specular_strength = pow(max(dot(in.world_normal, half_dir), 0.0), 32.0);
    let specular_color = specular_strength * light.color;

   
    let world_reflect = reflect(-view_dir, in.world_normal);
    let reflection = textureSample(env_map, env_sampler, world_reflect).rgb;
    let shininess = 0.1;

    let final_color = (diffuse_color + specular_color) * in.color + reflection * shininess;

    return vec4<f32>(final_color, 1.0);
}