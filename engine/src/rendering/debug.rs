use super::{Light, ShaderManager, Vertex, VertexInstance};
use crate::{camera::Camera, BindGroup, EngineError, RenderBindGroupLayouts, Texture, WgpuBuffer};
use bytemuck::{Pod, Zeroable};
use wgpu::{BufferUsages, RenderPipeline};

#[repr(C)]
#[derive(Copy, Clone, Zeroable, Pod)]
pub struct DebugUniform {
    pub mode: u32,
    _pad0: [f32; 3],
    pub zfar: f32,
    _pad1: [f32; 3],
    pub znear: f32,
    _pad2: [f32; 3],
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
    mode: u32,
}

impl DebugMode {
    pub fn new(
        device: &wgpu::Device,
        shaders: &mut ShaderManager,
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
        };
        let buffer = WgpuBuffer::from_data(
            device,
            bytemuck::bytes_of(&uniform),
            BufferUsages::UNIFORM,
            Some("debug uniform buffer"),
        );
        let bind_group = BindGroup::debug(device, camera.buffer(), light.buffer(), &buffer);
        let shader = shaders.load(device, "debug.wgsl")?;
        let buffers = &[Vertex::LAYOUT, VertexInstance::LAYOUT];
        let bind_group_layouts = [
            RenderBindGroupLayouts::debug(),
            RenderBindGroupLayouts::equirect_dst(),
            RenderBindGroupLayouts::material_storage(),
            RenderBindGroupLayouts::normal(),
        ];
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("debug_pipeline_layout"),
            bind_group_layouts: &bind_group_layouts,
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
            write_mask: wgpu::ColorWrites::all(),
        };

        let depth_stencil = wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                targets: &[Some(color_target)],
                compilation_options: Default::default(),
            }),
            primitive: primitive,
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
            mode: 0,
        })
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
        let mut mode = if self.mode == 0 {
            1
        } else {
            let current_mode = self.mode;
            let next_mode = current_mode + 1;
            next_mode
        };

        if mode > 7 {
            mode = 0;
        }
        self.rebuild(device, mode, camera, light);
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
        };
        let buffer = WgpuBuffer::from_data(
            device,
            bytemuck::bytes_of(&self.uniform),
            BufferUsages::UNIFORM,
            Some("debug uniform buffer"),
        );
        self.bind_group = BindGroup::debug(device, camera.buffer(), light.buffer(), &buffer);

        self.uniform = uniform;
        self.buffer = buffer;
        self.mode = mode;
    }
}
