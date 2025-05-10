use crate::CacheStorage;

#[warn(dead_code)]
pub struct WgpuRenderer {
    depth_stencil_state: Option<wgpu::DepthStencilState>,
    hdr: crate::HDR,
}

impl WgpuRenderer {
    pub fn new(
        device:&wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> Result<Self, crate::EngineError> {
        let depth_stencil_state = crate::Texture::depth_stencil_state();
        let hdr =crate::PipelineManager::hdr(device, surface_config)?;

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
    pub fn pass(
        &self,
        managers: &mut crate::Managers,
        rpass: &mut wgpu::RenderPass,
        world: &crate::World,
        camera: &crate::camera::Camera,
        uniform_bind_group: &wgpu::BindGroup,
    ) {
        let mut bind_group_idx = 0;
    
        // === Projection Setup ===
        if let Some(projection) = world.projection() {
            projection.render(rpass, managers, camera);
        }
    
        // === Bind global uniforms ===
        rpass.set_bind_group(bind_group_idx, uniform_bind_group, &[]);
        bind_group_idx += 1;
    
        // === Projection Bind Group ===
        if let Some(projection) = world.projection() {
            if let Some(projection_bg) = managers
                .bind_group_manager
                .get(&projection.dst_shader_key)
            {
                rpass.set_bind_group(bind_group_idx, projection_bg.as_ref(), &[]);
                bind_group_idx += 1;
            }
        }
    
        // === Entity Rendering ===
        for (entity_id, maybe_renderable) in world.get_renderables().iter().enumerate() {
            let Some(rend) = maybe_renderable else { continue };
            if !rend.visible { continue };
            self.render_entity(rpass, managers, world, entity_id, rend, camera, bind_group_idx);
        }
    }
    
    fn render_entity(
        &self,
        rpass: &mut wgpu::RenderPass,
        managers: &mut crate::Managers,
        world: &crate::World,
        entity_id: usize,
        rend: &crate::Renderable,
        camera: &crate::camera::Camera,
        base_bind_group: u32,
    ) {
        let model = crate::CacheStorage::get(&managers.model_manager, &rend.model_key).unwrap();
    
        for mesh_instance in &model.meshes {
    
            let material = crate::CacheStorage::get(
                &managers.material_manager,
                &mesh_instance.material_key,
            )
            .unwrap();
    
            let pipeline = managers
                .pipeline_manager
                .render
                .get(&crate::CacheKey::from(material.name.as_str()))
                .unwrap();
    
            rpass.set_pipeline(pipeline);
    
            for (i, bind_group) in material.bind_groups.iter().enumerate() {
                rpass.set_bind_group((i + 1) as u32 + base_bind_group, bind_group.as_ref(), &[]);
            }
    
            self.draw_mesh_instance(
                rpass,
                managers,
                world,
                crate::Entity(entity_id),
                &mesh_instance,
                &model.name,
                camera,
            );
        }
    }
    
    fn draw_mesh_instance(
        &self,
        rpass: &mut wgpu::RenderPass,
        managers: &crate::Managers,
        world: &crate::World,
        entity: crate::Entity,
        mesh_instance: &crate::MeshInstance,
        model_name: &str,
        camera: &crate::camera::Camera,
    ) {
        let frustum = camera.frustum();
    
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
    
        let instance_buffer = if let Some(..) = world.instance.batches().get(&entity) {
            let instance_data = world.instance.raw_data_for(entity, Some(&frustum));
            if instance_data.is_empty() {
                return;
            }
    
            crate::WgpuBuffer::from_data(
                &managers.device,
                &instance_data,
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                Some(&format!("{model_name} instance batch buffer")),
            )
        } else {
            let transform = world.transforms[entity.0]
                .as_ref()
                .unwrap_or(&crate::Transform::default()).data();
            let data = bytemuck::bytes_of(&transform);
            crate::WgpuBuffer::from_data(
                &managers.device,
                data,
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                Some(&format!("{model_name} instance buffer")),
            )
        };
    
        rpass.set_index_buffer(index_buffer.get().slice(..), wgpu::IndexFormat::Uint32);
        rpass.set_vertex_buffer(0, vertex_buffer.get().slice(..));
        rpass.set_vertex_buffer(1, instance_buffer.get().slice(..));
        rpass.draw_indexed(0..mesh_instance.index_count, 0, 0..1); // or `..instance_count`
    }
    
    
}

impl crate::Renderer for WgpuRenderer {
    fn render(
        &self,
        managers: &mut crate::Managers,
        rpass: &mut wgpu::RenderPass,
        world: &crate::World,
        camera: &crate::camera::Camera,
        uniform_bind_group: &wgpu::BindGroup,
    ) {
       self.pass(managers, rpass, world, camera, uniform_bind_group);
    }
    
}
