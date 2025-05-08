use cgmath::SquareMatrix;

#[derive(Debug, Copy, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct Rotation {
    pub quat: cgmath::Quaternion<f32>,
}

impl Rotation {
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

#[derive(Debug, Copy, Clone)]
pub struct Transform {
    pub model_matrix: cgmath::Matrix4<f32>,
    pub normal_matrix: cgmath::Matrix4<f32>,
}
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformRaw {
    pub model_matrix: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 4]; 4],
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
            cgmath::Matrix4::from_translation(cgmath::Vector3::new(pos.x, pos.y, 0.0));
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
    pub fn data(&self) -> TransformRaw {
        TransformRaw {
            model_matrix: self.model_matrix.into(),
            normal_matrix: self.normal_matrix.into(),
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
