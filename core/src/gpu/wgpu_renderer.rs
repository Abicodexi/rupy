use crate::CacheStorage;

#[warn(dead_code)]
pub struct WgpuRenderer {
    depth_stencil_state: Option<wgpu::DepthStencilState>,
    depth_texture: crate::Texture,
    hdr: crate::HDR,
}

impl WgpuRenderer {
    pub fn new(
        managers: &mut crate::Managers,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> Result<Self, crate::EngineError> {
        let depth_stencil_state = crate::Texture::depth_stencil_state();

        let hdr = managers
            .pipeline_manager
            .hdr(&managers.device, surface_config)?;

        let depth_texture = crate::Texture::new(
            &managers.device,
            wgpu::Extent3d {
                width: surface_config.width,
                height: surface_config.height,
                depth_or_array_layers: 1,
            },
            crate::Texture::DEPTH_FORMAT,
            1,
            wgpu::TextureViewDimension::D2,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            Some(wgpu::AddressMode::ClampToEdge),
            wgpu::FilterMode::Linear,
            Some(managers.device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual),
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            })),
            Some("Depth texture"),
        );
        Ok(WgpuRenderer {
            depth_stencil_state: Some(depth_stencil_state),
            depth_texture,
            hdr,
        })
    }

    pub fn set_depth_texture(&mut self, texture: crate::Texture) {
        self.depth_texture = texture;
    }
    pub fn depth_texture(&self) -> &crate::Texture {
        &self.depth_texture
    }

    pub fn depth_stencil_state(&self) -> &Option<wgpu::DepthStencilState> {
        &self.depth_stencil_state
    }
    pub fn compute_pass(&self, world: &crate::World, managers: &mut crate::Managers) {
        if let Some(projection) = world.projection() {
            projection.compute_projection(managers, Some("equirect projection compute pass"));
        }
    }
    pub fn hdr(&self, encoder: &mut wgpu::CommandEncoder, output: &wgpu::TextureView) {
        self.hdr.compute(encoder, output);
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
        let mut bg_idx = 0;
        if let Some(projection) = world.projection() {
            projection.render(rpass, managers, camera);
        }

        let frustum = &camera.frustum();
        rpass.set_bind_group(bg_idx, uniform_bind_group, &[]);
        bg_idx += 1;
        if let Some(projection) = world.projection() {
            if let Some(projection_bind_group) =
                managers.bind_group_manager.get(&projection.dst_shader_key)
            {
                rpass.set_bind_group(bg_idx, projection_bind_group.as_ref(), &[]);
                bg_idx += 1;
            }
        }

        for (entity, rend_opt) in world.get_renderables().iter().enumerate() {
            let rend = match rend_opt {
                Some(r) if r.visible => r,
                _ => continue,
            };
            let model = crate::CacheStorage::get(&managers.model_manager, &rend.model_key).unwrap();
            for mesh_instance in &model.meshes {
                if let Some(mesh) = <crate::MeshManager as crate::CacheStorage<crate::Mesh>>::get(
                    &managers.mesh_manager,
                    &mesh_instance.material_key,
                ) {
                    let material = crate::CacheStorage::get(
                        &managers.material_manager,
                        &mesh_instance.material_key,
                    )
                    .unwrap();
                    let pipeline = managers
                        .pipeline_manager
                        .render
                        .get(&crate::CacheKey::from(material.name.as_ref()))
                        .unwrap();

                    rpass.set_pipeline(&pipeline);
                    for (i, bg) in material.bind_groups.iter().enumerate() {
                        rpass.set_bind_group((i + 1) as u32 + bg_idx, bg.as_ref(), &[]);
                    }

                    if world
                        .instance
                        .batches()
                        .contains_key(&crate::Entity(entity))
                    {
                        let instance_data: Vec<_> = world
                            .instance
                            .raw_data_for(crate::Entity(entity), Some(frustum));
                        if !instance_data.is_empty() {
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
                            let instance_buffer = crate::WgpuBuffer::from_data(
                                &managers.device,
                                &instance_data,
                                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                                Some(&format!("{} instance batch buffer", model.name)),
                            );
                            rpass.set_index_buffer(
                                index_buffer.get().slice(..),
                                wgpu::IndexFormat::Uint32,
                            );
                            rpass.set_vertex_buffer(0, vertex_buffer.get().slice(..));
                            rpass.set_vertex_buffer(1, instance_buffer.get().slice(..));
                            rpass.draw_indexed(
                                0..mesh.index_count,
                                0,
                                0..instance_data.len() as u32,
                            );
                        }
                    } else {
                        let instance_buffer = if let Some(tr) = &world.transforms[entity] {
                            let ib = crate::WgpuBuffer::from_data(
                                &managers.device,
                                bytemuck::bytes_of(&tr.data()),
                                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                                Some(&format!("{} instance buffer", model.name)),
                            );
                            managers.queue.write_buffer(
                                &ib.get(),
                                0,
                                bytemuck::cast_slice(bytemuck::bytes_of(&tr.data())),
                            );
                            ib
                        } else {
                            let tr = crate::Transform::default();
                            let ib = crate::WgpuBuffer::from_data(
                                &managers.device,
                                &bytemuck::bytes_of(&crate::Transform::default().data()),
                                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                                Some(&format!("{} instance buffer", model.name)),
                            );
                            managers.queue.write_buffer(
                                &ib.get(),
                                0,
                                bytemuck::cast_slice(bytemuck::bytes_of(&tr.data())),
                            );
                            ib
                        };

                        let vertex_buffer = crate::CacheStorage::get(
                            &managers.buffer_manager.w_buffer,
                            &mesh.vertex_buffer_key,
                        )
                        .unwrap();
                        let index_buffer = crate::CacheStorage::get(
                            &managers.buffer_manager.w_buffer,
                            &mesh.index_buffer_key,
                        )
                        .unwrap();

                        rpass.set_index_buffer(
                            index_buffer.get().slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        rpass.set_vertex_buffer(0, vertex_buffer.get().slice(..));
                        rpass.set_vertex_buffer(1, instance_buffer.get().slice(..));

                        rpass.draw_indexed(0..mesh.index_count, 0, 0..1);
                    }
                }
            }
        }
    }
}
