use pollster::FutureExt;

use crate::{log_error, CacheKey, EngineError, EquirectProjection, ModelManager};

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

#[derive(Debug)]
pub struct World {
    pub positions: Vec<Option<super::Position>>,
    pub velocities: Vec<Option<super::Velocity>>,
    pub renderables: Vec<Option<super::Renderable>>,
    pub rotations: Vec<Option<super::Rotation>>,
    pub scales: Vec<Option<super::Scale>>,
    pub transforms: Vec<Option<super::Transform>>,
    projection: crate::EquirectProjection,
    entity_count: usize,
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
        Ok(Self {
            positions: Vec::new(),
            velocities: Vec::new(),
            renderables: Vec::new(),
            rotations: Vec::new(),
            scales: Vec::new(),
            transforms: Vec::new(),
            projection,
            entity_count: 0,
        })
    }
    pub fn entity_count(&self) -> usize {
        self.entity_count
    }
    pub fn set_projection(&mut self, projection: crate::EquirectProjection) {
        self.projection = projection;
    }
    pub fn projection(&self) -> &crate::EquirectProjection {
        &self.projection
    }

    pub fn spawn_model(
        &mut self,
        model: &str,
        position: Option<super::Position>,
        rotation: Option<super::Rotation>,
        scale: Option<super::Scale>,
    ) {
        let entity: super::Entity = self.spawn();
        let position = position.unwrap_or((1.0, 1.0, 1.0).into());
        let rotation = rotation.unwrap_or(cgmath::Deg(00.0 % 360.0).into());
        let scale = scale.unwrap_or(
            cgmath::Vector3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            }
            .into(),
        );
        let renderable = super::Renderable {
            model_key: model.into(),
            visible: true,
        };
        self.insert_position(entity, position);
        self.insert_rotation(entity, rotation);
        self.insert_scale(entity, scale);
        self.insert_renderable(entity, renderable);
        crate::log_debug!("Spawned model entity: {} {}", entity.0, model);
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
    ) -> Option<crate::CacheKey> {
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
    pub fn spawn(&mut self) -> super::Entity {
        let id = self.entity_count;
        self.entity_count += 1;
        self.ensure_capacity(self.entity_count);
        super::Entity(id)
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
    pub fn insert_position(&mut self, entity: super::Entity, pos: super::Position) {
        self.ensure_capacity(entity.0);
        self.positions[entity.0] = Some(pos);
    }

    pub fn insert_velocity(&mut self, entity: super::Entity, vel: super::Velocity) {
        self.ensure_capacity(entity.0);
        self.velocities[entity.0] = Some(vel);
    }
    pub fn insert_scale(&mut self, entity: super::Entity, scale: super::Scale) {
        self.ensure_capacity(entity.0);
        self.scales[entity.0] = Some(scale);
    }
    pub fn insert_rotation(&mut self, entity: super::Entity, rot: super::Rotation) {
        self.ensure_capacity(entity.0);
        self.rotations[entity.0] = Some(rot);
    }
    pub fn insert_renderable(&mut self, entity: super::Entity, renderable: super::Renderable) {
        self.ensure_capacity(entity.0);
        self.renderables[entity.0] = Some(renderable);
    }

    pub fn get_renderable(&self, entity: super::Entity) -> Option<&super::Renderable> {
        self.renderables.get(entity.0)?.as_ref()
    }
    pub fn get_renderables(&self) -> &Vec<Option<super::Renderable>> {
        self.renderables.as_ref()
    }
    pub fn get_transform(&self, entity: super::Entity) -> Option<&super::Transform> {
        self.transforms.get(entity.0)?.as_ref()
    }

    pub fn update(&mut self, dt: f32) {
        self.update_physics();
        self.update_transforms(dt as f64);
    }
    pub fn update_physics(&mut self) {
        for i in 0..self.entity_count {
            if let (Some(pos), Some(vel)) = (&mut self.positions[i], self.velocities[i]) {
                pos.update(&vel);
            }
        }
    }
    pub fn update_transforms(&mut self, dt: f64) {
        let delta = <cgmath::Quaternion<f32> as cgmath::Rotation3>::from_angle_z(cgmath::Deg(
            (dt * 90.0) as f32,
        ));

        for i in 0..self.entity_count {
            if let (Some(pos), Some(rot), Some(scale)) = (
                self.positions[i].as_ref(),
                self.rotations[i].as_mut(),
                self.scales[i].as_ref(),
            ) {
                rot.update(delta);
                let transform = super::Transform::from_components(pos, rot, scale);
                self.transforms[i] = Some(transform);
            }
        }
    }
}
