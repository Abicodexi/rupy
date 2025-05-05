#[derive(Clone)]
pub struct Material {
    pub name: String,
    pub bind_groups: Vec<wgpu::BindGroup>,
    pub front_face: wgpu::FrontFace,
    pub topology: wgpu::PrimitiveTopology,
    pub shader_key: crate::CacheKey,
    pub pipeline_key: crate::CacheKey,
    pub texture_key: Option<crate::CacheKey>,
    pub blend_state: Option<wgpu::BlendState>,
    pub cull_mode: Option<wgpu::Face>,
}

pub struct MaterialManager {
    materials: crate::HashCache<std::sync::Arc<Material>>,
}

impl MaterialManager {
    pub fn new() -> Self {
        Self {
            materials: crate::HashCache::new(),
        }
    }
    pub async fn create_material(
        &mut self,
        resources: &crate::Resources,
        shader_manager: &mut crate::ShaderManager,
        texture_manager: &mut crate::TextureManager,
        pipeline_manager: &mut crate::PipelineManager,
        config: &wgpu::SurfaceConfiguration,
        mut bind_group_layouts: Vec<&wgpu::BindGroupLayout>,
        mut bind_groups: Vec<wgpu::BindGroup>,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        material_name: &str,
        shader_rel_path: &str,
        texture_rel_path: Option<&str>,
        texture_bind_group_layout: Option<&wgpu::BindGroupLayout>,
        topology: wgpu::PrimitiveTopology,
        front_face: wgpu::FrontFace,
        polygon_mode: wgpu::PolygonMode,
        blend_state: Option<wgpu::BlendState>,
        cull_mode: Option<wgpu::Face>,
    ) -> Result<std::sync::Arc<Material>, crate::EngineError> {
        let material_key: crate::CacheKey = material_name.into();

        if let Some(cached_material) = self.materials.get(&material_key) {
            crate::log_info!("Returning cached material: {}", material_key.id);
            return Ok(cached_material.clone());
        } else {
            let shader_key = crate::CacheKey::from(shader_rel_path);
            let default_shader = shader_manager.get_or_create(shader_key.clone(), || {
                let shader_module = resources.asset_loader.load_shader(shader_rel_path)?;
                Ok(std::sync::Arc::new(shader_module))
            });

            let texture_key = if let (Some(texture_path), Some(texture_layout)) =
                (texture_rel_path, texture_bind_group_layout)
            {
                texture_manager
                    .load(
                        &resources.gpu.queue,
                        material_name,
                        &resources.asset_loader,
                        texture_path,
                    )
                    .await?;
                if let Some(texture_bind_group) = texture_manager.bind_group_for(
                    &resources.gpu.device,
                    material_name,
                    texture_layout,
                ) {
                    bind_groups.push(texture_bind_group.clone());
                    bind_group_layouts.push(texture_layout)
                };
                Some(crate::CacheKey::from(material_name))
            } else {
                None
            };
            let default_pipeline_layout =
                resources
                    .gpu
                    .device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("default pipeline layout"),
                        bind_group_layouts: &bind_group_layouts,
                        push_constant_ranges: &[],
                    });
            let pipeline_key = crate::CacheKey::from(material_name);
            let default_pipeline: std::sync::Arc<wgpu::RenderPipeline> = resources
                .gpu
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("default pipeline"),
                    layout: Some(&default_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &default_shader,
                        entry_point: Some("vs_main"),
                        buffers,
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &default_shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: config.format,
                            blend: blend_state,
                            write_mask: wgpu::ColorWrites::default(),
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: topology,
                        strip_index_format: None,
                        front_face: front_face,
                        cull_mode: cull_mode,
                        unclipped_depth: false,
                        polygon_mode,
                        conservative: false,
                    },
                    depth_stencil: Some(texture_manager.depth_stencil_state.clone()),
                    multisample: Default::default(),
                    multiview: None,
                    cache: None,
                })
                .into();

            pipeline_manager
                .render_pipelines
                .insert(pipeline_key.clone(), default_pipeline.clone());

            let material = std::sync::Arc::new(Material {
                name: material_name.to_string(),
                bind_groups,
                shader_key,
                pipeline_key,
                texture_key,
                blend_state,
                cull_mode,
                front_face,
                topology,
            });
            self.materials
                .insert(material_key.clone(), material.clone());

            return Ok(self.materials.get(&material_key).unwrap().clone());
        }
    }
}
impl crate::CacheStorage<std::sync::Arc<Material>> for MaterialManager {
    fn get(&self, key: &crate::CacheKey) -> Option<&std::sync::Arc<Material>> {
        self.materials.get(key)
    }
    fn contains(&self, key: &crate::CacheKey) -> bool {
        self.materials.contains_key(key)
    }
    fn get_mut(&mut self, key: &crate::CacheKey) -> Option<&mut std::sync::Arc<Material>> {
        self.materials.get_mut(key)
    }
    fn get_or_create<F>(
        &mut self,
        key: crate::CacheKey,
        create_fn: F,
    ) -> &mut std::sync::Arc<Material>
    where
        F: FnOnce() -> std::sync::Arc<Material>,
    {
        self.materials.entry(key).or_insert_with(create_fn)
    }
    fn insert(&mut self, key: crate::CacheKey, resource: std::sync::Arc<Material>) {
        self.materials.insert(key, resource);
    }
    fn remove(&mut self, key: &crate::CacheKey) {
        self.materials.remove(key);
    }
}
