use crate::{BindGroup, ModelManager, VertexInstance};

#[warn(dead_code)]
pub struct WgpuRenderer {
    hdr: crate::HDR,
}

impl WgpuRenderer {
    pub fn new(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> Result<Self, crate::EngineError> {
        let hdr = crate::PipelineManager::hdr(device, surface_config)?;

        Ok(WgpuRenderer { hdr })
    }

    pub fn compute_pass(&self, world: &crate::World, queue: &wgpu::Queue, device: &wgpu::Device) {
        let projection = world.projection();
        projection.compute_projection(queue, device, Some("equirect projection compute pass"));
    }
    pub fn final_blit_to_surface(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        hdr_texture: &crate::Texture,
        surface_view: &wgpu::TextureView,
    ) {
        let bind_group = crate::BindGroup::hdr(&device, hdr_texture, "final blit");

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
        scene_texture: &crate::Texture,
        hdr_fb: &super::FrameBuffer,
    ) {
        let bind_group = crate::BindGroup::hdr(&model_manager.device, scene_texture, "hdr input");

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

impl crate::Renderer for WgpuRenderer {
    fn render(
        &self,
        models: &mut ModelManager,
        rpass: &mut wgpu::RenderPass,
        world: &crate::World,
        camera: &crate::camera::Camera,
        light: &crate::Light,
        uniform_bind_group: &wgpu::BindGroup,
    ) {
        let projection = world.projection();
        rpass.set_bind_group(0, uniform_bind_group, &[]);
        rpass.set_bind_group(1, &projection.dst_bind_group, &[]);
        rpass.set_pipeline(&projection.dst_pipeline);
        rpass.draw(0..3, 0..1);
        rpass.set_bind_group(2, &models.materials.storage_bind_group, &[]);

        for (model_key, instances) in world.instance_batch(camera, models) {
            if instances.is_empty() {
                continue;
            }
            let (count, vdata) = VertexInstance::to_data(&instances);
            let instance_buffer = super::WgpuBuffer::from_data(
                &models.device,
                &vdata, // &[u8]
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                Some(&format!("{} instance buffer", model_key.id())),
            );
            let model = crate::CacheStorage::get(models, &model_key).unwrap();

            if let Some(mat) = &model.instance.material {
                let pipeline = &mat.pipeline;
                rpass.set_pipeline(pipeline);
                rpass.set_bind_group(3, mat.bind_group.as_ref(), &[]);
                let vertex_buffer = &model.instance.mesh.vertex_buffer;
                let index_buffer = &model.instance.mesh.index_buffer;
                let index_count = model.instance.mesh.index_count;
                rpass.set_vertex_buffer(0, vertex_buffer.get().slice(..));
                rpass.set_vertex_buffer(1, instance_buffer.get().slice(..));
                rpass.set_index_buffer(index_buffer.get().slice(..), wgpu::IndexFormat::Uint32);
                rpass.draw_indexed(0..index_count, 0, 0..count);
            }
        }
    }
}
