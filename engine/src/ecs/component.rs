use cgmath::SquareMatrix;

use crate::VertexInstance;

#[derive(Debug, Copy, Clone)]
pub struct Position {
    x: f32,
    y: f32,
    z: f32,
}
impl Position {
    pub fn update(&mut self, velocity: &crate::Velocity) {
        self.x += velocity.dx;
        self.y += velocity.dy;
        self.z += velocity.dz;
    }
}
impl From<(f32, f32, f32)> for Position {
    fn from(value: (f32, f32, f32)) -> Self {
        Self {
            x: value.0,
            y: value.1,
            z: value.2,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Velocity {
    dx: f32,
    dy: f32,
    dz: f32,
}

impl From<(f32, f32, f32)> for Velocity {
    fn from(value: (f32, f32, f32)) -> Self {
        Self {
            dx: value.0,
            dy: value.1,
            dz: value.2,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Rotation {
    quat: cgmath::Quaternion<f32>,
}

impl From<cgmath::Quaternion<f32>> for Rotation {
    fn from(value: cgmath::Quaternion<f32>) -> Self {
        Self { quat: value }
    }
}

impl From<cgmath::Deg<f32>> for Rotation {
    fn from(value: cgmath::Deg<f32>) -> Self {
        Rotation {
            quat: <cgmath::Quaternion<f32> as cgmath::Rotation3>::from_angle_z(value),
        }
    }
}

impl Rotation {
    pub fn update(&mut self, delta: cgmath::Quaternion<f32>) {
        self.quat = delta * self.quat
    }
    pub fn from_euler(yaw: f32, pitch: f32, roll: f32) -> Self {
        let yaw = cgmath::Rad(yaw);
        let pitch = cgmath::Rad(pitch);
        let roll = cgmath::Rad(roll);
        Self {
            quat: <cgmath::Quaternion<f32> as cgmath::Rotation3>::from_angle_y(yaw)
                * <cgmath::Quaternion<f32> as cgmath::Rotation3>::from_angle_x(pitch)
                * <cgmath::Quaternion<f32> as cgmath::Rotation3>::from_angle_z(roll),
        }
    }

    pub fn identity() -> Self {
        Self {
            quat: <cgmath::Quaternion<f32> as cgmath::One>::one(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Scale {
    pub value: cgmath::Vector3<f32>,
}

impl Scale {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            value: cgmath::Vector3 { x, y, z },
        }
    }
    pub fn uniform(s: f32) -> Self {
        Self {
            value: cgmath::Vector3::new(s, s, s),
        }
    }

    pub fn one() -> Self {
        Self {
            value: cgmath::Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

impl From<cgmath::Vector3<f32>> for Scale {
    fn from(value: cgmath::Vector3<f32>) -> Self {
        Self { value }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Transform {
    pub model_matrix: cgmath::Matrix4<f32>,
    pub normal_matrix: cgmath::Matrix4<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            model_matrix: cgmath::Matrix4::<f32>::identity(),
            normal_matrix: cgmath::Matrix4::<f32>::identity(),
        }
    }
}

impl Transform {
    pub fn from_components(pos: &Position, rot: &Rotation, scale: &Scale) -> Self {
        let translation =
            cgmath::Matrix4::from_translation(cgmath::Vector3::new(pos.x, pos.y, pos.z));
        let rotation = cgmath::Matrix4::from(rot.quat);
        let scaling =
            cgmath::Matrix4::from_nonuniform_scale(scale.value.x, scale.value.y, scale.value.z);
        let model_matrix = translation * rotation * scaling;
        let inv =
            cgmath::SquareMatrix::invert(&model_matrix).expect("model matrix was not invertible");

        // 2) transpose it
        let inv_transpose = cgmath::Matrix::transpose(&inv);

        // 3) extract the upper‐left 3×3 as your normal matrix
        let normal_matrix = cgmath::Transform::inverse_transform(&inv_transpose).unwrap();

        Self {
            model_matrix,
            normal_matrix,
        }
    }
    pub fn to_vertex_instance(&self, mat_id: u32) -> VertexInstance {
        let model = &self.model_matrix;
        let normal = &self.normal_matrix;

        let extract = |m: &cgmath::Matrix4<f32>| -> ([f32; 3], [f32; 3]) {
            ([m.x.x, m.x.y, m.x.z], [m.y.x, m.y.y, m.y.z])
        };
        let (nrm, tan) = extract(normal);
        let translation = {
            let w = model.w; // cgmath::Vector4
            [w.x, w.y, w.z]
        };

        VertexInstance {
            // 4×4 model matrix
            model: (*model).into(),

            // per-instance color override
            color: [1.0, 1.0, 1.0],
            _pad0: 0.0,

            // per-instance translation
            translation,
            _pad1: 0.0,

            // per-instance UV offset
            uv_offset: [0.0, 0.0],
            _pad2: [0.0; 2],

            // per-instance normals/tangents
            normal: nrm,
            _pad3: 0.0,
            tangent: tan,
            _pad4: 0.0,
            material_id: mat_id,
            _pad5: [0.0; 3],
        }
    }
}

// Renderable (per entity)
//   ↳ Model (shared resource)
//      ↳ Mesh(es) (vertex/index buffers)
//      ↳ Material(s) (shader, textures, uniforms)
#[derive(Debug, Clone)]
pub struct Renderable {
    pub model_key: crate::CacheKey,
    pub visible: bool,
}

impl From<crate::Entity> for Renderable {
    fn from(value: crate::Entity) -> Self {
        Self {
            model_key: value.into(),
            visible: true,
        }
    }
}

impl From<&crate::Entity> for Renderable {
    fn from(value: &crate::Entity) -> Self {
        Self {
            model_key: value.clone().into(),
            visible: true,
        }
    }
}
