use glam::Vec3;

use crate::{camera::Camera, Medium, Terrain};

use super::{Entity, Position, Velocity};

pub const GROUND_Y: f32 = 0.0;

pub const GRAVITY: f32 = -9.81;

pub const ENTITY_MIN_Y: f32 = GROUND_Y + 2.0;

#[derive(Debug)]
pub struct Physics {
    pub positions: Vec<Option<Position>>,
    pub velocities: Vec<Option<Velocity>>,
}

impl Physics {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            velocities: Vec::new(),
        }
    }

    fn resize(&mut self, size: usize) {
        self.positions.resize(size, None);
        self.velocities.resize(size, None);
    }

    fn ensure_capacity(&mut self, idx: usize) {
        let needed = idx + 1;
        if self.positions.len() < needed || self.velocities.len() < needed {
            self.resize(needed);
        }
    }

    pub fn insert_position(&mut self, entity: Entity, pos: Position) {
        self.ensure_capacity(entity.0);
        self.positions[entity.0] = Some(pos);
    }

    pub fn insert_velocity(&mut self, entity: Entity, vel: Velocity) {
        self.ensure_capacity(entity.0);
        self.velocities[entity.0] = Some(vel);
    }

    /// Physics tick: updates positions/velocities
    pub fn update(&mut self, camera: &Camera, dt: f32, terrain: &Terrain) {
        let camera_pos = *camera.eye();

        let medium = if camera.free_look() || camera_pos.y > GROUND_Y + 4.0 {
            Medium::Air
        } else {
            let pos = Vec3 {
                x: camera_pos.x,
                y: 0.0,
                z: camera_pos.z,
            };
            terrain.medium_at(pos)
        };
        let medium_props = medium.properties();

        let drag_factor = medium_props.drag.powf(dt);
        let max_fall_speed = -50.0;

        for (pos_opt, vel_opt) in self.positions.iter_mut().zip(&mut self.velocities) {
            if let (Some(pos), Some(vel)) = (pos_opt, vel_opt) {
                vel.0.x *= drag_factor;
                vel.0.z *= drag_factor;
                if vel.0.x.abs() < 0.01 {
                    vel.0.x = 0.0;
                }
                if vel.0.z.abs() < 0.01 {
                    vel.0.z = 0.0;
                }
                vel.0.y += medium_props.gravity.y * dt;
                vel.0.y = vel.0.y.max(max_fall_speed);
                pos.0 += vel.0 * dt;

                if pos.0.y < ENTITY_MIN_Y {
                    pos.0.y = ENTITY_MIN_Y;
                    if vel.0.y < 0.0 {
                        vel.0.y = 0.0;
                    }
                }
            }
        }
    }
}
