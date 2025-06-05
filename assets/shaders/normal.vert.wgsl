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
    @location(10) uv_offset: vec2<f32>,
    @location(11) normal: vec3<f32>,
    @location(12) tangent: vec3<f32>,
    @location(13) material_id: u32,
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
    let world_pos = world_pos4.xyz;

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

