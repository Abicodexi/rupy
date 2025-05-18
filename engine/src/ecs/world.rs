use super::{Position, Renderable, Rotation, Scale, Transform, Velocity};
use crate::{
    camera::Camera, log_debug, log_error, CacheKey, EngineError, Entity, EquirectProjection,
    Medium, ModelManager, Terrain,
};
use glam::Vec3;
use pollster::FutureExt;

pub static RUNNING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

fn _still_running() -> bool {
    RUNNING.load(std::sync::atomic::Ordering::Relaxed)
}
fn _stop_running() {
    RUNNING.store(false, std::sync::atomic::Ordering::Relaxed)
}

pub static BATCH_DIRTY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn _is_batch_dirty() -> bool {
    BATCH_DIRTY.load(std::sync::atomic::Ordering::Relaxed)
}
fn _set_batch_dirty(val: bool) {
    BATCH_DIRTY.store(val, std::sync::atomic::Ordering::Relaxed)
}

fn is_grounded(pos: &Position, vel: &Velocity) -> bool {
    (pos.0.y - GROUND_Y).abs() < 0.001 && vel.0.y.abs() < 0.01
}

pub const GROUND_Y: f32 = 2.0;

#[derive(Debug)]
pub struct World {
    pub positions: Vec<Option<Position>>,
    pub velocities: Vec<Option<Velocity>>,
    pub renderables: Vec<Option<Renderable>>,
    pub rotations: Vec<Option<Rotation>>,
    pub scales: Vec<Option<Scale>>,
    pub transforms: Vec<Option<Transform>>,
    projection: EquirectProjection,
    entity_count: usize,
    pub terrain: Terrain,
}

impl World {
    pub fn running() -> bool {
        _still_running()
    }
    pub fn stop() {
        _stop_running();
    }

    pub fn new(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        depth_stencil_state: Option<wgpu::DepthStencilState>,
    ) -> Result<Self, EngineError> {
        let projection = EquirectProjection::new(
            &queue,
            &device,
            &config,
            "equirect_src.wgsl",
            "equirect_dst.wgsl",
            "pure-sky.hdr",
            depth_stencil_state,
        )?;
        let terrain = Terrain::new(Medium::Ground);
        Ok(Self {
            positions: Vec::new(),
            velocities: Vec::new(),
            renderables: Vec::new(),
            rotations: Vec::new(),
            scales: Vec::new(),
            transforms: Vec::new(),
            projection,
            entity_count: 0,
            terrain,
        })
    }
    pub fn entity_count(&self) -> usize {
        self.entity_count
    }
    pub fn set_projection(&mut self, projection: EquirectProjection) {
        self.projection = projection;
    }
    pub fn projection(&self) -> &EquirectProjection {
        &self.projection
    }

