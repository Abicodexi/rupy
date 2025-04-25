use std::sync::Arc;

use crate::{
    camera::uniform::CameraUniform,
    texture::{Texture, TextureManager},
    BindGroupLayouts, CacheKey, EngineError, GpuContext, Mesh, Renderer, VertexTexture, WgpuBuffer,
    WgpuBufferCache,
};
use wgpu::{
    CommandEncoder, CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, SurfaceConfiguration, SurfaceTexture,
};
#[warn(dead_code)]
pub struct WgpuRenderer {
    pub default_pipeline: wgpu::RenderPipeline,
    pub equirect_dst_pipeline: wgpu::RenderPipeline,
    pub equirect_src_pipeline: wgpu::ComputePipeline,
    pub depth_texture: Texture,
    pub device: Arc<wgpu::Device>,
}

impl WgpuRenderer {
    pub fn new(
        gpu: &GpuContext,
        config: &SurfaceConfiguration,
        bind_group_layouts: &BindGroupLayouts,
    ) -> Result<Self, EngineError> {
        let depth_stencil = wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };

        let default_shader = gpu
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("default shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("C:\\Users\\abism\\Desktop\\rupy\\v_texture.wgsl").into(),
                ),
            });

        let default_pipeline_layout =
            gpu.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("pipeline layout"),
                    bind_group_layouts: &[&bind_group_layouts.camera, &bind_group_layouts.texture],
                    push_constant_ranges: &[],
                });

        let default_pipeline =
            gpu.device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("default pipeline"),
                    layout: Some(&default_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &default_shader,
                        entry_point: "vs_main",
                        buffers: &[VertexTexture::LAYOUT],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &default_shader,
                        entry_point: "fs_main",
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
                });

        let equirect_src_shader = gpu
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("default shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("C:\\Users\\abism\\Desktop\\rupy\\equirect_src.wgsl").into(),
                ),
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
                    entry_point: "compute_equirect_to_cubemap",
                    compilation_options: Default::default(),
                    cache: None,
                });

        let equirect_dst_shader = gpu
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Equirect dst shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("C:\\Users\\abism\\Desktop\\rupy\\equirect_dst.wgsl").into(),
                ),
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
                        entry_point: "vs_main",
                        buffers: &[],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &equirect_dst_shader,
                        entry_point: "fs_main",
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
                    depth_stencil: Some(depth_stencil),

                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                    cache: None,
                });

        let depth_texture = Texture::create(
            gpu.device(),
            wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            Texture::DEPTH_FORMAT,
            1,
            wgpu::TextureViewDimension::D2,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            Some(wgpu::AddressMode::ClampToEdge),
            wgpu::FilterMode::Linear,
            Some(gpu.device().create_sampler(&wgpu::SamplerDescriptor {
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
            default_pipeline,
            equirect_dst_pipeline,
            equirect_src_pipeline,
            depth_texture,
            device: Arc::clone(&gpu.device),
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
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(num_workgroups, num_workgroups, 6);

        drop(pass);
        queue.submit([encoder.finish()]);
    }
}

impl Renderer for WgpuRenderer {
    fn resize(&mut self, config: &SurfaceConfiguration) {
        self.depth_texture = Texture::create(
            &self.device,
            wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            Texture::DEPTH_FORMAT,
            1,
            wgpu::TextureViewDimension::D2,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            Some(wgpu::AddressMode::ClampToEdge),
            wgpu::FilterMode::Linear,
            Some(self.device.create_sampler(&wgpu::SamplerDescriptor {
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
        )
    }

    fn update(&mut self, _dt: f32) {}

    fn render(
        &self,
        gpu: &GpuContext,
        surface_texture: SurfaceTexture,
        bind_group_layouts: &BindGroupLayouts,
        texture_manager: &TextureManager,
        wgpu_buffer_cache: &mut WgpuBufferCache,
        camera_uniform: &CameraUniform,
    ) {
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = gpu
            .device()
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("main pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            let camera_bind_group = gpu.device().create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("camera_bg"),
                layout: &bind_group_layouts.camera,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu_buffer_cache
                        .get_or_create_buffer(&CacheKey::new("camera_uniform_buffer"), || {
                            WgpuBuffer::from_data(
                                gpu.device(),
                                &[*camera_uniform],
                                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                            )
                        })
                        .buffer
                        .as_entire_binding(),
                }],
            });
            rpass.set_bind_group(0, &camera_bind_group, &[]);

            if let Some(tex) = texture_manager.get("cube_diffuse") {
                let bind_group = gpu.device().create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &bind_group_layouts.texture,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&tex.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&tex.sampler),
                        },
                    ],
                    label: Some("texture_bind_group"),
                });

                rpass.set_bind_group(1, &bind_group, &[]);
            }
            rpass.set_pipeline(&self.default_pipeline);
            // rpass.set_vertex_buffer(0, self.vertex_buffer.buffer.slice(..));
            // rpass.draw(0..self.vertex_count, 0..1);
        }

        gpu.queue().submit(Some(encoder.finish()));
        surface_texture.present();
    }
    fn render_mesh(
        &self,
        rpass: &mut wgpu::RenderPass,
        camera_bind_group: &wgpu::BindGroup,
        texture_bind_group: &wgpu::BindGroup,
        wgpu_buffer_cache: &mut WgpuBufferCache,
        mesh: &Mesh,
    ) {
        rpass.set_pipeline(&self.default_pipeline);
        rpass.set_bind_group(0, camera_bind_group, &[]);
        rpass.set_bind_group(1, texture_bind_group, &[]);

        match mesh {
            Mesh::Shared { key, count } => {
                if let Some(vb) = wgpu_buffer_cache.get_buffer(key) {
                    rpass.set_vertex_buffer(0, vb.buffer.slice(..));
                    rpass.draw(0..*count, 0..1);
                }
            }
            Mesh::Unique { buffer, count } => {
                rpass.set_vertex_buffer(0, buffer.buffer.slice(..));
                rpass.draw(0..*count, 0..1);
            }
        }
    }
}
