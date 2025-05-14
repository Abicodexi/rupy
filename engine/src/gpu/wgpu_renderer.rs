use crate::UnifiedVertexInstance;

#[warn(dead_code)]
pub struct WgpuRenderer {
    depth_stencil_state: Option<wgpu::DepthStencilState>,
    hdr: crate::HDR,
}

impl WgpuRenderer {
    pub fn new(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> Result<Self, crate::EngineError> {
        let depth_stencil_state = crate::Texture::depth_stencil_state();
        let hdr = crate::PipelineManager::hdr(device, surface_config)?;

        Ok(WgpuRenderer {
            depth_stencil_state: Some(depth_stencil_state),
            hdr,
        })
    }

    pub fn depth_stencil_state(&self) -> &Option<wgpu::DepthStencilState> {
        &self.depth_stencil_state
    }
    pub fn compute_pass(&self, world: &crate::World, managers: &mut crate::Managers) {
        if let Some(projection) = world.projection() {
            projection.compute_projection(managers, Some("equirect projection compute pass"));
        }
    }
    pub fn final_blit_to_surface(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        hdr_texture: &crate::Texture,
        surface_view: &wgpu::TextureView,
        managers: &crate::Managers,
    ) {
        let bind_group = crate::BindGroup::hdr(&managers.device, hdr_texture, "final blit");

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
        managers: &crate::Managers,
        scene_texture: &crate::Texture,
        hdr_fb: &super::FrameBuffer,
    ) {
        let bind_group = crate::BindGroup::hdr(&managers.device, scene_texture, "hdr input");

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
        managers: &mut crate::Managers,
        rpass: &mut wgpu::RenderPass,
        world: &crate::World,
        camera: &crate::camera::Camera,
        light: &crate::Light,
        uniform_bind_group: &wgpu::BindGroup,
    ) {
        // === Global Bind Groups ===
        let mut bind_group_idx = 0;
        rpass.set_bind_group(bind_group_idx, uniform_bind_group, &[]);
        bind_group_idx += 1;

        if let Some(projection) = world.projection() {
            rpass.set_bind_group(bind_group_idx, &projection.dst_bind_group, &[]);
            bind_group_idx += 1;
            rpass.set_pipeline(&projection.dst_pipeline);
            rpass.draw(0..3, 0..1);
        }

        for (model_key, instances) in world.instance_batch(camera) {
            if instances.is_empty() {
                continue;
            }
            let (count, data) = UnifiedVertexInstance::to_data(instances);
            let instance_buffer = super::WgpuBuffer::from_data(
                &managers.device,
                &data, // &[u8]
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                Some(&format!("{} instance buffer", model_key.id())),
            );

            // === Render All Meshes for This Model ===
            let model = crate::CacheStorage::get(&managers.model_manager, &model_key).unwrap();

            for mesh_instance in &model.meshes {
                let material = crate::CacheStorage::get(
                    &managers.material_manager,
                    &mesh_instance.material_key,
                )
                .unwrap();
                let pipeline = crate::CacheStorage::get(
                    &managers.pipeline_manager.render,
                    &crate::CacheKey::from(material.name.as_str()),
                )
                .unwrap();

                rpass.set_pipeline(pipeline);

                for (i, bind_group) in material.bind_groups.iter().enumerate() {
                    rpass.set_bind_group(i as u32 + bind_group_idx, bind_group.as_ref(), &[]);
                }

                let vertex_buffer = crate::CacheStorage::get(
                    &managers.buffer_manager.w_buffer,
                    &mesh_instance.vertex_buffer_key,
                )
                .unwrap();
                let index_buffer = crate::CacheStorage::get(
                    &managers.buffer_manager.w_buffer,
                    &mesh_instance.index_buffer_key,
                )
                .unwrap();
                rpass.set_vertex_buffer(0, vertex_buffer.get().slice(..));
                rpass.set_vertex_buffer(1, instance_buffer.get().slice(..));
                rpass.set_index_buffer(index_buffer.get().slice(..), wgpu::IndexFormat::Uint32);
                rpass.draw_indexed(0..mesh_instance.index_count, 0, 0..count);
            }
        }
    }
}
