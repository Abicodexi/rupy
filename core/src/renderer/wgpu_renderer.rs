use std::sync::Arc;

use super::{Mesh, VertexTexture};
use crate::{
    assets::loader::AssetLoader, pipeline::PipelineManager, texture::TextureManager,
    BindGroupLayouts, CacheKey, EngineError, GpuContext, Renderer, ShaderManager,
    WgpuBufferManager,
};
use wgpu::{CommandEncoder, DepthStencilState, SurfaceConfiguration};

#[warn(dead_code)]
pub struct WgpuRenderer {
    pub default_pipeline: Arc<wgpu::RenderPipeline>,
    pub equirect_dst_pipeline: wgpu::RenderPipeline,
    pub equirect_src_pipeline: wgpu::ComputePipeline,
}

impl WgpuRenderer {
    pub fn new(
        gpu: &GpuContext,
        asset_loader: &AssetLoader,
        shader_manager: &mut ShaderManager,
        pipeline_manager: &mut PipelineManager,
        config: &SurfaceConfiguration,
        depth_stencil: &DepthStencilState,
        bind_group_layouts: &BindGroupLayouts,
    ) -> Result<Self, EngineError> {
        let default_shader = shader_manager.get_or_create("v_texture.wgsl", || {
            let shader_module = asset_loader.load_shader("v_texture.wgsl")?;
            Ok(Arc::new(shader_module))
        });

        let default_pipeline_layout =
            gpu.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("default pipeline layout"),
                    bind_group_layouts: &[&bind_group_layouts.camera, &bind_group_layouts.texture],
                    push_constant_ranges: &[],
                });
        let default_pipeline_cache_key = CacheKey::from("default_pipeline");
        let default_pipeline: Arc<wgpu::RenderPipeline> = gpu
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("default pipeline"),
                layout: Some(&default_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &default_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[VertexTexture::LAYOUT],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &default_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::default(),
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: Default::default(),
                depth_stencil: Some(depth_stencil.clone()),
                multisample: Default::default(),
                multiview: None,
                cache: None,
            })
            .into();
        pipeline_manager
            .render_pipelines
            .insert(default_pipeline_cache_key, default_pipeline.clone());
        let equirect_src_shader = shader_manager.get_or_create("equirect_src.wgsl", || {
            let shader_module = asset_loader.load_shader("equirect_src.wgsl")?;
            Ok(Arc::new(shader_module))
        });

        let equirect_src_pipeline_layout =
            gpu.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Equirect src pipeline layout"),
                    bind_group_layouts: &[&bind_group_layouts.equirect_src],
                    push_constant_ranges: &[],
                });

        let equirect_src_pipeline =
            gpu.device()
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("Equirect src pipeline"),
                    layout: Some(&equirect_src_pipeline_layout),
                    module: &equirect_src_shader,
                    entry_point: Some("compute_equirect_to_cubemap"),
                    compilation_options: Default::default(),
                    cache: None,
                });

        let equirect_dst_shader = shader_manager.get_or_create("equirect_dst.wgsl", || {
            let shader_module = asset_loader.load_shader("equirect_dst.wgsl")?;
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

        let equirect_dst_pipeline =
            gpu.device()
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
                    depth_stencil: Some(depth_stencil.clone()),

                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                    cache: None,
                });

        Ok(WgpuRenderer {
            default_pipeline,
            equirect_dst_pipeline,
            equirect_src_pipeline,
        })
    }
    pub fn equirect_projection(
        &self,
        queue: &wgpu::Queue,
        mut encoder: CommandEncoder,
        bind_group: &wgpu::BindGroup,
        dst_size: u32,
        label: Option<&str>,
    ) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label,
            timestamp_writes: None,
        });

        let num_workgroups = (dst_size + 15) / 16;
        pass.set_pipeline(&self.equirect_src_pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.dispatch_workgroups(num_workgroups, num_workgroups, 6);

        drop(pass);
        queue.submit([encoder.finish()]);
    }
}

impl Renderer for WgpuRenderer {
    fn update(&mut self, _dt: f32) {}

    fn render(
        &self,
        rpass: &mut wgpu::RenderPass,
        bind_group_layouts: &BindGroupLayouts,
        texture_manager: &mut TextureManager,
        w_buffer_manager: &mut WgpuBufferManager,
        camera_bind_group: &wgpu::BindGroup,
        mesh: &Mesh,
    ) {
        if let Some(skybox_bind_group) = texture_manager
            .bind_group_for("equirect_projection_dst", &bind_group_layouts.equirect_dst)
        {
            rpass.set_bind_group(0, camera_bind_group, &[]);
            rpass.set_bind_group(1, skybox_bind_group, &[]);
            rpass.set_pipeline(&self.equirect_dst_pipeline);
            rpass.draw(0..3, 0..1);
        }

        if let Some(texture_bind_group) =
            texture_manager.bind_group_for("cube_diffuse", &bind_group_layouts.texture)
        {
            mesh.draw(
                rpass,
                &self.default_pipeline,
                vec![&camera_bind_group, texture_bind_group],
                &w_buffer_manager,
            );
        }
        drop(rpass);
    }
}
