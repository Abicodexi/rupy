use crate::CacheStorage;

#[warn(dead_code)]
pub struct WgpuRenderer {
    depth_stencil_state: Option<wgpu::DepthStencilState>,
    depth_texture: crate::Texture,
    hdr_pipeline: wgpu::RenderPipeline,
    hdr_texture: crate::Texture,
    hdr_bind_group: wgpu::BindGroup,
}

impl WgpuRenderer {
    pub fn new(
        managers: &mut crate::Managers,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> Result<Self, crate::EngineError> {
        let depth_stencil_state = wgpu::DepthStencilState {
            format: crate::Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };
        let hdr_sampler = managers.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let hdr_texture = crate::Texture::new(
            &managers.device,
            wgpu::Extent3d {
                width: surface_config.width,
                height: surface_config.height,
                depth_or_array_layers: 1,
            },
            surface_config.format.add_srgb_suffix(),
            1,
            wgpu::TextureViewDimension::D2,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            Some(wgpu::AddressMode::ClampToEdge),
            wgpu::FilterMode::Nearest,
            Some(hdr_sampler),
            Some("hdr"),
        );
        let hdr_bind_group = crate::BindGroup::hdr(&managers.device, &hdr_texture, "hdr");
        let hdr_pipeline_layout =
            managers
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("hdr pipeline layout"),
                    bind_group_layouts: &[&crate::BindGroupLayouts::texture()],
                    push_constant_ranges: &[],
                });
        let hdr_shader = crate::Asset::shader(managers, "hdr.wgsl")?;
        let hdr_pipeline =
            managers
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(&hdr_texture.label),
                    layout: Some(&hdr_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &hdr_shader,
                        entry_point: Some("vs_main"),
                        buffers: &[],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &hdr_shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: surface_config.format.add_srgb_suffix(),
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,

                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                    cache: None,
                });

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
            hdr_texture,
            hdr_bind_group,
            hdr_pipeline,
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
    pub fn process_hdr(&self, encoder: &mut wgpu::CommandEncoder, output: &wgpu::TextureView) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("hdr compute pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.hdr_pipeline);
        pass.set_bind_group(0, &self.hdr_bind_group, &[]);
        pass.draw(0..3, 0..1);
        drop(pass);
    }
}

impl crate::Renderer for WgpuRenderer {
    fn render(
        &self,
        managers: &mut crate::Managers,
        rpass: &mut wgpu::RenderPass,
        world: &crate::World,
        camera: &crate::camera::Camera,
    ) {
        if let Some(projection) = world.projection() {
            projection.render(rpass, managers, camera);
        }

        let frustum = &camera.frustum();

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
                        .render_pipeline_manager
                        .get(&crate::CacheKey::from(material.name.clone()))
                        .unwrap();

                    rpass.set_pipeline(&pipeline);
                    for (i, bg) in material.bind_groups.iter().enumerate() {
                        rpass.set_bind_group(i as u32, bg.as_ref(), &[]);
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
