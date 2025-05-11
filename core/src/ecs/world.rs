use std::{collections::HashMap, sync::Arc};

use cgmath::InnerSpace;

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
    projection: Option<crate::EquirectProjection>,
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
        _still_running()
    }
    pub fn stop() {
        _stop_running();
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
        // self.update_physics();
        // self.update_transforms(dt as f64);
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

    // pub fn render_batch(
    //     &self,
    //     frustum: Option<&crate::camera::Frustum>,
    // ) -> std::collections::HashMap<crate::CacheKey, Vec<crate::VertexNormalInstance>> {
    //     let mut batch: std::collections::HashMap<
    //         crate::CacheKey,
    //         Vec<crate::VertexNormalInstance>,
    //     > = HashMap::new();
    //     let highlight: [f32; 4] = [1.0, 10.0, 1.0, 0.0];
    //     for idx in 0..self.entity_count {
    //         let renderable = match &self.renderables[idx] {
    //             Some(r) if r.visible => r,
    //             _ => continue,
    //         };

    //         let transform = match &self.transforms[idx] {
    //             Some(t) => t,
    //             None => continue,
    //         };
    //         let magnitude = match &self.scales[idx] {
    //             Some(s) => cgmath::InnerSpace::magnitude(s.value),
    //             None => crate::Scale::one().value.magnitude(),
    //         };
    //         let center = cgmath::Point3::new(
    //             transform.model_matrix.w.x,
    //             transform.model_matrix.w.y,
    //             transform.model_matrix.w.z,
    //         );
    //         if let Some(f) = frustum {
    //             if !f.contains_sphere(center, magnitude) {
    //                 continue;
    //             }
    //         }
    //         let mut data: crate::VertexNormalInstance = transform.to_vertex_instance();
    //         data.color = highlight;
    //         batch.entry(renderable.model_key).or_default().push(data);
    //     }
    //     batch
    // }

    pub fn render_batch(
        &self,
        camera: &crate::camera::Camera,
    ) -> HashMap<crate::CacheKey, Vec<crate::VertexNormalInstance>> {
        let mut best_hit: Option<(usize, f32)> = None;
        for idx in 0..self.entity_count {
            let (_, t, s) = match (
                &self.renderables[idx],
                &self.transforms[idx],
                &self.scales[idx],
            ) {
                (Some(r), Some(t), Some(s)) if r.visible => (r, t, s),
                _ => continue,
            };

            let center =
                cgmath::Point3::new(t.model_matrix.w.x, t.model_matrix.w.y, t.model_matrix.w.z);
            let radius = cgmath::InnerSpace::magnitude(s.value);

            if let Some(t_ray) = camera.is_looking_at(center, radius) {
                if t_ray <= camera.pick_distance() {
                    if best_hit.is_none() || t_ray < best_hit.unwrap().1 {
                        best_hit = Some((idx, t_ray));
                    }
                }
            }
        }
        let picked_idx = best_hit.map(|(i, _)| i);

        let default_color: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        let highlight: [f32; 4] = [1.0, 25.0, 1.0, 1.0];

        let mut batch: HashMap<crate::CacheKey, Vec<crate::VertexNormalInstance>> = HashMap::new();
        let frustum = camera.frustum();
        for idx in 0..self.entity_count {
            let renderable = match &self.renderables[idx] {
                Some(r) if r.visible => r,
                _ => continue,
            };
            let transform = match &self.transforms[idx] {
                Some(t) => t,
                _ => continue,
            };

            let center = cgmath::Point3::new(
                transform.model_matrix.w.x,
                transform.model_matrix.w.y,
                transform.model_matrix.w.z,
            );
            let magnitude = match &self.scales[idx] {
                Some(s) => cgmath::InnerSpace::magnitude(s.value),
                None => 1.0,
            };
            if !frustum.contains_sphere(center, magnitude) {
                continue;
            }

            let mut data: crate::VertexNormalInstance = transform.to_vertex_instance();
            data.color = if Some(idx) == picked_idx {
                highlight
            } else {
                default_color
            };

            batch.entry(renderable.model_key).or_default().push(data);
        }

        batch
    }
}

pub struct WorldTick;

impl WorldTick {
    pub fn run_tokio() {
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(std::time::Duration::from_millis(80));
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
            }
        });
    }
    pub fn run(world: std::sync::Arc<std::sync::RwLock<World>>) {
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

                // ~60Hz
                std::thread::sleep(std::time::Duration::from_millis(16));
            }
            crate::log_info!("World updater thread exiting");
        });
    }
}
