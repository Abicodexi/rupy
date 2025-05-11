use std::sync::Arc;

use crate::{log_debug, log_info};

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
pub struct InstanceBatcher {
    batches: std::collections::HashMap<crate::CacheKey, Vec<super::Transform>>,
}

impl InstanceBatcher {
    pub fn new() -> Self {
        Self {
            batches: std::collections::HashMap::new(),
        }
    }

    pub fn add_instance(&mut self, model_key: crate::CacheKey, transform: super::Transform) {
        log_info!(
            "Containts {} {}",
            model_key.id(),
            self.batches.contains_key(&model_key)
        );
        self.batches.entry(model_key).or_default().push(transform);
    }

    pub fn clear(&mut self) {
        self.batches.clear();
    }

    pub fn batches(&self) -> &std::collections::HashMap<crate::CacheKey, Vec<super::Transform>> {
        &self.batches
    }

    pub fn raw_data_for(
        &self,
        model_key: &crate::CacheKey,
        frustum: Option<&crate::camera::Frustum>,
    ) -> Vec<crate::VertexNormalInstance> {
        self.batches
            .get(model_key)
            .map(|transforms| {
                transforms
                    .iter()
                    .filter_map(|t| {
                        let pos = cgmath::Point3::new(
                            t.model_matrix.w.x,
                            t.model_matrix.w.y,
                            t.model_matrix.w.z,
                        );
                        if frustum.map_or(false, |f| f.contains_sphere(pos, 0.1)) {
                            Some(t.to_vertex_instance())
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
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
    pub instance: InstanceBatcher,
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
            instance: InstanceBatcher::new(),
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

    pub fn batch_instance(&mut self, model_key: crate::CacheKey, transform: super::Transform) {
        self.instance.add_instance(model_key, transform);
    }

    pub fn spawn_model(
        &mut self,
        model: &str,
        position: Option<super::Position>,
        rotation: Option<super::Rotation>,
        scale: Option<super::Scale>,
    ) {
        let entity: super::Entity = self.spawn();
        let position = position.unwrap_or((1.0, 1.0).into());
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
        log_debug!("Spawned model entity: {} {}", entity.0, model);
    }
    pub fn load_object(
        obj: &str,
        managers: &mut crate::Managers,
        surface_config: &wgpu::SurfaceConfiguration,
        depth_stencil_state: &Option<wgpu::DepthStencilState>,
    ) -> Option<crate::CacheKey> {
        match crate::Asset::tobj(obj, managers, &surface_config, depth_stencil_state) {
            Ok(model) => {
                let key: crate::CacheKey = obj.into();
                managers.model_manager.models.insert(key, model.into());
                Some(key)
            }
            Err(e) => {
                crate::log_error!("{}", e.to_string());
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

        // === Update per-entity transforms ===
        // === Rebuild instance batches ===
        let mut new_batches: std::collections::HashMap<crate::CacheKey, Vec<super::Transform>> =
            std::collections::HashMap::new();

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

        for (i, rend_opt) in self.renderables.iter().enumerate() {
            if let (Some(rend), Some(transform)) = (rend_opt, self.transforms[i]) {
                new_batches
                    .entry(rend.model_key)
                    .or_insert_with(Vec::new)
                    .push(transform);
            }
        }

        self.instance.batches = new_batches;
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

                if let Err(e) = tx.send(crate::ApplicationEvent::Draw) {
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

                if let Err(e) = update_tx.send(crate::ApplicationEvent::Draw) {
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
