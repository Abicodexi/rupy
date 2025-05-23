

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

struct Debug {
    mode:  u32,
    pad0:  vec3<f32>,
    znear: f32,
    pad1:  vec3<f32>,
    zfar:  f32,
};

@group(0) @binding(2) var<uniform> debug: Debug;

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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Camera view direction (normalize view - position)
    let view_dir = normalize(in.world_view_pos - in.world_position);

    var out_color = vec4<f32>(in.tint_color.r, in.tint_color.g, in.tint_color.b, 1.0);

    switch debug.mode {
        case 0u: {
            let N = normalize(in.world_normal);
            let dNdx = dpdx(N);
            let dNdy = dpdy(N);
            let edge_strength = length(dNdx) + length(dNdy);
            let threshold = 0.2;

            var color: vec4<f32>;
            if (edge_strength > threshold) {
                color = vec4<f32>(0.0, 0.0, 0.0, 1.0); // Border: black
            } else {
                color = vec4<f32>(1.0, 1.0, 1.0, 1.0); // Fill: white
            }
            out_color = color;
        }
        case 1u: {
            // Normals (world space), mapped from [-1,1] to [0,1]
            out_color = vec4<f32>(normalize(in.world_normal) * 0.5 + 0.5, 1.0);
        }
        case 2u: {
            // Tangents (world space)
            out_color = vec4<f32>(normalize(in.world_tangent) * 0.5 + 0.5, 1.0);
        }
        case 3u: {
            // Camera view direction (at pixel)
            out_color = vec4<f32>(view_dir * 0.5 + 0.5, 1.0);
        }
        case 4u: {
            let ndc_z = in.clip_position.z / in.clip_position.w; // [-1, 1]
            let eye_z = (2.0 * debug.znear * debug.zfar) / (debug.zfar + debug.znear - ndc_z * (debug.zfar - debug.znear)); // linear eye-space z
            let linear_depth = (eye_z - debug.znear) / (debug.zfar - debug.znear); // [0, 1]
            out_color = vec4<f32>(linear_depth, linear_depth, linear_depth, 1.0);
        }
        case 5u: {
            // UV debug (repeated every 1.0)
            let uv = fract(in.tex_coords);
            out_color = vec4<f32>(uv, 0.0, 1.0);
        }
        case 6u: {
            let mid = f32(in.material_id) / 16.0; // assuming <=16 materials
            out_color = vec4<f32>(mid, 1.0 - mid, 0.3 + 0.7 * mid, 1.0);
        }
        default: {
            out_color = vec4<f32>(1.0, 0.0, 1.0, 1.0); // magenta, error
        }
    }
    return out_color;
}