    pub fn insert_object(
        &mut self,
        renderable: Renderable,
        position: Option<Position>,
        rotation: Option<Rotation>,
        scale: Option<Scale>,
    ) {
        let entity: Entity = self.spawn();
        let position = position.unwrap_or(Position::origin());
        let rotation = rotation.unwrap_or(Rotation::zero());
        let scale = scale.unwrap_or(Scale::one());
        self.insert_position(entity, position);
        self.insert_rotation(entity, rotation);
        self.insert_scale(entity, scale);
        self.insert_renderable(entity, renderable);
        crate::log_debug!("Spawned model entity: {}", entity.0);
    }
    pub fn load_object(
        model_manager: &mut ModelManager,
        file: &str,
        shader: &str,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        surface_configuration: &wgpu::SurfaceConfiguration,
        primitive: wgpu::PrimitiveState,
        color_target: wgpu::ColorTargetState,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> Option<CacheKey> {
        match model_manager
            .load_object_file(
                file,
                shader,
                buffers,
                bind_group_layouts,
                surface_configuration,
                primitive,
                color_target,
                depth_stencil,
            )
            .block_on()
        {
            Err(e) => {
                log_error!("{}: {}", file, e.to_string());
                None
            }
            _ => Some(CacheKey::from(file)),
        }
    }
    pub fn spawn(&mut self) -> Entity {
        let id = self.entity_count;
        self.entity_count += 1;
        self.ensure_capacity(self.entity_count);
        Entity(id)
    }
    fn resize(&mut self, size: usize) {
        self.positions.resize(size, None);
        self.velocities.resize(size, None);
        self.renderables.resize(size, None);
        self.rotations.resize(size, None);
        self.scales.resize(size, None);
        self.transforms.resize(size, None);
    }
    fn ensure_capacity(&mut self, idx: usize) {
        let needed = idx + 1;
        if self.positions.len() < needed
            || self.velocities.len() < needed
            || self.rotations.len() < needed
            || self.renderables.len() < needed
            || self.scales.len() < needed
            || self.transforms.len() < needed
        {
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
    pub fn insert_scale(&mut self, entity: Entity, scale: Scale) {
        self.ensure_capacity(entity.0);
        self.scales[entity.0] = Some(scale);
    }
    pub fn insert_rotation(&mut self, entity: Entity, rot: Rotation) {
        self.ensure_capacity(entity.0);
        self.rotations[entity.0] = Some(rot);
    }
    pub fn insert_renderable(&mut self, entity: Entity, renderable: Renderable) {
        self.ensure_capacity(entity.0);
        self.renderables[entity.0] = Some(renderable);
    }

    pub fn get_renderable(&self, entity: Entity) -> Option<&Renderable> {
        self.renderables.get(entity.0)?.as_ref()
    }
    pub fn get_renderables(&self) -> &Vec<Option<Renderable>> {
        self.renderables.as_ref()
    }
    pub fn get_transform(&self, entity: Entity) -> Option<&Transform> {
        self.transforms.get(entity.0)?.as_ref()
    }

    pub fn update(&mut self, queue: &wgpu::Queue, device: &wgpu::Device, camera: &Camera, dt: f32) {
        self.terrain.update_instance_buffer(queue, device);
        self.update_physics(camera, dt);
        self.update_transforms();
    }
    pub fn update_physics(&mut self, camera: &Camera, dt: f32) {
        let medium = {
            if camera.free_look() {
                Medium::Vacuum
            } else {
                self.terrain.default_medium()
            }
        };

        let medium_props = medium.properties();

        let drag_factor = medium_props.drag.powf(dt);
        let ground_y = GROUND_Y;
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
                // Apply gravity
                vel.0.y += medium_props.gravity.y * dt;
                vel.0.y = vel.0.y.max(max_fall_speed);

                // Apply drag
                vel.0.x *= drag_factor;
                vel.0.z *= drag_factor;

                // Update position
                pos.0 += vel.0 * dt;

                // Ground collision check
                if pos.0.y <= ground_y {
                    pos.0.y = ground_y;

                    // Snap + consume vertical velocity
                    if vel.0.y < 0.0 {
                        vel.0.y = 0.0;
                    }
                }
            }
        }
    }
    pub fn update_transforms(&mut self) {
        for i in 0..self.renderables.len() {
            if let (Some(pos), Some(rot), Some(scale)) = (
                self.positions[i].as_ref(),
                self.rotations[i].as_ref(),
                self.scales[i].as_ref(),
            ) {
                self.transforms[i] = Some(Transform::from_components(pos, rot, scale));
            }
        }
    }

    pub fn generate_terrain(
        &mut self,
        center: Vec3,
        radius: i32,
        surface_config: &wgpu::SurfaceConfiguration,
        depth_stencil: &wgpu::DepthStencilState,
        model_manager: &mut ModelManager,
    ) {
        let terrain_entity = self.spawn();

        let renderable = self.terrain.generate_initial_chunks(
            center,
            radius,
            surface_config,
            depth_stencil,
            model_manager,
        );
        self.insert_renderable(terrain_entity, renderable);
    }
}
