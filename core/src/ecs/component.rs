#[derive(Copy, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Copy, Clone)]
pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
}

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
pub struct Transform {
    pub matrix: cgmath::Matrix4<f32>,
}

impl Transform {
    pub fn from_components(pos: &Position, rot: &Rotation, scale: &Scale) -> Self {
        let translation =
            cgmath::Matrix4::from_translation(cgmath::Vector3::new(pos.x, pos.y, 0.0));
        let rotation = cgmath::Matrix4::from(rot.quat);
        let scaling =
            cgmath::Matrix4::from_nonuniform_scale(scale.value.x, scale.value.y, scale.value.z);

        Self {
            matrix: translation * rotation * scaling,
        }
    }
    pub fn data(&self) -> [[f32; 4]; 4] {
        self.matrix.into()
    }
}

// Renderable (per entity)
//   ↳ Model (shared resource)
//      ↳ Mesh(es) (vertex/index buffers)
//      ↳ Material(s) (shader, textures, uniforms)
#[derive(Clone)]
pub struct Renderable {
    pub model_key: crate::CacheKey,
    pub visible: bool,
}
