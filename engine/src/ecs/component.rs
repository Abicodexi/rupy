use std::collections::HashMap;

use crate::{CacheKey, VertexInstance};

use super::Entity;

use glam::{Mat4, Quat, Vec3};

#[derive(Debug, Copy, Clone)]
pub struct Position(pub Vec3);

impl Position {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Position(Vec3::new(x, y, z))
    }
    pub fn update(&mut self, velocity: &Velocity) {
        self.0 += velocity.0;
    }
    pub fn to_vec3(&self) -> Vec3 {
        self.0
    }
    pub fn origin() -> Self {
        Position::new(0.0, 0.0, 0.0)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Velocity(pub Vec3);

impl From<(f32, f32, f32)> for Velocity {
    fn from(value: (f32, f32, f32)) -> Self {
        Velocity(Vec3::new(value.0, value.1, value.2))
    }
}
impl From<Vec3> for Velocity {
    fn from(value: Vec3) -> Self {
        Velocity(value)
    }
}
#[derive(Debug, Copy, Clone)]
pub struct Rotation(pub Quat);

impl From<Quat> for Rotation {
    fn from(value: Quat) -> Self {
        Self(value)
    }
}
impl From<[f32; 4]> for Rotation {
    fn from(value: [f32; 4]) -> Self {
        Self::from(Quat::from_array(value))
    }
}
impl From<Vec3> for Rotation {
    fn from(value: Vec3) -> Self {
        Rotation::from(Quat::from_array([value.x, value.y, value.z, 0.0]))
    }
}

impl Rotation {
    pub fn update(&mut self, delta: Quat) {
        self.0 = delta * self.0;
    }
    pub fn from_euler(yaw: f32, pitch: f32, roll: f32) -> Self {
        // Note: glam expects (yaw, pitch, roll) in radians
        Self(Quat::from_euler(glam::EulerRot::YXZ, yaw, pitch, roll))
    }
    pub fn quat(&self) -> Quat {
        self.0
    }
    pub fn zero() -> Self {
        Self(Quat::from_array([0.0, 0.0, 0.0, 0.0]))
    }
    /// Returns a rotation that makes the model's forward (+Z) face the -Z direction in world space.
    /// Optionally takes an up vector (default Y).
    pub fn face_neg_z(up: Vec3) -> Self {
        // +Z (model) to -Z (world): so, look from origin toward -Z.
        let rot = Quat::from_mat4(&Mat4::look_at_lh(Vec3::ZERO, -Vec3::Z, up).inverse());
        Self(rot)
    }

    /// Returns a rotation that makes the model's forward (+Z) face the +Z direction in world space.
    /// Optionally takes an up vector (default Y).
    pub fn face_pos_z(up: Vec3) -> Self {
        // +Z (model) to +Z (world): look from origin toward +Z.
        let rot = Quat::from_mat4(&Mat4::look_at_lh(Vec3::ZERO, Vec3::Z, up).inverse());
        Self(rot)
    }

    /// Defaults with Y up, for convenience
    pub fn face_neg_z_y_up() -> Self {
        Self::face_neg_z(Vec3::Y)
    }
    pub fn face_pos_z_y_up() -> Self {
        Self::face_pos_z(Vec3::Y)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Scale(pub Vec3);

impl Scale {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Scale(Vec3::new(x, y, z))
    }
    pub fn uniform(s: f32) -> Self {
        Scale(Vec3::splat(s))
    }
    pub fn one() -> Self {
        Scale(Vec3::ONE)
    }
    pub fn zero() -> Self {
        Scale(Vec3::ZERO)
    }
}
impl From<Vec3> for Scale {
    fn from(value: Vec3) -> Self {
        Self(value)
    }
}
#[derive(Debug, Copy, Clone)]
pub struct Transform {
    pub model_matrix: glam::Mat4,
    pub normal_matrix: glam::Mat4,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            model_matrix: glam::Mat4::IDENTITY,
            normal_matrix: glam::Mat4::IDENTITY,
        }
    }
}

impl Transform {
    pub fn from_components(pos: &Position, rot: &Rotation, scale: &Scale) -> Self {
        let translation = Mat4::from_translation(pos.0);
        let rotation = Mat4::from_quat(rot.0);
        let scaling = Mat4::from_scale(scale.0);

        let model_matrix = translation * rotation * scaling;
        let normal_matrix = model_matrix.inverse().transpose();
        Self {
            model_matrix,
            normal_matrix,
        }
    }

    pub fn to_vertex_instance(&self, mat_id: u32) -> VertexInstance {
        let model: [[f32; 4]; 4] = self.model_matrix.to_cols_array_2d();
        let normal: [[f32; 4]; 4] = self.normal_matrix.to_cols_array_2d();
        let translation = self.model_matrix.w_axis.truncate().to_array();

        VertexInstance {
            model,
            color: [1.0, 1.0, 1.0],
            _pad0: 0.0,
            translation,
            _pad1: 0.0,
            uv_offset: [0.0, 0.0],
            _pad2: [0.0; 2],
            normal: [normal[0][0], normal[0][1], normal[0][2]],
            _pad3: 0.0,
            tangent: [normal[1][0], normal[1][1], normal[1][2]],
            _pad4: 0.0,
            material_id: mat_id,
            _pad5: [0.0; 3],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Renderable {
    pub model_key: CacheKey,
    pub visible: bool,
}

impl Renderable {
    pub fn new(key: CacheKey) -> Self {
        Self {
            model_key: key,
            visible: true,
        }
    }
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

pub struct ComponentVec<T> {
    components: Vec<T>,
    entities: Vec<Entity>,
    map: HashMap<Entity, usize>, // Entity -> index in Vec
}

impl<T> ComponentVec<T> {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            entities: Vec::new(),
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, entity: Entity, component: T) {
        if let Some(&idx) = self.map.get(&entity) {
            self.components[idx] = component;
        } else {
            let idx = self.components.len();
            self.map.insert(entity, idx);
            self.entities.push(entity);
            self.components.push(component);
        }
    }

    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        let idx = self.map.remove(&entity)?;
        let last_idx = self.components.len() - 1;
        self.components.swap(idx, last_idx);
        self.entities.swap(idx, last_idx);

        let _removed_entity = self.entities.pop().unwrap();
        let removed_component = self.components.pop().unwrap();

        if idx != last_idx {
            let moved_entity = self.entities[idx];
            self.map.insert(moved_entity, idx);
        }

        Some(removed_component)
    }

    pub fn get(&self, entity: Entity) -> Option<&T> {
        self.map.get(&entity).map(|&idx| &self.components[idx])
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        let idx = *self.map.get(&entity)?;
        Some(&mut self.components[idx])
    }

    pub fn iter(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.entities.iter().copied().zip(self.components.iter())
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Entity, &mut T)> {
        self.entities
            .iter()
            .copied()
            .zip(self.components.iter_mut())
    }
}
