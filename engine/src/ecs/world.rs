use std::sync::Arc;

use super::{Physics, Position, Renderable, Rotation, Scale, Transform, Velocity};
use crate::{
    camera::Camera, log_error, log_info, BindGroupManager, CacheKey, EngineError, Entity,
    Environment, Light, Material, MaterialManager, Medium, ModelManager, PipelineManager,
    RenderBindGroupLayouts, ShaderManager, Terrain, TextureManager, Time,
};
use glam::Vec3;
use pollster::FutureExt;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

pub static RUNNING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn _start_running() {
    RUNNING.store(true, std::sync::atomic::Ordering::Relaxed)
}
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

#[derive(Debug)]
pub struct World {
    physics: Physics,
    renderables: Vec<Option<Renderable>>,
    rotations: Vec<Option<Rotation>>,
    scales: Vec<Option<Scale>>,
    transforms: Vec<Option<Transform>>,
    environment: Environment,
    entity_count: usize,
    terrain: Terrain,
    dirty: bool,
}

impl World {
    pub fn running() -> bool {
        _still_running()
    }
    pub fn stop() {
        log_info!("Stopping world");
        _stop_running();
    }
    pub fn start(&self) {
        log_info!("Starting world with entity count: {}", self.entity_count);
        _start_running();
    }
    pub fn new(environment: Environment, terrain: Terrain) -> Result<Self, EngineError> {
        Ok(Self {
            physics: Physics::new(),
            renderables: Vec::new(),
            rotations: Vec::new(),
            scales: Vec::new(),
            transforms: Vec::new(),
            entity_count: 0,
            environment,
            terrain,
            dirty: false,
        })
    }
    pub fn entity_count(&self) -> usize {
        self.entity_count
    }
    // === Physics ===
    pub fn physics(&self) -> &Physics {
        &self.physics
    }

    pub fn physics_mut(&mut self) -> &mut Physics {
        &mut self.physics
    }

    // === Transforms ===
    pub fn transform(&self, entity: Entity) -> Option<&Transform> {
        self.transforms.get(entity.0)?.as_ref()
    }

    pub fn transforms(&self) -> &[Option<Transform>] {
        &self.transforms
    }

    // === Rotations ===
    pub fn rotation(&self, entity: Entity) -> Option<&Rotation> {
        self.rotations.get(entity.0)?.as_ref()
    }
    pub fn rotations(&self) -> &[Option<Rotation>] {
        &self.rotations
    }
    pub fn rotation_mut(&mut self, entity: Entity) -> Option<&mut Rotation> {
        self.rotations.get_mut(entity.0)?.as_mut()
    }

    // === Scales ===
    pub fn scale(&self, entity: Entity) -> Option<&Scale> {
        self.scales.get(entity.0)?.as_ref()
    }
    pub fn scales(&self) -> &[Option<Scale>] {
        &self.scales
    }
    pub fn scale_mut(&mut self, entity: Entity) -> Option<&mut Scale> {
        self.scales.get_mut(entity.0)?.as_mut()
    }

    // === Positions ===
    pub fn position(&self, entity: Entity) -> Option<&Position> {
        self.physics.position(entity)
    }
    pub fn positions(&self) -> &[Option<Position>] {
        &self.physics.positions()
    }
    pub fn velocity(&self, entity: Entity) -> Option<&Velocity> {
        self.physics.velocity(entity)
    }
    pub fn velocities(&self) -> &[Option<Velocity>] {
        &self.physics.velocities()
    }
    // === Renderables ===
    pub fn renderable(&self, entity: Entity) -> Option<&Renderable> {
        self.renderables.get(entity.0)?.as_ref()
    }

    pub fn renderables(&self) -> &[Option<Renderable>] {
        &self.renderables
    }

    // === Terrain ===
    pub fn terrain(&self) -> &Terrain {
        &self.terrain
    }

