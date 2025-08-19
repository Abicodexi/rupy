use std::sync::Arc;

use crate::{
    camera::Camera,
    gfx::{
        bind_group::{
            debug_group, debug_layout, material_storage_layout, normal_texture_layout,
            skybox_cubemap_layout,
        },
        buffer::WgpuBuffer,
    },
    AssetService, CacheKey, EngineError, Light, Texture, Vertex, VertexInstance,
};
use bytemuck::{Pod, Zeroable};
use wgpu::{BufferUsages, RenderPipeline};

#[repr(C)]
#[derive(Debug, Copy, Clone, Zeroable, Pod)]
pub struct DebugUniform {
    pub mode: u32,
    _pad0: [f32; 3],
    pub zfar: f32,
    _pad1: [f32; 3],
    pub znear: f32,
    _pad2: [f32; 3],
    normal_line_length: f32,
    normal_color: [f32; 3],
}

impl DebugUniform {
    pub fn next(&mut self) {
        self.mode = match self.mode {
            0 => 1,
            1 => 2,
            2 => 3,
            3 => 4,
            4 => 5,
            5 => 6,
            _ => 0,
        };
    }
}

pub struct DebugMode {
    buffer: WgpuBuffer,
    uniform: DebugUniform,
    bind_group: wgpu::BindGroup,
    pipeline: RenderPipeline,
    normal_line_pipeline: RenderPipeline,
    mode: u32,
}

impl DebugMode {
    pub fn new(
        service: &'static Arc<AssetService>,
        camera: &Camera,
        light: &Light,
        surface_configuration: &wgpu::SurfaceConfiguration,
    ) -> Result<Self, EngineError> {
        let zfar = camera.zfar();
        let znear = camera.znear();
        let uniform = DebugUniform {
            mode: 0,
            _pad0: [0.0; 3],
            zfar,
            _pad1: [0.0; 3],
            znear,
            _pad2: [0.0; 3],
            normal_line_length: 1.0,
            normal_color: [1.0; 3],
        };
        let buffer = WgpuBuffer::from_data(
            service.device(),
            bytemuck::bytes_of(&uniform),
            BufferUsages::UNIFORM,
            Some("debug uniform buffer"),
        );
        let bind_group = debug_group(
            service.device(),
            &debug_layout(service.device()),
            camera.buffer(),
            light.buffer(),
            &buffer,
        );
        service.load_shader("debug.wgsl");
        let shader = service.get_shader(&CacheKey::from("debug.wgsl")).unwrap();
        let buffers = &[Vertex::LAYOUT, VertexInstance::LAYOUT];

        let pipeline_layout =
            service
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("debug_pipeline_layout"),
                    bind_group_layouts: &[
                        &debug_layout(service.device()),
                        &skybox_cubemap_layout(service.device()),
                        &material_storage_layout(service.device()),
                        &normal_texture_layout(service.device()),
                    ],
                    push_constant_ranges: &[],
                });

        let primitive = wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        };

        let color_target = wgpu::ColorTargetState {
            format: surface_configuration.format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        };

        let depth_stencil = wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };

        let pipeline = service
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("debug_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers,
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(color_target.clone())],
                    compilation_options: Default::default(),
                }),
                primitive: primitive,
                depth_stencil: Some(depth_stencil.clone()),

                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });
        // normal_lines pipeline
        service.load_shader("normal_lines.wgsl");
        let line_shader = service
            .get_shader(&CacheKey::from("normal_lines.wgsl"))
            .unwrap();
        let line_pipeline_layout =
            service
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("normal_line_pipeline_layout"),
                    bind_group_layouts: &[
                        &debug_layout(service.device()),
                        &skybox_cubemap_layout(service.device()),
                        &material_storage_layout(service.device()),
                        &normal_texture_layout(service.device()),
                    ],
                    push_constant_ranges: &[],
                });

        let normal_line_pipeline =
            service
                .device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("normal_line_pipeline"),
                    layout: Some(&line_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &line_shader,
                        entry_point: Some("vs_main"),
                        buffers: &[Vertex::LAYOUT, VertexInstance::LAYOUT],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &line_shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(color_target)],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::LineList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Fill,
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

        Ok(Self {
            buffer,
            uniform,
            bind_group,
            pipeline,
            normal_line_pipeline,
            mode: 0,
        })
    }

    pub fn normal_line_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.normal_line_pipeline
    }

    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    pub fn uniform(&self) -> &DebugUniform {
        &self.uniform
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn buffer(&self) -> &WgpuBuffer {
        &self.buffer
    }

    pub fn mode(&self) -> u32 {
        self.mode
    }

    pub fn next_mode(&mut self, device: &wgpu::Device, camera: &Camera, light: &Light) {
        let current_mode = self.mode;
        let mut next_mode = current_mode + 1;

        if next_mode > 7 {
            next_mode = 0;
        }
        self.rebuild(device, next_mode, camera, light);
    }

    fn rebuild(&mut self, device: &wgpu::Device, mode: u32, camera: &Camera, light: &Light) {
        let zfar = camera.zfar();
        let znear = camera.znear();
        let uniform = DebugUniform {
            mode,
            _pad0: [0.0; 3],
            zfar,
            _pad1: [0.0; 3],
            znear,
            _pad2: [0.0; 3],
            normal_line_length: 1.0,
            normal_color: [1.0; 3],
        };
        let buffer = WgpuBuffer::from_data(
            device,
            bytemuck::bytes_of(&self.uniform),
            BufferUsages::UNIFORM,
            Some("debug uniform buffer"),
        );
        self.bind_group = debug_group(
            device,
            &debug_layout(device),
            camera.buffer(),
            light.buffer(),
            &buffer,
        );

        self.uniform = uniform;
        self.buffer = buffer;
        self.mode = mode;
    }
}
