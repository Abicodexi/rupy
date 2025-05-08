use std::sync::Arc;

use cgmath::{Matrix3, Transform};

use crate::log_debug;

static WORLD: std::sync::OnceLock<std::sync::Arc<std::sync::RwLock<crate::World>>> =
    std::sync::OnceLock::new();

fn init_world() {
    let world = World::new();
    let arc_world = std::sync::Arc::new(std::sync::RwLock::new(world));
    WORLD
        .set(arc_world)
        .expect("Global world was already initialized");
}

fn world() -> Option<std::sync::Arc<std::sync::RwLock<World>>> {
    WORLD.get().cloned()
}
pub static RUNNING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

fn still_running() -> bool {
    RUNNING.load(std::sync::atomic::Ordering::Relaxed)
}
fn stop_running() {
    RUNNING.store(false, std::sync::atomic::Ordering::Relaxed)
}
#[derive(Debug)]
pub struct InstanceBatch {
    batches: std::collections::HashMap<super::Entity, Vec<(super::Entity, super::Transform)>>,
}
impl InstanceBatch {
    pub fn new() -> Self {
        Self {
            batches: std::collections::HashMap::new(),
        }
    }
    pub fn batches(
        &self,
    ) -> &std::collections::HashMap<super::Entity, Vec<(super::Entity, super::Transform)>> {
        &self.batches
    }
    pub fn raw_data_for(
        &self,
        target: super::Entity,
        frustum: Option<&crate::camera::Frustum>,
    ) -> Vec<crate::TransformRaw> {
        match self.batches.get(&target) {
            None => Vec::new(),
            Some(batch) => batch
                .iter()
                .filter_map(|&(_source, transform)| {
                    let pos = cgmath::Point3::new(
                        transform.model_matrix.w.x,
                        transform.model_matrix.w.y,
                        transform.model_matrix.w.z,
                    );

                    if frustum.map_or(true, |f| f.contains_sphere(pos, 0.1)) {
                        Some(transform.data())
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }
    pub fn raw_data(
        &self,
        frustum: Option<&crate::camera::Frustum>,
    ) -> Vec<Vec<crate::TransformRaw>> {
        self.batches
            .values()
            .map(|batch| {
                batch
                    .iter()
                    .filter_map(|&(_source_entity, transform)| {
                        let pos = cgmath::Point3::new(
                            transform.model_matrix.w.x,
                            transform.model_matrix.w.y,
                            transform.model_matrix.w.z,
                        );
                        if frustum.map_or(true, |f| f.contains_sphere(pos, 0.1)) {
                            Some(transform.data())
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .collect()
    }
}
#[derive(Debug)]
pub struct World {
    pub positions: Vec<Option<super::Position>>,
    pub velocities: Vec<Option<super::Velocity>>,
    pub renderables: Vec<Option<super::Renderable>>,
    pub rotations: Vec<Option<super::Rotation>>,
    pub scales: Vec<Option<super::Scale>>,
    pub transforms: Vec<Option<super::Transform>>,
    projection: Option<crate::EquirectProjection>,
    pub instance: InstanceBatch,
    entity_count: usize,
}

impl World {
    pub fn get() -> Option<std::sync::Arc<std::sync::RwLock<World>>> {
        world()
    }
    pub fn init() {
        init_world();
    }
    pub fn running() -> bool {
        still_running()
    }
    pub fn stop() {
        stop_running();
    }
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            velocities: Vec::new(),
            renderables: Vec::new(),
            rotations: Vec::new(),
            scales: Vec::new(),
            transforms: Vec::new(),
            projection: None,
            instance: InstanceBatch::new(),
            entity_count: 0,
        }
    }
    pub fn set_projection(&mut self, projection: crate::EquirectProjection) -> bool {
        self.projection = Some(projection);
        self.projection.is_some()
    }
    pub fn projection(&self) -> &Option<crate::EquirectProjection> {
        &self.projection
    }

    pub fn batch_instance(
        &mut self,
        target: super::Entity,
        source: super::Entity,
        transform: super::Transform,
    ) {
        if let Some(batch) = self.instance.batches.get_mut(&target) {
            batch.push((source, transform));
        } else {
            self.instance
                .batches
                .insert(target, vec![(source, transform)]);
        }
    }

    pub fn new_renderable(
        &mut self,
        model_key: &str,
        position: Option<super::Position>,
        rotation: Option<super::Rotation>,
        scale: Option<super::Scale>,
    ) {
        let entity: super::Entity = self.spawn();
        let angle_deg = 00.0 % 360.0;
        let angle = cgmath::Deg(angle_deg);
        let position = position.unwrap_or(super::Position { x: 1.0, y: 1.0 });
        let rotation = rotation.unwrap_or(super::Rotation {
            quat: <cgmath::Quaternion<f32> as cgmath::Rotation3>::from_angle_z(angle),
        });
        let scale = scale.unwrap_or(super::Scale {
            value: cgmath::Vector3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        });
        let renderable = super::Renderable {
            model_key: model_key.into(),
            visible: true,
        };
        self.insert_position(entity, position);
        self.insert_rotation(entity, rotation);
        self.insert_scale(entity, scale);
        self.insert_renderable(entity, renderable);
        log_debug!("Spawned renderable entity: {} {}", entity.0, model_key);
    }
    pub fn load_object(
        obj: &str,
        managers: &mut crate::Managers,
        uniform_bind_group: &wgpu::BindGroup,
        camera: &crate::camera::Camera,
        light: &crate::Light,
        surface_config: &wgpu::SurfaceConfiguration,
        depth_stencil_state: &Option<wgpu::DepthStencilState>,
    ) -> Option<crate::CacheKey> {
        match crate::Model::from_obj(
            obj,
            managers,
            uniform_bind_group,
            &camera,
            light,
            &surface_config,
            depth_stencil_state,
        ) {
            Ok(Some(model)) => {
                managers
                    .model_manager
                    .models
                    .insert(obj.into(), model.into());
                Some(crate::CacheKey::from(obj))
            }
            Err(e) => {
                crate::log_error!("Error loading tobj: {}", e);
                None
            }
            _ => {
                crate::log_error!("Error loading tobj: model returned as None");
                None
            }
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

    /// Pure‐data tick: physics, transforms, and rebuild your instance_batches.
    pub fn update(&mut self, dt: f32) {
        self.update_physics();
        self.update_transforms(dt as f64);
    }
    pub fn update_physics(&mut self) {
        for i in 0..self.entity_count {
            if let (Some(pos), Some(vel)) = (&mut self.positions[i], self.velocities[i]) {
                pos.x += vel.dx;
                pos.y += vel.dy;
            }
        }
    }
    pub fn update_transforms(&mut self, dt: f64) {
        let delta = <cgmath::Quaternion<f32> as cgmath::Rotation3>::from_angle_z(cgmath::Deg(
            (dt * 90.0) as f32,
        ));

        let len = self.entity_count;
        let positions = &self.positions;
        let rotations = &mut self.rotations;
        let scales = &self.scales;
        let transforms = &mut self.transforms;
        let instance_batches = &mut self.instance.batches;
        let mut update_entity = |i: usize| {
            if let (Some(pos), Some(rot), Some(scale)) = (
                positions[i].as_ref(),
                rotations[i].as_mut(),
                scales[i].as_ref(),
            ) {
                rot.quat = delta * rot.quat;
                let transform = super::Transform::from_components(pos, rot, scale);
                transforms[i] = Some(transform);

                if let Some(batch) = instance_batches.get_mut(&super::Entity(i)) {
                    for (source, target_transform) in batch.iter_mut() {
                        if source.0 < transforms.len() {
                            if let Some(t) = transforms[source.0] {
                                *target_transform = t;
                            }
                        }
                    }
                }
            }
        };

        let mut i = 0;
        while i + 3 < len {
            update_entity(i);
            update_entity(i + 1);
            update_entity(i + 2);
            update_entity(i + 3);
            i += 4;
        }

        while i < len {
            update_entity(i);
            i += 1;
        }
    }
}

pub struct WorldTick;

impl WorldTick {
    pub fn run_tokio(update_tx: &Arc<crossbeam::channel::Sender<crate::ApplicationEvent>>) {
        let tx = update_tx.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(std::time::Duration::from_millis(16));
            let mut last = std::time::Instant::now();
            loop {
                ticker.tick().await;

                if !World::running() {
                    crate::log_info!("World tick stopping (shutdown flag set)");
                    break;
                }

                let now = std::time::Instant::now();
                let dt = (now - last).as_secs_f32();
                last = now;

                {
                    if let Some(world) = World::get() {
                        match world.write() {
                            Ok(mut w) => {
                                w.update(dt);
                            }
                            Err(e) => {
                                crate::log_error!(
                                    "World update failed, could not acquire lock: {}",
                                    e
                                );
                            }
                        }
                    }
                }

                if let Err(e) = tx.send(crate::ApplicationEvent::WorldRequestRedraw) {
                    crate::log_error!("Event loop closed, stopping world tick: {}", e);
                    break;
                }
            }
        });
    }
    pub fn run(
        world: std::sync::Arc<std::sync::RwLock<World>>,
        update_tx: std::sync::Arc<crossbeam::channel::Sender<crate::ApplicationEvent>>,
    ) {
        std::thread::spawn(move || {
            let mut last = std::time::Instant::now();
            while World::running() {
                let now = std::time::Instant::now();
                let dt = (now - last).as_secs_f32();
                last = now;

                {
                    let mut w = world.write().unwrap();
                    w.update(dt);
                }

                if let Err(e) = update_tx.send(crate::ApplicationEvent::WorldRequestRedraw) {
                    crate::log_error!("Event loop closed, stopping updater: {}", e);
                    break;
                }

                // ~60Hz
                std::thread::sleep(std::time::Duration::from_millis(16));
            }
            crate::log_info!("World updater thread exiting");
        });
    }
}
