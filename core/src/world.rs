use crate::{
    frustum::Frustum,
    log_info,
    renderer::{material::MaterialManager, model::AABB, Model},
    AssetLoader, BindGroupLayouts, CacheKey, CacheStorage, EngineError, Environment,
    EquirectProjection, GpuContext, Managers, MeshManager, Resources, WgpuBuffer,
};
use cgmath::{Point3, Rotation3};
use std::collections::HashMap;

use super::{
    component::{Position, Renderable, Rotation, Scale, Transform, Velocity},
    entity::Entity,
};

pub struct InstanceBatch {
    batches: HashMap<Entity, Vec<(Entity, Transform)>>,
}

pub struct World {
    positions: Vec<Option<Position>>,
    velocities: Vec<Option<Velocity>>,
    renderables: Vec<Option<Renderable>>,
    rotations: Vec<Option<Rotation>>,
    scales: Vec<Option<Scale>>,
    transforms: Vec<Option<Transform>>,
    managers: Managers,
    environment: Option<Environment>,
    instance_batches: InstanceBatch,
    entity_count: usize,
}

impl World {
    pub fn new(managers: Managers) -> Self {
        Self {
            positions: Vec::new(),
            velocities: Vec::new(),
            renderables: Vec::new(),
            rotations: Vec::new(),
            scales: Vec::new(),
            transforms: Vec::new(),
            managers,
            environment: None,
            instance_batches: InstanceBatch {
                batches: HashMap::new(),
            },
            entity_count: 0,
        }
    }
    pub fn managers(&self) -> &Managers {
        &self.managers
    }
    pub fn managers_mut(&mut self) -> &mut Managers {
        &mut self.managers
    }
    pub fn add_to_instance_batch(
        &mut self,
        target_entity: Entity,
        source_entity: Entity,
        transform: Transform,
    ) {
        if let Some(existing_batch) = self.instance_batches.batches.get_mut(&target_entity) {
            existing_batch.push((source_entity, transform));
        } else {
            self.instance_batches
                .batches
                .insert(target_entity, vec![(source_entity, transform)]);
        }
    }

    pub fn build_environment(
        &mut self,
        resources: &Resources,
        bind_group_layouts: &BindGroupLayouts,
        equirect_projection: Option<EquirectProjection>,
    ) -> Result<(), EngineError> {
        let environment = Environment::new(
            &resources,
            &mut self.managers,
            &bind_group_layouts,
            equirect_projection,
        )?;
        self.environment = Some(environment);
        Ok(())
    }
    pub fn render_environment(
        &mut self,
        rpass: &mut wgpu::RenderPass,
        bind_group_layouts: &BindGroupLayouts,
        camera_bind_group: &wgpu::BindGroup,
    ) {
        if let Some(environment) = &self.environment {
            environment.render(
                rpass,
                &mut self.managers,
                bind_group_layouts,
                camera_bind_group,
            );
        }
    }

    pub fn spawn(&mut self) -> Entity {
        let id = self.entity_count;
        self.entity_count += 1;

        self.positions.resize(self.entity_count, None);
        self.velocities.resize(self.entity_count, None);
        self.renderables.resize(self.entity_count, None);
        self.rotations.resize(self.entity_count, None);
        self.scales.resize(self.entity_count, None);
        self.transforms.resize(self.entity_count, None);

        Entity(id)
    }
    pub async fn load_model<V: bytemuck::Pod, I: bytemuck::Pod>(
        &mut self,
        gpu: &GpuContext,
        asset_loader: &AssetLoader,
        config: &wgpu::SurfaceConfiguration,
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        bind_groups: Vec<wgpu::BindGroup>,
        model_name: &str,
        material_name: &str,
        shader_rel_path: &str,
        texture_rel_path: Option<&str>,
        texture_bind_group_layout: Option<&wgpu::BindGroupLayout>,
        blend_state: wgpu::BlendState,
        cull_mode: wgpu::Face,
        topology: wgpu::PrimitiveTopology,
        front_face: wgpu::FrontFace,
        polygon_mode: wgpu::PolygonMode,
        vertices: &[V],
        indices: &[I],
        aabb: AABB,
    ) -> Result<(), EngineError> {
        let material = MaterialManager::create_material(
            &gpu,
            &asset_loader,
            &mut self.managers,
            &config,
            bind_group_layouts,
            bind_groups,
            &material_name,
            shader_rel_path,
            texture_rel_path,
            texture_bind_group_layout,
            blend_state,
            cull_mode,
            topology,
            front_face,
            polygon_mode,
        )
        .await?;
        let mesh_instance = MeshManager::create_instance(
            &mut self.managers,
            material_name,
            &vertices,
            &indices,
            &material,
        );
        let model = Model {
            meshes: vec![mesh_instance],
            bounding_radius: aabb,
            name: model_name.to_string(),
        };
        self.managers
            .model_manager
            .insert(model_name.into(), model.into());

        Ok(())
    }

