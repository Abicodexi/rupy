use {
    super::{DebugMode, PipelineManager, RenderPass, VertexInstance, AABB, HDR},
    crate::{
        camera::{self, Frustum},
        BindGroup, CacheKey, CacheStorage, EngineError, FrameBuffer, ModelManager, Rotation, Scale,
        Texture, Transform, WgpuBuffer, World,
    },
    glam::{Mat4, Vec3},
    wgpu::IndexFormat,
};

#[warn(dead_code)]
pub struct Renderer3d {
    hdr: HDR,
    pub instances: InstanceBuffers,
}

impl Renderer3d {
    pub fn new(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> Result<Self, EngineError> {
        let hdr = PipelineManager::hdr(device, surface_config)?;
        let instances = InstanceBuffers::new();

        Ok(Renderer3d { hdr, instances })
    }

    pub fn compute_pass(&self, world: &World, queue: &wgpu::Queue, device: &wgpu::Device) {
        let projection = world.projection();
        projection.compute_projection(queue, device, Some("equirect projection compute pass"));
    }
    pub fn final_blit_to_surface(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        hdr_texture: &Texture,
        surface_view: &wgpu::TextureView,
    ) {
        let bind_group = BindGroup::hdr(&device, hdr_texture, "final blit");

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Final Blit to Surface"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.hdr.pipeline());
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }

    pub fn hdr(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        model_manager: &ModelManager,
        scene_texture: &Texture,
        hdr_fb: &FrameBuffer,
    ) {
        let bind_group = BindGroup::hdr(&model_manager.device, scene_texture, "hdr input");

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("HDR Pass"),
            color_attachments: &[Some(hdr_fb.color_attachment())],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.hdr.pipeline());
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

impl RenderPass for Renderer3d {
    fn render(
        &self,
        models: &mut ModelManager,
        rpass: &mut wgpu::RenderPass,
        world: &World,
        uniform_bind_group: &wgpu::BindGroup,
        debug_mode: &DebugMode,
    ) {
        let projection = world.projection();

        rpass.set_bind_group(0, uniform_bind_group, &[]);
        rpass.set_bind_group(1, &projection.dst_bind_group, &[]);
        rpass.set_bind_group(2, &models.materials.storage_bind_group, &[]);

        rpass.set_pipeline(&projection.dst_pipeline);
        rpass.draw(0..3, 0..1);

        self.instances
            .draw(rpass, models, debug_mode, uniform_bind_group);

        {
            for instance in world.terrain.mesh_instances() {
                let Some(mat) = instance.material.as_ref() else {
                    continue;
                };

                let Some(instance_buffer) = world.terrain.instance_buffer() else {
                    continue;
                };

                let mesh = &instance.mesh;

                rpass.set_bind_group(3, mat.bind_group.as_ref(), &[]);

                rpass.set_vertex_buffer(0, mesh.vertex_buffer.get().slice(..));
                rpass.set_vertex_buffer(1, instance_buffer.buffer.get().slice(..));

                rpass.set_index_buffer(mesh.index_buffer.get().slice(..), IndexFormat::Uint32);

                if debug_mode.mode() > 0 {
                    rpass.set_bind_group(0, debug_mode.bind_group(), &[]);
                    rpass.set_pipeline(debug_mode.pipeline());
                    rpass.draw_indexed(0..mesh.index_count, 0, 0..instance_buffer.count as u32);
                } else {
                    rpass.set_bind_group(0, uniform_bind_group, &[]);
                    rpass.set_pipeline(&mat.pipeline);
                    rpass.draw_indexed(0..mesh.index_count, 0, 0..instance_buffer.count as u32);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct InstanceBufferData {
    pub buffer: WgpuBuffer,
    pub count: usize,
    pub capacity: usize,
    pub dirty: bool,
}

#[derive(Debug)]
pub struct InstanceBuffers {
    pub batch: std::collections::HashMap<CacheKey, Vec<VertexInstance>>,
    pub buffers: std::collections::HashMap<CacheKey, InstanceBufferData>,
}

impl InstanceBuffers {
    pub fn new() -> Self {
        Self {
            batch: std::collections::HashMap::new(),
            buffers: std::collections::HashMap::new(),
        }
    }

    pub fn update(
        &mut self,
        world: &World,
        camera: &camera::Camera,
        model_manager: &mut ModelManager,
    ) {
        let frustum = camera.frustum();
        self.batch.clear();

        let default_scale = Scale::one();
        let default_rotation = Rotation::zero();

        for idx in 0..world.entity_count() {
            let Some(renderable) = &world.renderables[idx] else {
                continue;
            };
            let Some(position) = &world.physics.positions[idx] else {
                continue;
            };

            if !renderable.visible {
                continue;
            }

            let rotation = world.rotations[idx].as_ref().unwrap_or(&default_rotation);
            let scale = world.scales[idx].as_ref().unwrap_or(&default_scale);

            let transform = Transform::from_components(position, rotation, scale);

            if let Some(model) = model_manager.models.get(&renderable.model_key) {
                if !frustum_cull_aabb(&frustum, &model.aabb, &transform.model_matrix) {
                    continue;
                }
                if let Some(material) = &model.instance.material {
                    let data = transform.to_vertex_instance(material.idx);
                    self.batch
                        .entry(renderable.model_key)
                        .or_default()
                        .push(data);
                }
            }
        }

        for (key, data) in &self.batch {
            let instances = data;

            let byte_data = VertexInstance::bytes(instances);
            let byte_size = data.len();
            let buffer_data = self
                .buffers
                .entry(*key)
                .or_insert_with(|| InstanceBufferData {
                    buffer: WgpuBuffer::from_data(
                        &model_manager.device,
                        &byte_data,
                        wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        Some(&format!(" instance buffer {}", key.id())),
                    ),
                    count: instances.len(),
                    capacity: byte_size,
                    dirty: false,
                });

            buffer_data.count = instances.len();
            buffer_data.dirty = true;
        }
    }

    pub fn upload(&mut self, queue: &wgpu::Queue, device: &wgpu::Device) {
        for (key, data) in &mut self.buffers {
            if let Some(instances) = self.batch.get(key) {
                if data.dirty {
                    let byte_data = VertexInstance::bytes(instances);
                    data.buffer.write_data(queue, device, &byte_data, Some(0));
                    data.dirty = false;
                    data.count = instances.len();
                }
            }
        }
    }

    pub fn draw(
        &self,
        rpass: &mut wgpu::RenderPass,
        models: &ModelManager,
        debug: &DebugMode,
        uniform_bind_group: &wgpu::BindGroup,
    ) {
        for (model_key, data) in &self.buffers {
            if data.count == 0 {
                continue;
            }

            let Some(model) = models.get(model_key) else {
                continue;
            };
            let Some(mat) = &model.instance.material else {
                continue;
            };

            let mesh = &model.instance.mesh;

            rpass.set_bind_group(3, mat.bind_group.as_ref(), &[]);

            rpass.set_vertex_buffer(0, mesh.vertex_buffer.get().slice(..));
            rpass.set_vertex_buffer(1, data.buffer.get().slice(..));
            rpass.set_index_buffer(mesh.index_buffer.get().slice(..), IndexFormat::Uint32);

            if debug.mode() > 0 {
                rpass.set_bind_group(0, debug.bind_group(), &[]);
                rpass.set_pipeline(debug.pipeline());
                rpass.draw_indexed(0..mesh.index_count, 0, 0..data.count as u32);
            } else {
                rpass.set_bind_group(0, uniform_bind_group, &[]);
                rpass.set_pipeline(&mat.pipeline);
                rpass.draw_indexed(0..mesh.index_count, 0, 0..data.count as u32);
            }
        }
    }
}

pub fn frustum_cull_aabb(frustum: &Frustum, aabb: &AABB, model_matrix: &Mat4) -> bool {
    let corners = [
        Vec3::new(aabb.min.x, aabb.min.y, aabb.min.z),
        Vec3::new(aabb.min.x, aabb.min.y, aabb.max.z),
        Vec3::new(aabb.min.x, aabb.max.y, aabb.min.z),
        Vec3::new(aabb.min.x, aabb.max.y, aabb.max.z),
        Vec3::new(aabb.max.x, aabb.min.y, aabb.min.z),
        Vec3::new(aabb.max.x, aabb.min.y, aabb.max.z),
        Vec3::new(aabb.max.x, aabb.max.y, aabb.min.z),
        Vec3::new(aabb.max.x, aabb.max.y, aabb.max.z),
    ];
    for plane in frustum.planes.iter() {
        if corners
            .iter()
            .all(|corner| plane.distance(model_matrix.transform_point3(*corner)) < 0.0)
        {
            return false;
        }
    }
    return true;
}
