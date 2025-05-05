pub struct InstanceBatch {
    batches: std::collections::HashMap<super::Entity, Vec<(super::Entity, super::Transform)>>,
}
impl InstanceBatch {
    pub fn new() -> Self {
        Self {
            batches: std::collections::HashMap::new(),
        }
    }
}

pub struct World {
    positions: Vec<Option<super::Position>>,
    velocities: Vec<Option<super::Velocity>>,
    renderables: Vec<Option<super::Renderable>>,
    rotations: Vec<Option<super::Rotation>>,
    scales: Vec<Option<super::Scale>>,
    transforms: Vec<Option<super::Transform>>,
    environment: crate::Environment,
    instance_batches: InstanceBatch,
    entity_count: usize,
}

impl World {
    pub fn new(environment: crate::Environment) -> Self {
        Self {
            positions: Vec::new(),
            velocities: Vec::new(),
            renderables: Vec::new(),
            rotations: Vec::new(),
            scales: Vec::new(),
            transforms: Vec::new(),
            environment,
            instance_batches: InstanceBatch::new(),
            entity_count: 0,
        }
    }

    pub fn compute_equirect_projection(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        managers: &mut crate::Managers,
        bind_group_layouts: &crate::BindGroupLayouts,
    ) {
        let encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Equirect projection command encoder"),
        });
        self.environment.equirect_projection.compute_projection(
            queue,
            device,
            encoder,
            managers,
            bind_group_layouts,
            Some("Equirect projection pass"),
        );
    }

    pub fn add_instance_to_batch(
        &mut self,
        target: super::Entity,
        source: super::Entity,
        transform: super::Transform,
    ) {
        if let Some(batch) = self.instance_batches.batches.get_mut(&target) {
            batch.push((source, transform));
        } else {
            self.instance_batches
                .batches
                .insert(target, vec![(source, transform)]);
        }
    }

    pub fn spawn(&mut self) -> super::Entity {
        let id = self.entity_count;
        self.entity_count += 1;

        self.positions.resize(self.entity_count, None);
        self.velocities.resize(self.entity_count, None);
        self.renderables.resize(self.entity_count, None);
        self.rotations.resize(self.entity_count, None);
        self.scales.resize(self.entity_count, None);
        self.transforms.resize(self.entity_count, None);

        super::Entity(id)
    }

    pub fn insert_position(&mut self, entity: super::Entity, pos: super::Position) {
        self.positions[entity.0] = Some(pos);
    }

    pub fn insert_velocity(&mut self, entity: super::Entity, vel: super::Velocity) {
        self.velocities[entity.0] = Some(vel);
    }
    pub fn insert_scale(&mut self, entity: super::Entity, scale: super::Scale) {
        self.scales[entity.0] = Some(scale);
    }
    pub fn insert_rotation(&mut self, entity: super::Entity, rot: super::Rotation) {
        self.rotations[entity.0] = Some(rot);
    }
    pub fn insert_renderable(&mut self, entity: super::Entity, renderable: super::Renderable) {
        self.renderables[entity.0] = Some(renderable);
    }

    pub fn get_renderable(&self, entity: super::Entity) -> Option<&super::Renderable> {
        self.renderables.get(entity.0)?.as_ref()
    }
    pub fn get_transform(&self, entity: super::Entity) -> Option<&super::Transform> {
        self.transforms.get(entity.0)?.as_ref()
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        managers: &mut crate::Managers,
        rpass: &mut wgpu::RenderPass,
        bind_group_layouts: &crate::BindGroupLayouts,
        camera: &crate::camera::Camera,
    ) {
        let frustum = &camera.frustum;
        self.environment.render(
            device,
            rpass,
            managers,
            bind_group_layouts,
            &camera.bind_group,
        );
        for i in 0..self.entity_count {
            if let Some(rend) = &self.renderables[i] {
                if !rend.visible {
                    continue;
                }

                if let Some(model) =
                    crate::CacheStorage::get(&managers.model_manager, &rend.model_key)
                {
                    for mesh_instance in model.meshes.iter() {
                        if let Some(material) = crate::CacheStorage::get(
                            &managers.material_manager,
                            &mesh_instance.material_key,
                        ) {
                            if let Some(material_pipeline) = managers
                                .pipeline_manager
                                .get_render_pipeline(material.pipeline_key.clone())
                            {
                                if let Some(mesh) =
                                    <crate::MeshManager as crate::CacheStorage<crate::Mesh>>::get(
                                        &managers.mesh_manager,
                                        &mesh_instance.material_key,
                                    )
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
                                        self.instance_batches.batches.get(&super::Entity(i)),
                                        crate::CacheStorage::get(
                                            &managers.buffer_manager.w_buffer,
                                            &mesh.vertex_buffer_key,
                                        ),
                                        crate::CacheStorage::get(
                                            &managers.buffer_manager.w_buffer,
                                            &mesh.index_buffer_key,
                                        ),
                                    ) {
                                        let instance_data: Vec<_> = instance_batch
                                            .iter()
                                            .filter_map(|t| {
                                                let position = cgmath::Point3::new(
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
                                        crate::log_info!("instance count: {}", instance_data.len());
                                        if !instance_data.is_empty() {
                                            let instance_buffer = crate::WgpuBuffer::from_data(
                                                queue,
                                                device,
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
                                                wgpu::IndexFormat::Uint16,
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
                                    } else if let (
                                        Some(transform),
                                        Some(mesh_vertex_buffer),
                                        Some(mesh_index_buffer),
                                    ) = (
                                        self.transforms[i],
                                        crate::CacheStorage::get(
                                            &managers.buffer_manager.w_buffer,
                                            &mesh.vertex_buffer_key,
                                        ),
                                        crate::CacheStorage::get(
                                            &managers.buffer_manager.w_buffer,
                                            &mesh.index_buffer_key,
                                        ),
                                    ) {
                                        if !frustum.contains_sphere(
                                            cgmath::Point3::new(
                                                transform.matrix.w.x,
                                                transform.matrix.w.y,
                                                transform.matrix.w.z,
                                            ),
                                            0.1,
                                        ) {
                                            continue;
                                        }

                                        let instance_buffer = crate::WgpuBuffer::from_data(
                                            queue,
                                            device,
                                            &transform.data(),
                                            wgpu::BufferUsages::VERTEX
                                                | wgpu::BufferUsages::COPY_DST,
                                            Some(&format!("{} instance buffer", model.name)),
                                        );
                                        queue.write_buffer(
                                            &instance_buffer.buffer,
                                            0,
                                            bytemuck::cast_slice(&transform.data()),
                                        );
                                        rpass.set_index_buffer(
                                            mesh_index_buffer.buffer.slice(..),
                                            wgpu::IndexFormat::Uint16,
                                        );
                                        rpass.set_vertex_buffer(
                                            0,
                                            mesh_vertex_buffer.buffer.slice(..),
                                        );
                                        rpass
                                            .set_vertex_buffer(1, instance_buffer.buffer.slice(..));

                                        rpass.draw_indexed(0..mesh.index_count, 0, 0..1);
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
    pub fn update_transforms(&mut self, dt: f64, frustum: crate::camera::Frustum) {
        let delta = <cgmath::Quaternion<f32> as cgmath::Rotation3>::from_angle_z(cgmath::Deg(
            (dt * 90.0) as f32,
        ));

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
                let transform = super::Transform::from_components(pos, rot, scale);
                transforms[i] = Some(transform);

                if let Some(batch) = instance_batches.get_mut(&super::Entity(i)) {
                    for (source, target_transform) in batch.iter_mut() {
                        let in_sphere = frustum.contains_sphere(
                            cgmath::Point3::new(
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