    pub fn terrain_mut(&mut self) -> &mut Terrain {
        &mut self.terrain
    }
    pub fn environment(&self) -> &Environment {
        &self.environment
    }
    pub fn environment_mut(&mut self) -> &mut Environment {
        &mut self.environment
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
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        model_manager: &mut ModelManager,
        material_manager: &mut MaterialManager,
        texture_manager: &mut TextureManager,
        shader_manager: &mut ShaderManager,
        pipeline_manager: &mut PipelineManager,
        bind_group_manager: &mut BindGroupManager,
        layouts: &RenderBindGroupLayouts,
        file: &str,
        v_shader: &str,
        f_shader: &str,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
        format: wgpu::TextureFormat,
        primitive: wgpu::PrimitiveState,
        color_target: wgpu::ColorTargetState,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> Option<CacheKey> {
        match model_manager
            .load(
                queue,
                device,
                material_manager,
                texture_manager,
                shader_manager,
                pipeline_manager,
                bind_group_manager,
                layouts,
                file,
                v_shader,
                f_shader,
                buffers,
                bind_group_layouts,
                format,
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
        self.physics.resize(size);
        self.renderables.resize(size, None);
        self.rotations.resize(size, None);
        self.scales.resize(size, None);
        self.transforms.resize(size, None);
    }
    fn ensure_capacity(&mut self, idx: usize) {
        let needed = idx + 1;
        if self.physics.positions().len() < needed
            || self.physics.velocities().len() < needed
            || self.rotations.len() < needed
            || self.renderables.len() < needed
            || self.scales.len() < needed
            || self.transforms.len() < needed
        {
            self.resize(needed);
        }
    }
    pub fn insert_position(&mut self, entity: Entity, pos: Position) {
        self.physics.insert_position(entity, pos);
    }
    pub fn insert_velocity(&mut self, entity: Entity, vel: Velocity) {
        self.physics.insert_velocity(entity, vel);
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

    pub fn update_transforms(&mut self) {
        self.transforms
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, transform)| {
                if let (Some(pos), Some(rot), Some(scale)) = (
                    self.physics.positions()[i].as_ref(),
                    self.rotations[i].as_ref(),
                    self.scales[i].as_ref(),
                ) {
                    *transform = Some(Transform::from_components(pos, rot, scale));
                }
            });
    }

    pub fn generate_terrain(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        center: Vec3,
        radius: i32,
        medium: Medium,
        material: &Arc<Material>,
    ) -> Result<(), EngineError> {
        let entity = self.spawn();
        let component = self
            .terrain
            .chunks(queue, device, center, radius, medium, material)?;

        self.insert_renderable(entity, component);
        Ok(())
    }

    pub fn light(&self) -> &Light {
        self.environment.light()
    }
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    pub fn upload(&mut self, queue: &wgpu::Queue, device: &wgpu::Device) {
        if self.dirty {
            self.environment.upload_light(queue, device);
            self.dirty = false;
        }
    }
    pub fn update(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        time: &Time,
        camera: &Camera,
        bossman: Entity,
    ) {
        let dt = time.delta_time;
        self.environment.compute_sky_projection(queue, device);

        let view_distance = 1;

        let camera_pos = camera.eye();

        if let Some(cam_entity) = camera.entity() {
            if let (Some(cam_pos), Some(boss_pos)) = (
                self.physics.position(cam_entity),
                self.physics.position(bossman),
            ) {
                let direction = cam_pos.0 - boss_pos.0;
                let mut direction_normalized = direction.normalize_or_zero();
                let speed = 1.0;
                let velocity = direction_normalized * speed;
                direction_normalized.y = 0.0;
                let rot_to_camera = glam::Quat::from_rotation_arc(Vec3::Z, direction_normalized);
                self.insert_rotation(bossman, Rotation::from(rot_to_camera));
                self.insert_velocity(bossman, Velocity(velocity));
            }
        }

        self.environment.update_light(dt);
        self.physics.update(camera, dt, &self.terrain);
        self.update_transforms();
        self.terrain.update_streaming(camera_pos, view_distance);
        self.terrain.update_instance_buffer(queue, device);
        self.dirty = true;
    }
}
