#[derive(Debug)]
pub struct EquirectProjection {
    pub src_shader_key: crate::CacheKey,
    pub dst_shader_key: crate::CacheKey,
    pub src_texture_key: crate::CacheKey,
    pub dst_texture_key: crate::CacheKey,
}

impl EquirectProjection {
    pub const DEST_SIZE: u32 = 1080;
    pub const NUM_WORKGROUPS: u32 = (Self::DEST_SIZE + 15) / 16;
    pub const DEPTH_OR_ARRAY_LAYERS: u32 = 6;
    pub fn new(
        managers: &mut crate::Managers,
        config: &wgpu::SurfaceConfiguration,
        src_shader: &str,
        dst_shader: &str,
        hdr_texture: &str,
        depth_stencil_state: &Option<wgpu::DepthStencilState>,
    ) -> Result<Self, crate::EngineError> {
        let path = &crate::Asset::resolve(&format!("hdr/{}", hdr_texture));
        let bytes = crate::Asset::read_bytes(&path)?;
        let (pixels, meta) = crate::Texture::decode_hdr(&bytes)?;

        let src = crate::Texture::new(
            &managers.device,
            wgpu::Extent3d {
                width: meta.width,
                height: meta.height,
                depth_or_array_layers: 1,
            },
            crate::Texture::HDR_FORMAT,
            1,
            wgpu::TextureViewDimension::D2,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            None,
            wgpu::FilterMode::Linear,
            None,
            Some(&format!("{} source texture", hdr_texture)),
        );

        let dst = crate::Texture::new(
            &managers.device,
            wgpu::Extent3d {
                width: Self::DEST_SIZE,
                height: Self::DEST_SIZE,
                depth_or_array_layers: Self::DEPTH_OR_ARRAY_LAYERS,
            },
            crate::Texture::HDR_FORMAT,
            1,
            wgpu::TextureViewDimension::Cube,
            wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            Some(wgpu::AddressMode::ClampToEdge),
            wgpu::FilterMode::Nearest,
            None,
            Some(&format!("{} destination texture", hdr_texture)),
        );

        managers.queue.write_texture(
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

        let dst_bind_group = crate::BindGroup::equirect_dst(&managers.device, &dst);
        let src_bind_group = crate::BindGroup::equirect_src(&managers.device, &src, &dst);

        let src_texture_key = crate::CacheKey::from(src.label.clone());
        let dst_texture_key = crate::CacheKey::from(dst.label.clone());

        let src_shader_key = crate::CacheKey::from(src_shader);
        let dst_shader_key = crate::CacheKey::from(dst_shader);

        let equirect_src_shader = managers
            .shader_manager
            .load(&managers.device, src_shader)
            .unwrap()
            .clone();

        let equirect_src_pipeline_layout =
            managers
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(&format!("{} layout", src_shader)),
                    bind_group_layouts: &[&crate::BindGroupLayouts::equirect_src()],
                    push_constant_ranges: &[],
                });

        crate::CacheStorage::get_or_create(
            &mut managers.pipeline_manager.compute,
            src_shader_key.clone(),
            || {
                managers
                    .device
                    .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                        label: Some(src_shader),
                        layout: Some(&equirect_src_pipeline_layout),
                        module: &equirect_src_shader,
                        entry_point: Some("compute_equirect_to_cubemap"),
                        compilation_options: Default::default(),
                        cache: None,
                    })
                    .into()
            },
        );
        let equirect_dst_shader = managers
            .shader_manager
            .load(&managers.device, dst_shader)
            .unwrap();

        let equirect_dst_layout =
            managers
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(&format!("{} layout", dst_shader)),
                    bind_group_layouts: &[
                        crate::BindGroupLayouts::camera(),
                        crate::BindGroupLayouts::equirect_dst(),
                    ],
                    push_constant_ranges: &[],
                });

        crate::CacheStorage::get_or_create(
            &mut managers.pipeline_manager.render,
            dst_shader_key.clone(),
            || {
                managers
                    .device
                    .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some(dst_shader),
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
                            cull_mode: Some(wgpu::Face::Back),
                            polygon_mode: wgpu::PolygonMode::Fill,
                            unclipped_depth: false,
                            conservative: false,
                        },
                        depth_stencil: depth_stencil_state.as_ref().cloned(),

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
            src_texture_key,
            dst_texture_key,
        })
    }

    pub fn compute_projection(&self, managers: &mut crate::Managers, label: Option<&str>) {
        if let (Some(projection_compute_pipeline), Some(src_bind_group)) = (
            crate::CacheStorage::get(&managers.pipeline_manager.compute, &self.src_shader_key),
            managers.bind_group_manager.bind_group_for(
                &managers.texture_manager,
                &self.src_texture_key,
                crate::BindGroupLayouts::equirect_src(),
            ),
        ) {
            let mut encoder =
                managers
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("compute encoder"),
                    });
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label,
                timestamp_writes: None,
            });

            pass.set_pipeline(&projection_compute_pipeline);
            pass.set_bind_group(0, src_bind_group.as_ref(), &[]);
            pass.dispatch_workgroups(Self::NUM_WORKGROUPS, Self::NUM_WORKGROUPS, 6);

            drop(pass);
            managers.queue.submit([encoder.finish()]);
        }
    }
    pub fn render(
        &self,
        rpass: &mut wgpu::RenderPass,
        managers: &crate::Managers,
        camera: &crate::camera::Camera,
    ) {
        if let (Some(equirect_projection_bind_group), Some(equirect_projection_pipeline)) = (
            managers
                .bind_group_manager
                .bind_group(&self.dst_texture_key),
            crate::CacheStorage::get(&managers.pipeline_manager.render, &self.dst_shader_key),
        ) {
            rpass.set_bind_group(0, camera.bind_group(), &[]);
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
