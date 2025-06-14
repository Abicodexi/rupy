use crate::{AssetService, Entity, Texture};
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct CameraModel {
    model: String,
    v_shader: String,
    f_shader: String,
    entity: Option<Entity>,
    distance: f32,
    height: f32,
    target_height: f32,
    shoulder_offset: f32,
}

impl CameraModel {
    pub fn new(model: &str, v_shader: &str, f_shader: &str) -> Self {
        Self {
            model: model.to_string(),
            v_shader: v_shader.to_string(),
            f_shader: f_shader.to_string(),
            entity: None,
            distance: 1.0,
            height: 2.0,
            target_height: 2.0,
            shoulder_offset: 0.0,
        }
    }
    pub fn height(&self) -> f32 {
        self.height
    }
    pub fn distance(&self) -> f32 {
        self.distance
    }
    pub fn target_height(&self) -> f32 {
        self.target_height
    }
    pub fn shoulder_offset(&self) -> f32 {
        self.shoulder_offset
    }
    pub fn model(&self) -> &str {
        &self.model
    }
    pub fn entity(&self) -> Option<Entity> {
        self.entity
    }
    pub fn set_entity(&mut self, entity: Entity) {
        self.entity = Some(entity)
    }

    pub fn configure(&mut self, model: &str, v_shader: &str, f_shader: &str) {
        self.model = model.to_owned();
        self.v_shader = v_shader.to_owned();
        self.f_shader = f_shader.to_owned();
    }

    pub fn shaders(&self) -> (&str, &str) {
        (&self.v_shader, &self.f_shader)
    }

    pub fn load_model(
        &mut self,
        service: &AssetService,
        bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
        format: wgpu::TextureFormat,
    ) {
        let file = &self.model;
        let (v_shader, f_shader) = self.shaders();

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
            format,
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
        service.load_model(
            file.to_string(),
            v_shader.to_string(),
            f_shader.to_string(),
            bind_group_layouts,
            primitive,
            color_target,
            Some(depth_stencil),
        );
    }
}
