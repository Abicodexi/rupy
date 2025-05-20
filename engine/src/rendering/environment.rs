use crate::RenderBindGroupLayouts;

#[derive(Debug)]
pub struct WorldProjection {
    pub src_shader: wgpu::ShaderModule,
    pub dst_shader: wgpu::ShaderModule,
    pub src_texture: crate::Texture,
    pub dst_texture: crate::Texture,
    pub src_pipeline: wgpu::ComputePipeline,
    pub dst_pipeline: wgpu::RenderPipeline,
    pub src_bind_group: wgpu::BindGroup,
    pub dst_bind_group: wgpu::BindGroup,
}

impl WorldProjection {
    pub const DEST_SIZE: u32 = 1080;
    pub const NUM_WORKGROUPS: u32 = (Self::DEST_SIZE + 15) / 16;
    pub const DEPTH_OR_ARRAY_LAYERS: u32 = 6;
    pub fn new(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        src_shader: &str,
        dst_shader: &str,
        hdr_texture: &str,
        depth_stencil_state: Option<wgpu::DepthStencilState>,
    ) -> Result<Self, crate::EngineError> {
        let path = &crate::Asset::resolve(&format!("hdr/{}", hdr_texture));
        let bytes = crate::Asset::read_bytes(&path)?;
        let (pixels, meta) = crate::Texture::decode_hdr(&bytes)?;

        let src_texture = crate::Texture::new(
            device,
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

        let dst_texture = crate::Texture::new(
            device,
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

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &src_texture.texture,
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
            src_texture.texture.size(),
        );

        let dst_bind_group = crate::BindGroup::equirect_dst(device, &dst_texture);
        let src_bind_group = crate::BindGroup::equirect_src(device, &src_texture, &dst_texture);

        let equirect_src_shader = crate::Shader::load(src_shader)?;

        let equirect_src_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("{} layout", src_shader)),
                bind_group_layouts: &[&RenderBindGroupLayouts::equirect_src()],
                push_constant_ranges: &[],
            });

        let src_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(src_shader),
            layout: Some(&equirect_src_pipeline_layout),
            module: &equirect_src_shader,
            entry_point: Some("compute_equirect_to_cubemap"),
            compilation_options: Default::default(),
            cache: None,
        });
        let equirect_dst_shader = crate::Shader::load(dst_shader)?;

        let equirect_dst_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{} layout", dst_shader)),
            bind_group_layouts: &[
                RenderBindGroupLayouts::uniform(),
                RenderBindGroupLayouts::equirect_dst(),
            ],
            push_constant_ranges: &[],
        });

        let dst_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
        });

        Ok(WorldProjection {
            src_shader: equirect_src_shader,
            dst_shader: equirect_dst_shader,
            src_texture,
            dst_texture,
            dst_pipeline,
            src_pipeline,
            src_bind_group,
            dst_bind_group,
        })
    }

    pub fn compute_projection(
        &self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        label: Option<&str>,
    ) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("compute encoder"),
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label,
            timestamp_writes: None,
        });

        pass.set_pipeline(&self.src_pipeline);
        pass.set_bind_group(0, &self.src_bind_group, &[]);
        pass.dispatch_workgroups(Self::NUM_WORKGROUPS, Self::NUM_WORKGROUPS, 6);

        drop(pass);
        queue.submit([encoder.finish()]);
    }
    pub fn render(&self, rpass: &mut wgpu::RenderPass, uniform_bind_group: &wgpu::BindGroup) {
        rpass.set_bind_group(0, uniform_bind_group, &[]);
        rpass.set_bind_group(1, &self.dst_bind_group, &[]);
        rpass.set_pipeline(&self.dst_pipeline);
        rpass.draw(0..3, 0..1);
    }
}
#[derive(Debug)]
pub struct Environment {
    wp: WorldProjection,
}

impl Environment {
    pub fn new(wp: WorldProjection) -> Self {
        Self { wp }
    }
    pub fn render(&self, rpass: &mut wgpu::RenderPass, uniform_bind_group: &wgpu::BindGroup) {
        self.wp.render(rpass, uniform_bind_group);
    }
    pub fn projection(&self) -> &WorldProjection {
        &self.wp
    }
}
