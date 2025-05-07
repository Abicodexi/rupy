#[derive(Debug)]
pub struct EquirectProjection {
    pub src_shader_key: crate::CacheKey,
    pub dst_shader_key: crate::CacheKey,
    pub dst_pipeline_key: crate::CacheKey,
    pub src_pipeline_key: crate::CacheKey,
    pub src_texture_key: crate::CacheKey,
    pub dst_texture_key: crate::CacheKey,

    pub hdr_rel_path: String,
    pub dst_size: u32,
    pub format: wgpu::TextureFormat,
}

impl EquirectProjection {
    pub fn new(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        managers: &mut crate::Managers,
        config: &wgpu::SurfaceConfiguration,
        src_shader: &str,
        dst_shader: &str,
        hdr_rel_path: &str,
        dst_size: u32,
        format: wgpu::TextureFormat,
    ) -> Result<Self, crate::EngineError> {
        let src_texture_key = crate::CacheKey::new("equirect projection source texture");
        let dst_texture_key = crate::CacheKey::new("equirect projection destination texture");
        let src_pipeline_key =
            crate::CacheKey::new(&format!("{} compute pipeline", src_texture_key.id));
        let dst_pipeline_key =
            crate::CacheKey::new(&format!("{} render pipeline", dst_texture_key.id));
        let src_shader_key = crate::CacheKey::from(src_shader);
        let dst_shader_key = crate::CacheKey::from(dst_shader);
        let compute_pipeline_entry = "compute_equirect_to_cubemap";

        let equirect_src_shader =
            if !crate::CacheStorage::contains(&managers.shader_manager, &src_shader_key) {
                let shader_module = crate::Asset::shader(managers, &src_shader).expect(&format!(
                    "AssetLoader load shader failed for {}",
                    src_shader
                ));
                shader_module
            } else {
                crate::CacheStorage::get(&managers.shader_manager, &src_shader_key)
                    .unwrap()
                    .clone()
            };

        let equirect_src_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("{} layout", src_pipeline_key.id)),
                bind_group_layouts: &[&crate::BindGroupLayouts::equirect_src()],
                push_constant_ranges: &[],
            });

        crate::CacheStorage::get_or_create(
            &mut managers.compute_pipeline_manager,
            src_pipeline_key.clone(),
            || {
                device
                    .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                        label: Some(&src_pipeline_key.id),
                        layout: Some(&equirect_src_pipeline_layout),
                        module: &equirect_src_shader,
                        entry_point: Some(compute_pipeline_entry),
                        compilation_options: Default::default(),
                        cache: None,
                    })
                    .into()
            },
        );
        let equirect_dst_shader =
            if !crate::CacheStorage::contains(&managers.shader_manager, &dst_shader_key) {
                let shader_module = crate::Asset::shader(managers, &dst_shader).expect(&format!(
                    "AssetLoader load shader failed for {}",
                    dst_shader
                ));
                shader_module
            } else {
                crate::CacheStorage::get(&managers.shader_manager, &dst_shader_key)
                    .unwrap()
                    .clone()
            };

        let equirect_dst_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{} layout", dst_pipeline_key.id)),
            bind_group_layouts: &[
                crate::BindGroupLayouts::camera(),
                crate::BindGroupLayouts::equirect_dst(),
            ],
            push_constant_ranges: &[],
        });
        crate::CacheStorage::get_or_create(
            &mut managers.render_pipeline_manager,
            dst_pipeline_key.clone(),
            || {
                device
                    .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some(&dst_pipeline_key.id),
                        layout: Some(&equirect_dst_layout),
                        vertex: wgpu::VertexState {
                            module: &equirect_dst_shader,
                            entry_point: Some("vs_main"),
                            buffers: &[],
                            compilation_options: Default::default(),
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &equirect_dst_shader,
                            entry_point: Some("fs_main"),
                            targets: &[Some(wgpu::ColorTargetState {
                                format: config.format,
                                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                            compilation_options: Default::default(),
                        }),
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: None,
                            polygon_mode: wgpu::PolygonMode::Fill,
                            unclipped_depth: false,
                            conservative: false,
                        },
                        depth_stencil: Some(managers.texture_manager.depth_stencil_state.clone()),

                        multisample: wgpu::MultisampleState {
                            count: 1,
                            mask: !0,
                            alpha_to_coverage_enabled: false,
                        },
                        multiview: None,
                        cache: None,
                    })
                    .into()
            },
        );

        let path = &crate::Asset::resolve(&format!("hdr\\{}", hdr_rel_path));
        let bytes = crate::Asset::read_bytes(&path)?;
        let (pixels, meta) = crate::Texture::decode_hdr(&bytes)?;

        let src = crate::Texture::new(
            &device,
            wgpu::Extent3d {
                width: meta.width,
                height: meta.height,
                depth_or_array_layers: 1,
            },
            format,
            1,
            wgpu::TextureViewDimension::D2,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            None,
            wgpu::FilterMode::Linear,
            None,
            Some(&format!("{} source texture", hdr_rel_path)),
        );

        let dst = crate::Texture::new(
            &device,
            wgpu::Extent3d {
                width: dst_size,
                height: dst_size,
                depth_or_array_layers: 6,
            },
            format,
            1,
            wgpu::TextureViewDimension::Cube,
            wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            Some(wgpu::AddressMode::ClampToEdge),
            wgpu::FilterMode::Nearest,
            None,
            Some(&format!("{} destination texture", hdr_rel_path)),
        );

        let src_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: crate::BindGroupLayouts::equirect_src(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&dst.create_projection_view()),
                },
            ],
            label: Some(&format!("{} bind group", src_texture_key.id,)),
        });

        let dst_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: crate::BindGroupLayouts::equirect_dst(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&dst.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&dst.sampler),
                },
            ],
            label: Some(&format!("{} bind group", dst_texture_key.id)),
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &src.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&pixels),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(meta.width * std::mem::size_of::<[f32; 4]>() as u32),
                rows_per_image: Some(meta.height),
            },
            src.texture.size(),
        );

        crate::CacheStorage::insert(
            &mut managers.bind_group_manager,
            src_texture_key.clone(),
            src_bind_group.into(),
        );
        crate::CacheStorage::insert(
            &mut managers.bind_group_manager,
            dst_texture_key.clone(),
            dst_bind_group.into(),
        );

        crate::CacheStorage::insert(
            &mut managers.texture_manager,
            src_texture_key.clone(),
            src.into(),
        );
        crate::CacheStorage::insert(
            &mut managers.texture_manager,
            src_texture_key.clone(),
            dst.into(),
        );

        Ok(EquirectProjection {
            src_shader_key,
            dst_shader_key,
            dst_pipeline_key,
            src_pipeline_key,
            src_texture_key,
            dst_texture_key,
            hdr_rel_path: hdr_rel_path.to_string(),
            dst_size,
            format,
        })
    }

    pub fn compute_projection(
        &self,
        queue: &wgpu::Queue,
        mut encoder: wgpu::CommandEncoder,
        managers: &mut crate::Managers,
        label: Option<&str>,
    ) {
        if let (Some(projection_compute_pipeline), Some(src_bind_group)) = (
            crate::CacheStorage::get(&managers.compute_pipeline_manager, &self.src_pipeline_key),
            managers.bind_group_manager.bind_group_for(
                &managers.texture_manager,
                &self.src_texture_key.id,
                crate::BindGroupLayouts::equirect_src(),
            ),
        ) {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label,
                timestamp_writes: None,
            });

            let num_workgroups = (self.dst_size + 15) / 16;
            pass.set_pipeline(&projection_compute_pipeline);
            pass.set_bind_group(0, src_bind_group.as_ref(), &[]);
            pass.dispatch_workgroups(num_workgroups, num_workgroups, 6);

            drop(pass);
            queue.submit([encoder.finish()]);
        }
    }
    pub fn render(
        &self,
        rpass: &mut wgpu::RenderPass,
        managers: &crate::Managers,
        camera: &crate::camera::Camera,
    ) {
        if let (
            Some(equirect_projection_bind_group),
            Some(camera_bind_group),
            Some(equirect_projection_pipeline),
        ) = (
            managers
                .bind_group_manager
                .bind_group(&self.dst_texture_key.id),
            managers
                .bind_group_manager
                .bind_group(&camera.bind_group.id),
            crate::CacheStorage::get(&managers.render_pipeline_manager, &self.dst_pipeline_key),
        ) {
            rpass.set_bind_group(0, camera_bind_group.as_ref(), &[]);
            rpass.set_bind_group(1, equirect_projection_bind_group.as_ref(), &[]);
            rpass.set_pipeline(&equirect_projection_pipeline);
            rpass.draw(0..3, 0..1);
        }
    }
}
#[derive(Debug)]
pub struct Environment {
    equirect_projection: EquirectProjection,
}

impl Environment {
    pub fn new(equirect_projection: EquirectProjection) -> Self {
        Self {
            equirect_projection,
        }
    }
    pub fn render(
        &self,
        rpass: &mut wgpu::RenderPass,
        managers: &mut crate::Managers,
        camera: &crate::camera::Camera,
    ) {
        self.equirect_projection.render(rpass, managers, camera);
    }
    pub fn projection(&self) -> &EquirectProjection {
        &self.equirect_projection
    }
}
