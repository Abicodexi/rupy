use super::{mesh::Mesh, vertex::VertexTexture};
use crate::{
    camera::uniform::CameraUniform, texture::TextureManager, BindGroupLayouts, CacheKey,
    EngineError, GpuContext, Renderer, WgpuBuffer, WgpuBufferCache,
};
use wgpu::{
    CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor,
    SurfaceConfiguration, SurfaceTexture,
};

pub struct WgpuRenderer {
    pipeline: wgpu::RenderPipeline,
}

impl WgpuRenderer {
    pub fn new(
        gpu: &GpuContext,
        config: &SurfaceConfiguration,
        bind_group_layouts: &BindGroupLayouts,
    ) -> Result<Self, EngineError> {
        let shader = gpu
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("default shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("C:\\Users\\abism\\Desktop\\rupy\\v_texture.wgsl").into(),
                ),
            });

        let pipeline_layout =
            gpu.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("pipeline layout"),
                    bind_group_layouts: &[&bind_group_layouts.camera, &bind_group_layouts.texture],
                    push_constant_ranges: &[],
                });

        let pipeline = gpu
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("default pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[VertexTexture::LAYOUT],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::default(),
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview: None,
                cache: None,
            });

        Ok(WgpuRenderer { pipeline })
    }
}

impl Renderer for WgpuRenderer {
    fn resize(&mut self, _config: &SurfaceConfiguration) {}

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
            rpass.set_pipeline(&self.pipeline);
            // rpass.set_vertex_buffer(0, self.vertex_buffer.buffer.slice(..));
            // rpass.draw(0..self.vertex_count, 0..1);
        }

        gpu.queue().submit(Some(encoder.finish()));
        surface_texture.present();
    }
    fn render_mesh(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        camera_bg: &wgpu::BindGroup,
        texture_bg: &wgpu::BindGroup,
        wgpu_buffer_cache: &mut WgpuBufferCache,
        mesh: &Mesh,
    ) {
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

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, camera_bg, &[]);
        rpass.set_bind_group(1, texture_bg, &[]);

        match mesh {
            Mesh::Shared { key, count } => {
                let vb = wgpu_buffer_cache.get_buffer(key).unwrap();
                rpass.set_vertex_buffer(0, vb.buffer.slice(..));
                rpass.draw(0..*count, 0..1);
            }
            Mesh::Unique { buffer, count } => {
                rpass.set_vertex_buffer(0, buffer.buffer.slice(..));
                rpass.draw(0..*count, 0..1);
            }
        }
    }
}
