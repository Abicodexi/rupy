use cgmath::{Matrix4, One, Quaternion, Rad, Rotation3, Vector3};

use crate::CacheKey;

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
    pub quat: Quaternion<f32>,
}

impl Rotation {
    pub fn from_euler(yaw: f32, pitch: f32, roll: f32) -> Self {
        let yaw = Rad(yaw);
        let pitch = Rad(pitch);
        let roll = Rad(roll);
        Self {
            quat: Quaternion::from_angle_y(yaw)
                * Quaternion::from_angle_x(pitch)
                * Quaternion::from_angle_z(roll),
        }
    }

    pub fn identity() -> Self {
        Self {
            quat: Quaternion::one(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Scale {
    pub value: Vector3<f32>,
}

impl Scale {
    pub fn uniform(s: f32) -> Self {
        Self {
            value: Vector3::new(s, s, s),
        }
    }

    pub fn one() -> Self {
        Self {
            value: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Transform {
    pub matrix: Matrix4<f32>,
}

impl Transform {
    pub fn from_components(pos: &Position, rot: &Rotation, scale: &Scale) -> Self {
        let translation = Matrix4::from_translation(Vector3::new(pos.x, pos.y, 0.0));
        let rotation = Matrix4::from(rot.quat);
        let scaling = Matrix4::from_nonuniform_scale(scale.value.x, scale.value.y, scale.value.z);

        Self {
            matrix: translation * rotation * scaling,
        }
    }
    pub fn data(&self) -> [[f32; 4]; 4] {
        self.matrix.into()
    }
}

#[derive(Clone)]
pub struct MeshInstance {
    pub mesh_key: CacheKey,
    pub material_key: CacheKey,
}

// Renderable (per entity)
//   ↳ Model (shared resource)
//      ↳ Mesh(es) (vertex/index buffers)
//      ↳ Material(s) (shader, textures, uniforms)
#[derive(Clone)]
pub struct Renderable {
    pub model_key: CacheKey,
    pub visible: bool,
}
