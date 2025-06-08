use crate::{
    BindGroupManager, CacheKey, CacheStorage, Entity, MaterialManager, ModelManager,
    PipelineManager, ShaderManager, Texture, TextureManager, World,
};

#[derive(Debug, Default)]
pub struct CameraModel {
    model: String,
    v_shader: String,
    f_shader: String,
    model_key: Option<CacheKey>,
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
            model_key: None,
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
    pub fn model_key(&self) -> Option<CacheKey> {
        self.model_key
    }

    pub fn update(&mut self, model: &str, v_shader: &str, f_shader: &str) {
        self.model = model.to_owned();
        self.v_shader = v_shader.to_owned();
        self.f_shader = f_shader.to_owned();
    }

    pub fn shaders(&self) -> (&str, &str) {
        (&self.v_shader, &self.f_shader)
    }

    pub fn load_model(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        model_manager: &mut ModelManager,
        material_manager: &mut MaterialManager,
        texture_manager: &mut TextureManager,
        shader_manager: &mut ShaderManager,
        pipeline_manager: &mut PipelineManager,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        bind_group_layouts: &Vec<&wgpu::BindGroupLayout>,
        surface_configuration: &wgpu::SurfaceConfiguration,
    ) {
        let prev_model = if let Some(key) = self.model_key {
            self.model_key = None;
            model_manager.remove(&key)
        } else {
            None
        };

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

        self.model_key = World::load_object(
            queue,
            device,
            model_manager,
            material_manager,
            texture_manager,
            shader_manager,
            pipeline_manager,
            file,
            v_shader,
            f_shader,
            buffers,
            bind_group_layouts,
            surface_configuration,
            primitive,
            color_target,
            Some(depth_stencil),
        );

        if self.model_key.is_none() {
            if let Some(model) = prev_model {
                let key = CacheKey::from(model.name.clone());
                model_manager.insert(key, model);
                self.model_key = Some(key);
            }
        }
    }
}