    pub fn insert_position(&mut self, entity: Entity, pos: Position) {
        self.positions[entity.0] = Some(pos);
    }

    pub fn insert_velocity(&mut self, entity: Entity, vel: Velocity) {
        self.velocities[entity.0] = Some(vel);
    }
    pub fn insert_scale(&mut self, entity: Entity, scale: Scale) {
        self.scales[entity.0] = Some(scale);
    }
    pub fn insert_rotation(&mut self, entity: Entity, rot: Rotation) {
        self.rotations[entity.0] = Some(rot);
    }
    pub fn insert_renderable(&mut self, entity: Entity, renderable: Renderable) {
        self.renderables[entity.0] = Some(renderable);
    }

    pub fn get_renderable(&self, entity: Entity) -> Option<&Renderable> {
        self.renderables.get(entity.0)?.as_ref()
    }
    pub fn get_transform(&self, entity: Entity) -> Option<&Transform> {
        self.transforms.get(entity.0)?.as_ref()
    }

    pub fn render_entities(&mut self, rpass: &mut wgpu::RenderPass, frustum: Frustum) {
        for i in 0..self.entity_count {
            if let (Some(rend), Some(pos)) = (&self.renderables[i], self.positions[i]) {
                if !rend.visible {
                    continue;
                }

                if let Some(model) = self.managers.model_manager.get(rend.model_key.clone()) {
                    for mesh_instance in model.meshes.iter() {
                        if let Some(material) = self
                            .managers
                            .material_manager
                            .get(mesh_instance.material_key.clone())
                        {
                            if let Some(material_pipeline) = self
                                .managers
                                .pipeline_manager
                                .get_render_pipeline(material.pipeline_key.clone())
                            {
                                if let Some(mesh) = self
                                    .managers
                                    .mesh_manager
                                    .get(mesh_instance.mesh_key.id.clone())
                                {
                                    for (i, bind_group) in material.bind_groups.iter().enumerate() {
                                        rpass.set_bind_group(i as u32, bind_group, &[]);
                                    }
                                    rpass.set_pipeline(&material_pipeline);

                                    if let (
                                        Some(instance_batch),
                                        Some(mesh_vertex_buffer),
                                        Some(mesh_index_buffer),
                                    ) = (
                                        self.instance_batches.batches.get(&Entity(i)),
                                        self.managers.buffer_manager.w_buffer.get(CacheKey::from(
                                            format!("{}:vertex_buffer", mesh.key.id.clone()),
                                        )),
                                        self.managers.buffer_manager.w_buffer.get(CacheKey::from(
                                            format!("{}:index_buffer", mesh.key.id),
                                        )),
                                    ) {
                                        let instance_data: Vec<_> = instance_batch
                                            .iter()
                                            .filter_map(|t| {
                                                let position = Point3::new(
                                                    t.1.matrix.w.x,
                                                    t.1.matrix.w.y,
                                                    t.1.matrix.w.z,
                                                );
                                                if frustum.contains_sphere(position, 0.1) {
                                                    Some(t.1.data())
                                                } else {
                                                    None
                                                }
                                            })
                                            .collect();
                                        log_info!("instance count: {}", instance_data.len());
                                        if !instance_data.is_empty() {
                                            let instance_buffer = WgpuBuffer::from_data(
                                                self.managers.texture_manager.queue(),
                                                self.managers.texture_manager.device(),
                                                &instance_data,
                                                wgpu::BufferUsages::VERTEX
                                                    | wgpu::BufferUsages::COPY_DST,
                                                Some(&format!(
                                                    "{} instance batch buffer",
                                                    model.name
                                                )),
                                            );
                                            rpass.set_index_buffer(
                                                mesh_index_buffer.buffer.slice(..),
                                                wgpu::IndexFormat::Uint32,
                                            );
                                            rpass.set_vertex_buffer(
                                                0,
                                                mesh_vertex_buffer.buffer.slice(..),
                                            );
                                            rpass.set_vertex_buffer(
                                                1,
                                                instance_buffer.buffer.slice(..),
                                            );
                                            rpass.draw_indexed(
                                                0..mesh.index_count,
                                                0,
                                                0..instance_data.len() as u32,
                                            );
                                        }
                                    } else if let (Some(transform), Some(mesh_vertex_buffer)) = (
                                        self.transforms[i],
                                        self.managers.buffer_manager.w_buffer.get(mesh.key.clone()),
                                    ) {
                                        let position = Point3::new(
                                            transform.matrix.w.x,
                                            transform.matrix.w.y,
                                            transform.matrix.w.z,
                                        );
                                        if !frustum.contains_sphere(position, 0.1) {
                                            continue;
                                        }

                                        let instance_buffer = WgpuBuffer::from_data(
                                            self.managers.texture_manager.queue(),
                                            self.managers.texture_manager.device(),
                                            &transform.data(),
                                            wgpu::BufferUsages::VERTEX
                                                | wgpu::BufferUsages::COPY_DST,
                                            Some(&format!("{} instance buffer", model.name)),
                                        );
                                        self.managers.texture_manager.queue().write_buffer(
                                            &instance_buffer.buffer,
                                            0,
                                            bytemuck::cast_slice(&transform.data()),
                                        );

                                        rpass
                                            .set_vertex_buffer(1, instance_buffer.buffer.slice(..));
                                        rpass.set_vertex_buffer(
                                            0,
                                            mesh_vertex_buffer.buffer.slice(..),
                                        );
                                        rpass.draw(0..mesh.vertex_count, 0..1);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn update_physics(&mut self) {
        for i in 0..self.entity_count {
            if let (Some(pos), Some(vel)) = (&mut self.positions[i], self.velocities[i]) {
                pos.x += vel.dx;
                pos.y += vel.dy;
            }
        }
    }
    pub fn update_transforms(&mut self, dt: f64, frustum: Frustum) {
        let delta = cgmath::Quaternion::from_angle_z(cgmath::Deg((dt * 90.0) as f32));

        let len = self.entity_count;
        let positions = &self.positions;
        let rotations = &mut self.rotations;
        let scales = &self.scales;
        let transforms = &mut self.transforms;
        let instance_batches = &mut self.instance_batches.batches;
        let mut update_entity = |i: usize| {
            if let (Some(pos), Some(rot), Some(scale)) = (
                positions[i].as_ref(),
                rotations[i].as_mut(),
                scales[i].as_ref(),
            ) {
                rot.quat = delta * rot.quat;
                let transform = Transform::from_components(pos, rot, scale);
                transforms[i] = Some(transform);

                if let Some(batch) = instance_batches.get_mut(&Entity(i)) {
                    for (source, target_transform) in batch.iter_mut() {
                        let in_sphere = frustum.contains_sphere(
                            Point3::new(
                                target_transform.matrix.w.x,
                                transform.matrix.w.y,
                                transform.matrix.w.z,
                            ),
                            0.1,
                        );
                        if in_sphere {
                            if source.0 < transforms.len() {
                                if let Some(t) = transforms[source.0] {
                                    *target_transform = t;
                                }
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
