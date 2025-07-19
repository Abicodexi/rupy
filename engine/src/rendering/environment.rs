use crate::{
  AssetService, BindGroup, CacheKey, EngineError,Texture,AssetLoader,Terrain,Light,Medium,WgpuBuffer
   
};
use std::sync::Arc;

#[derive(Debug)]
pub struct SkyProjection {
    pub src_texture: Arc<Texture>,
    pub dst_texture: Arc<Texture>,
    pub src_pipeline: Arc<wgpu::ComputePipeline>,
    pub dst_pipeline: Arc<wgpu::RenderPipeline>,
    pub src_bind_group: Arc<wgpu::BindGroup>,
    pub dst_bind_group: Arc<wgpu::BindGroup>,
}
use crate::TextureDescriptor;

impl SkyProjection {
    pub const DEST_SIZE: u32 = 1080;
    pub const NUM_WORKGROUPS: u32 = (Self::DEST_SIZE + 15) / 16;

    pub fn new(
        service: &AssetService,
        config: &wgpu::SurfaceConfiguration,
        src_shader: &str,
        dst_shader: &str,
        hdr_texture: &str,
    ) -> Result<Self, EngineError> {
        let dst_key = CacheKey::from("projection_dst");
        let depth_stencil = wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };

        let dst_texture = service.get_or_create_texture(dst_key, || {
            Ok(Texture::new(
                service.device(),
                TextureDescriptor {
                    size: wgpu::Extent3d {
                        width: Self::DEST_SIZE,
                        height: Self::DEST_SIZE,
                        depth_or_array_layers: 6,
                    },
                    format: Texture::HDR_FORMAT,
                    mip_level_count: 1,
                    view_dim: wgpu::TextureViewDimension::Cube,
                    usage: wgpu::TextureUsages::STORAGE_BINDING
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    address_mode: Some(wgpu::AddressMode::ClampToEdge),
                    mag_filter: wgpu::FilterMode::Nearest,
                    sampler: None,
                },
                Some("projection destination texture"),
            )
            .into())
        })?;

        let src_key = CacheKey::from(hdr_texture);
        let src_texture = if let Some(src_tex) = service.get_texture(&src_key) {
            src_tex
        } else {
            let path = AssetLoader::resolve("hdr").join(hdr_texture);
            let bytes = AssetLoader::bytes(path)?;
            let (pixels, meta) = Texture::decode_hdr(&bytes)?;
            let src_tex = Texture::new(
                service.device(),
                TextureDescriptor {
                    size: wgpu::Extent3d {
                        width: meta.width,
                        height: meta.height,
                        depth_or_array_layers: 1,
                    },
                    format: Texture::HDR_FORMAT,
                    mip_level_count: 1,
                    view_dim: wgpu::TextureViewDimension::D2,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST,
                    address_mode: None,
                    mag_filter: wgpu::FilterMode::Nearest,
                    sampler: None,
                },
                Some("projection source texture"),
            );
            service.queue().write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &src_tex.texture,
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
                src_tex.texture.size(),
            );
            Arc::new(src_tex)
        };

        let dst_bind_group = service.get_or_create_bind_group("projection_dst".into(), || {
            Ok(BindGroup::equirect_dst(
                service.device(),
                service.bind_group_layouts(),
                &dst_texture,
            )
            .into())
        })?;

        let src_bind_group = service.get_or_create_bind_group("projection_src".into(), || {
            Ok(BindGroup::equirect_src(
                service.device(),
                service.bind_group_layouts(),
                &src_texture,
                &dst_texture,
            )
            .into())
        })?;

        let equirect_src_pipeline_layout =
            service
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Equirect source pipeline layout"),
                    bind_group_layouts: &[service.bind_group_layouts().equirect_src()],
                    push_constant_ranges: &[],
                });

        let src_pipeline = service
            .get_or_load_compute_pipeline(
                src_shader,
                equirect_src_pipeline_layout,
                Some("compute_equirect_to_cubemap"),
                "equirect_src".into(),
                "Projection source pipeline",
            )
            .unwrap();

        let equirect_dst_layout =
            service
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(&format!("{} layout", dst_shader)),
                    bind_group_layouts: &[
                        service.bind_group_layouts().uniform().as_ref(),
                        service.bind_group_layouts().equirect_dst().as_ref(),
                    ],
                    push_constant_ranges: &[],
                });

        let dst_pipeline = service
            .get_or_load_render_pipeline(
                dst_shader,
                dst_shader,
                equirect_dst_layout,
                &[],
                config.format,
                Some(depth_stencil),
                "projection_dst".into(),
                "Equirect projection destination pipeline".to_string(),
            )
            .unwrap();

        Ok(SkyProjection {
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
        pass.set_bind_group(0, self.src_bind_group.as_ref(), &[]);
        pass.dispatch_workgroups(Self::NUM_WORKGROUPS, Self::NUM_WORKGROUPS, 6);

        drop(pass);
        queue.submit([encoder.finish()]);
    }

    pub fn render(&self, rpass: &mut wgpu::RenderPass, uniform_bind_group: &wgpu::BindGroup) {
        rpass.set_bind_group(0, uniform_bind_group, &[]);
        rpass.set_bind_group(1, self.dst_bind_group.as_ref(), &[]);
        rpass.set_pipeline(&self.dst_pipeline);
        rpass.draw(0..3, 0..1);
    }
}
#[derive(Debug)]
pub struct Environment {
    sky: SkyProjection,
    terrain: Terrain,
    light: Light,
}

impl Environment {
    pub fn new(
        service: &AssetService,
        config: &wgpu::SurfaceConfiguration,
        src_shader: &str,
        dst_shader: &str,
        hdr_texture: &str,
    ) -> Result<Self, EngineError> {
        Ok(Self {
            sky: SkyProjection::new(service, config, src_shader, dst_shader, hdr_texture)?,
            light: Light::new(service.device(), service.bind_group_layouts())?,
            terrain: Terrain::new(Medium::Ground),
        })
    }

    pub fn compute_sky_projection(&self, queue: &wgpu::Queue, device: &wgpu::Device) {
        self.sky.compute_projection(queue, device, Some("compute sky projection"));
    }

    pub fn render_skybox(&self, rpass: &mut wgpu::RenderPass, camera_bind_group: &wgpu::BindGroup) {
        self.sky.render(rpass, camera_bind_group);
    }

    pub fn update_light(&mut self, time: f64) {
        self.light.orbit(time);
    }

    pub fn upload_light(&mut self, queue: &wgpu::Queue, device: &wgpu::Device) {
        self.light.upload(queue, device);
    }

    pub fn sky_texture(&self) -> &Arc<Texture> {
        &self.sky.dst_texture
    }

    pub fn light(&self) -> &Light {
        &self.light
    }

    pub fn light_mut(&mut self) -> &mut Light {
        &mut self.light
    }

    pub fn light_bind_group(&self) -> &wgpu::BindGroup {
        self.light.bind_group()
    }

    pub fn light_buffer(&self) -> &WgpuBuffer {
        self.light.buffer()
    }

    pub fn terrain(&self) -> &Terrain {
        &self.terrain
    }

    pub fn terrain_mut(&mut self) -> &mut Terrain {
        &mut self.terrain
    }

    pub fn sky_projection(&self) -> &SkyProjection {
        &self.sky
    }

    pub fn sky_projection_mut(&mut self) -> &mut SkyProjection {
        &mut self.sky
    }
}
