use crate::{
    texture::{Texture, TextureManager},
    AssetLoader, BindGroupLayouts, CacheKey, CacheStorage, EngineError, GpuContext, Managers,
    Resources,
};
use std::sync::Arc;
use wgpu::SurfaceConfiguration;

pub struct EquirectProjection {
    pub src_shader_key: CacheKey,
    pub dst_shader_key: CacheKey,
    pub dst_pipeline_key: CacheKey,
    pub src_pipeline_key: CacheKey,
    pub src_texture_key: CacheKey,
    pub dst_texture_key: CacheKey,
    pub hdr_rel_path: String,
    pub dst_size: u32,
    pub format: wgpu::TextureFormat,
}

impl EquirectProjection {
    pub fn new(
        gpu: &GpuContext,
        asset_loader: &AssetLoader,
        managers: &mut Managers,
        config: &SurfaceConfiguration,
        bind_group_layouts: &BindGroupLayouts,
        src_shader_key: CacheKey,
        dst_shader_key: CacheKey,
        hdr_rel_path: &str,
        dst_size: u32,
        format: wgpu::TextureFormat,
    ) -> Result<Self, EngineError> {
        let equirect_src_shader =
            managers
                .shader_manager
                .get_or_create(src_shader_key.clone(), || {
                    let shader_module = asset_loader.load_shader(&src_shader_key.id)?;
                    Ok(Arc::new(shader_module))
                });

        let equirect_src_pipeline_layout =
            gpu.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Equirect src pipeline layout"),
                    bind_group_layouts: &[&bind_group_layouts.equirect_src],
                    push_constant_ranges: &[],
                });
        let src_pipeline_key = CacheKey::from("src_equirect");

        managers
            .pipeline_manager
            .get_or_create_compute_pipeline(src_pipeline_key.clone(), || {
                Ok(gpu
                    .device()
                    .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                        label: Some("Equirect src pipeline"),
                        layout: Some(&equirect_src_pipeline_layout),
                        module: &equirect_src_shader,
                        entry_point: Some("compute_equirect_to_cubemap"),
                        compilation_options: Default::default(),
                        cache: None,
                    })
                    .into())
            });
        let equirect_dst_shader =
            managers
                .shader_manager
                .get_or_create(dst_shader_key.clone(), || {
                    let shader_module = asset_loader.load_shader(&dst_shader_key.id)?;
                    Ok(Arc::new(shader_module))
                });

        let equirect_dst_layout =
            gpu.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Equirect dst pipeline layout"),
                    bind_group_layouts: &[
                        &bind_group_layouts.camera,
                        &bind_group_layouts.equirect_dst,
                    ],
                    push_constant_ranges: &[],
                });
        let dst_pipeline_key = CacheKey::from("dst_equirect");
        managers
            .pipeline_manager
            .get_or_create_render_pipeline(dst_pipeline_key.clone(), || {
                Ok(gpu
                    .device()
                    .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("Equirect dst pipeline"),
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
                    .into())
            });

        Ok(EquirectProjection {
            src_shader_key,
            dst_shader_key,
            dst_pipeline_key,
            src_pipeline_key,
            src_texture_key: CacheKey::new("equirect_projection_src"),
            dst_texture_key: CacheKey::new("equirect_projection_dst"),
            hdr_rel_path: hdr_rel_path.to_string(),
            dst_size,
            format,
        })
    }

    pub fn prepare(
        managers: &mut Managers,
        resources: &Resources,
        bind_group_layouts: &BindGroupLayouts,
        rel_path: &str,
        dst_size: u32,
        format: wgpu::TextureFormat,
    ) -> Result<(CacheKey, wgpu::BindGroup, CacheKey, wgpu::BindGroup), EngineError> {
        let path = &resources
            .asset_loader
            .resolve(&format!("hdr\\{}", rel_path));
        let bytes = AssetLoader::read_bytes(&path)?;
        let (pixels, meta) = TextureManager::decode_hdr(&bytes)?;

        let src_key = CacheKey::new("equirect_projection_src");
        let src = Texture::create(
            &resources.gpu.device,
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
            Some(&format!("src:{}", rel_path)),
        );

        let dst_key = CacheKey::new("equirect_projection_dst");
        let dst = Texture::create(
            &resources.gpu.device,
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
            Some(&format!("dst:{}", rel_path)),
        );

        let equirect_src_bind_group =
            resources
                .gpu
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &bind_group_layouts.equirect_src,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&src.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(
                                &dst.create_projection_view(),
                            ),
                        },
                    ],
                    label: Some("Equirect projection bind group"),
                });

        let equirect_dst_bind_group =
            resources
                .gpu
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &bind_group_layouts.equirect_dst,
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
                    label: Some("Skybox bind group"),
                });

        resources.gpu.queue.write_texture(
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

        managers
            .texture_manager
            .insert_texture_bind_group(&src_key, equirect_src_bind_group.clone());
        managers
            .texture_manager
            .insert_texture_bind_group(&dst_key, equirect_dst_bind_group.clone());

        managers.texture_manager.insert(src_key.clone(), src.into());
        managers.texture_manager.insert(dst_key.clone(), dst.into());

        Ok((
            src_key,
            equirect_src_bind_group,
            dst_key,
            equirect_dst_bind_group,
        ))
    }
    pub fn compute(
        &self,
        queue: &wgpu::Queue,
        mut encoder: wgpu::CommandEncoder,
        managers: &Managers,
        bind_group: &wgpu::BindGroup,
        dst_size: u32,
        label: Option<&str>,
    ) {
        if let Some(projection_compute_pipeline) = managers
            .pipeline_manager
            .get_compute_pipeline(self.src_pipeline_key.clone())
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label,
                timestamp_writes: None,
            });

            let num_workgroups = (dst_size + 15) / 16;
            pass.set_pipeline(&projection_compute_pipeline);
            pass.set_bind_group(0, bind_group, &[]);
            pass.dispatch_workgroups(num_workgroups, num_workgroups, 6);

            drop(pass);
            queue.submit([encoder.finish()]);
        }
    }
    pub fn render(
        &self,
        rpass: &mut wgpu::RenderPass,
        managers: &mut Managers,
        bind_group_layouts: &BindGroupLayouts,
        camera_bind_group: &wgpu::BindGroup,
    ) {
        if let (Some(equirect_projection_bind_group), Some(equirect_projection_pipeline)) = (
            managers
                .texture_manager
                .bind_group_for(&self.dst_texture_key.id, &bind_group_layouts.equirect_dst),
            managers
                .pipeline_manager
                .render_pipelines
                .get(&self.dst_pipeline_key),
        ) {
            rpass.set_bind_group(0, camera_bind_group, &[]);
            rpass.set_bind_group(1, equirect_projection_bind_group, &[]);
            rpass.set_pipeline(&equirect_projection_pipeline);
            rpass.draw(0..3, 0..1);
        }
    }
}

