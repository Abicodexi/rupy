use {
    super::{RenderPass, VertexInstance, AABB},
    crate::{
        camera::{self, Frustum},
        create_render_pipeline, AssetService, BindGroup, CacheKey, CacheStorage, DebugMode,
        EngineError, FrameBuffer, MaterialManager, ModelManager, RenderBindGroupLayouts, Rotation,
        Scale, Texture, Transform, WgpuBuffer, World,
    },
    glam::{Mat4, Vec3},
    rayon::iter::{IntoParallelRefIterator, ParallelIterator},
    std::{
        hash::{DefaultHasher, Hash, Hasher},
        sync::Arc,
    },
    wgpu::{IndexFormat, RenderPipeline},
};

#[warn(dead_code)]
pub struct Renderer3d {
    pub instances: InstanceBuffers,
    pub hdr_pipeline: RenderPipeline,
}

impl Renderer3d {
    pub fn new(
        service: &'static Arc<AssetService>,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> Result<Self, EngineError> {
        let instances: InstanceBuffers = InstanceBuffers::new();
        let v_shader = "hdr.vert.wgsl";
        let f_shader = "hdr.frag.wgsl";

        let label = "hdr pipeline";
        let pipeline_layout =
            service
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(&format!("{} layout", label)),
                    bind_group_layouts: &[service.bind_group_layouts().texture()],
                    push_constant_ranges: &[],
                });
        let hdr_pipeline = create_render_pipeline(
            &service,
            f_shader,
            v_shader,
            pipeline_layout,
            &[],
            surface_config.format,
            None,
            label.to_string(),
        )?;

        Ok(Renderer3d {
            instances,
            hdr_pipeline,
        })
    }

    pub fn compute_pass(&self, world: &World, queue: &wgpu::Queue, device: &wgpu::Device) {
        let projection = world.projection();
        projection.compute_projection(queue, device, Some("equirect projection compute pass"));
    }
    pub fn final_blit_to_surface(
        &self,
        device: &wgpu::Device,
        bind_group_layouts: &RenderBindGroupLayouts,
        encoder: &mut wgpu::CommandEncoder,
        hdr_texture: &Texture,
        surface_view: &wgpu::TextureView,
    ) {
        let bind_group = BindGroup::hdr(&device, bind_group_layouts, hdr_texture, "final blit");

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

        pass.set_pipeline(&self.hdr_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }

    pub fn hdr(
        &self,
        device: &wgpu::Device,
        bind_group_layouts: &RenderBindGroupLayouts,

        encoder: &mut wgpu::CommandEncoder,
        scene_texture: &Texture,
        hdr_fb: &FrameBuffer,
    ) {
        let bind_group = BindGroup::hdr(device, bind_group_layouts, scene_texture, "hdr input");

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("HDR Pass"),
            color_attachments: &[Some(hdr_fb.color_attachment())],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.hdr_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

impl RenderPass for Renderer3d {
    fn render(
        &self,
        models: &ModelManager,
        materials: &MaterialManager,
        rpass: &mut wgpu::RenderPass,
        world: &World,
        uniform_bind_group: &wgpu::BindGroup,
        debug_mode: &DebugMode,
    ) {
        let projection = world.projection();

        rpass.set_bind_group(0, uniform_bind_group, &[]);
        rpass.set_bind_group(1, projection.dst_bind_group.as_ref(), &[]);
        rpass.set_bind_group(2, materials.storage().bind_group(), &[]);

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
fn hash_vertex_instances(instances: &Vec<VertexInstance>) -> u64 {
    use std::hash::Hash;
    let mut hasher = DefaultHasher::new();
    Hash::hash(instances, &mut hasher);
    let hash = hasher.finish();
    hash
}
#[derive(Debug)]
pub struct InstanceBuffer {
    buffer: WgpuBuffer,
    count: usize,
    hash: u64,
    dirty: bool,
}
impl InstanceBuffer {
    pub fn new(buffer: WgpuBuffer, instances: Option<&Vec<VertexInstance>>) -> Self {
        Self {
            buffer,
            count: instances.unwrap_or(Vec::new().as_ref()).len(),
            hash: hash_vertex_instances(instances.unwrap_or(Vec::new().as_ref())),
            dirty: false,
        }
    }
    pub fn write_data<T>(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        data: &[T],
        offset: Option<u64>,
    ) where
        T: bytemuck::Pod,
    {
        self.buffer.write_data(queue, device, data, offset);
    }
}

#[derive(Debug)]
pub struct InstanceBuffers {
    batch: std::collections::HashMap<CacheKey, Vec<VertexInstance>>,
    buffers: std::collections::HashMap<CacheKey, InstanceBuffer>,
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
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        world: &World,
        camera: &camera::Camera,
        models: &ModelManager,
    ) {
        let frustum = camera.frustum();
        self.batch.clear();

        let default_scale = Scale::one();
        let default_rotation = Rotation::zero();

        for idx in 0..world.entity_count() {
            let Some(renderable) = &world.renderables[idx] else {
                continue;
            };
            if !renderable.visible {
                continue;
            }

            let Some(position) = &world.physics.positions[idx] else {
                continue;
            };
            let rotation = world.rotations[idx].as_ref().unwrap_or(&default_rotation);
            let scale = world.scales[idx].as_ref().unwrap_or(&default_scale);

            let transform = Transform::from_components(position, rotation, scale);

            if let Some(model) = models.get_resource(&renderable.model_key) {
                if !frustum.frustum_cull_aabb(&model.aabb, &transform.model_matrix) {
                    continue;
                }
                if let Some(material) = &model.instance.material {
                    let data =
                        transform.to_vertex_instance(material.storage_id.unwrap_or(0) as u32);
                    self.batch
                        .entry(renderable.model_key)
                        .or_default()
                        .push(data);
                }
            }
        }
        let updates: Vec<(CacheKey, u64, usize, Vec<u8>)> = if self.batch.len() >= 10 {
            self.batch
                .par_iter()
                .map(|(key, instances)| {
                    let mut hasher = DefaultHasher::new();
                    instances.iter().for_each(|i| i.hash(&mut hasher));
                    let hash = hasher.finish();
                    let count = instances.len();
                    let bytes = VertexInstance::bytes(instances);
                    (*key, hash, count, bytes)
                })
                .collect()
        } else {
            self.batch
                .iter()
                .map(|(key, instances)| {
                    let mut hasher = DefaultHasher::new();
                    instances.iter().for_each(|i| i.hash(&mut hasher));
                    let hash = hasher.finish();
                    let count = instances.len();
                    let bytes = VertexInstance::bytes(instances);
                    (*key, hash, count, bytes)
                })
                .collect()
        };

        for (key, hash, count, byte_data) in updates {
            let instance = self.buffers.entry(key).or_insert_with(|| InstanceBuffer {
                buffer: WgpuBuffer::from_data(
                    device,
                    &byte_data,
                    wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    Some(&format!("instance buffer {}", key.id())),
                ),
                count,
                dirty: true,
                hash,
            });

            if instance.hash != hash || instance.count != count {
                instance.dirty = true;
                instance.hash = hash;
                instance.count = count;
                instance.buffer.write_data(queue, device, &byte_data, None);
            }
        }
    }

    pub fn upload(&mut self, queue: &wgpu::Queue, device: &wgpu::Device) {
        for (key, buffer) in &mut self.buffers {
            if buffer.dirty {
                if let Some(instances) = self.batch.get(key) {
                    let byte_data = VertexInstance::bytes(instances);
                    buffer.buffer.write_data(queue, device, &byte_data, Some(0));
                    buffer.dirty = false;
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

            let Some(model) = models.get_resource(model_key) else {
                continue;
            };
            let Some(mat) = &model.instance.material else {
                continue;
            };

            let mesh = &model.instance.mesh;
            let count = data.count as u32;
            rpass.set_bind_group(3, mat.bind_group.as_ref(), &[]);

            rpass.set_vertex_buffer(0, mesh.vertex_buffer.get().slice(..));
            rpass.set_vertex_buffer(1, data.buffer.get().slice(..));
            rpass.set_index_buffer(mesh.index_buffer.get().slice(..), IndexFormat::Uint32);

            if debug.mode() > 0 {
                rpass.set_bind_group(0, debug.bind_group(), &[]);
                rpass.set_pipeline(debug.pipeline());
                rpass.draw_indexed(0..mesh.index_count, 0, 0..count);
            } else {
                rpass.set_bind_group(0, uniform_bind_group, &[]);
                rpass.set_pipeline(&mat.pipeline);
                rpass.draw_indexed(0..mesh.index_count, 0, 0..count);
            }
        }
    }
}