pub struct Environment {
    equirect_projection: Option<EquirectProjection>,
}

impl Environment {
    pub fn new(
        resources: &Resources,
        managers: &mut Managers,
        bind_group_layouts: &BindGroupLayouts,
        equirect_projection: Option<EquirectProjection>,
    ) -> Result<Self, EngineError> {
        if let Some(projection) = equirect_projection.as_ref() {
            let (src_key, src_bind_group, dst_key, dst_bind_group) = EquirectProjection::prepare(
                managers,
                resources,
                bind_group_layouts,
                &projection.hdr_rel_path,
                projection.dst_size,
                projection.format,
            )?;
            let label = "equirect_projection encoder";
            let encoder = resources
                .gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) });
            projection.compute(
                &resources.gpu.queue,
                encoder,
                managers,
                &src_bind_group,
                projection.dst_size,
                Some(label),
            );
        }
        Ok(Self {
            equirect_projection,
        })
    }

    pub fn render(
        &self,
        rpass: &mut wgpu::RenderPass,
        managers: &mut Managers,
        bind_group_layouts: &BindGroupLayouts,
        camera_bind_group: &wgpu::BindGroup,
    ) {
        if let Some(projection) = &self.equirect_projection {
            projection.render(rpass, managers, bind_group_layouts, camera_bind_group);
        }
    }
}
