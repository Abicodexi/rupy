use crate::CacheStorage;

#[derive(Clone)]
pub struct Material {
    pub name: String,
    pub bind_groups: Vec<std::sync::Arc<wgpu::BindGroup>>,
    pub front_face: wgpu::FrontFace,
    pub topology: wgpu::PrimitiveTopology,
    pub shader_key: crate::CacheKey,
    pub texture_key: Option<crate::CacheKey>,
    pub blend_state: Option<wgpu::BlendState>,
    pub cull_mode: Option<wgpu::Face>,
}
impl Default for Material {
    fn default() -> Self {
        Self {
            name: Default::default(),
            bind_groups: Default::default(),
            front_face: Default::default(),
            topology: Default::default(),
            shader_key: crate::CacheKey {
                id: "v_texture.wgsl".to_string(),
            },

            texture_key: Default::default(),
            blend_state: Default::default(),
            cull_mode: Default::default(),
        }
    }
}
impl Material {
    pub async fn create(
        device: &wgpu::Device,
        managers: &mut crate::Managers,
        config: &wgpu::SurfaceConfiguration,
        mut bind_group_layouts: Vec<&wgpu::BindGroupLayout>,
        mut bind_groups: Vec<std::sync::Arc<wgpu::BindGroup>>,
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

        if let Some(cached_material) = managers.material_manager.get(&material_key) {
            crate::log_info!("Returning cached material: {}", material_key.id);
            return Ok(cached_material.clone());
        } else {
            let shader_key = crate::CacheKey::from(shader_rel_path);
            let default_shader = crate::Shader::load(managers, shader_rel_path)?;

            let texture_key = if let (Some(texture_path), Some(texture_layout)) =
                (texture_rel_path, texture_bind_group_layout)
            {
                if !managers
                    .texture_manager
                    .contains(&crate::CacheKey::from(texture_path))
                {
                    crate::AssetLoader::load_texture(managers, texture_path).await?;
                }
                if let Some(texture_bind_group) = managers.bind_group_manager.bind_group_for(
                    &managers.texture_manager,
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
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("default pipeline layout"),
                    bind_group_layouts: &bind_group_layouts,
                    push_constant_ranges: &[],
                });
            let pipeline_key = crate::CacheKey::from(material_name);
            let default_pipeline: std::sync::Arc<wgpu::RenderPipeline> = device
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
                    depth_stencil: Some(managers.texture_manager.depth_stencil_state.clone()),
                    multisample: Default::default(),
                    multiview: None,
                    cache: None,
                })
                .into();

            crate::CacheStorage::insert(
                &mut managers.render_pipeline_manager,
                pipeline_key.clone(),
                default_pipeline,
            );

            let material = std::sync::Arc::new(Material {
                name: material_name.to_string(),
                bind_groups,
                shader_key,
                texture_key,
                blend_state,
                cull_mode,
                front_face,
                topology,
            });
            managers
                .material_manager
                .insert(material_key.clone(), material.clone());
            let material = managers.material_manager.get(&material_key).unwrap();
            return Ok(material.clone());
        }
    }
    pub fn load_tobj_materials(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        managers: &mut crate::Managers,
        camera: &crate::camera::Camera,
        surface_config: &wgpu::SurfaceConfiguration,
        mats: &[tobj::Material],
    ) -> Result<Vec<crate::Material>, crate::EngineError> {
        mats.iter()
            .map(|m| {
                crate::Material::from_tobj_material(
                    queue,
                    device,
                    managers,
                    camera,
                    surface_config,
                    m,
                )
            })
            .collect()
    }
    pub fn from_tobj_material(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        managers: &mut crate::Managers,
        camera: &crate::camera::Camera,
        surface_config: &wgpu::SurfaceConfiguration,
        mat: &tobj::Material,
    ) -> Result<crate::Material, crate::EngineError> {
        let mut bind_groups: Vec<std::sync::Arc<wgpu::BindGroup>> = Vec::new();
        let mut bind_group_layouts = vec![];
        if let Some(camera_bind_group) = managers.bind_group_manager.get(&camera.bind_group) {
            bind_groups.push(camera_bind_group.clone());
            bind_group_layouts.push(crate::BindGroupLayouts::camera());
        }

        let base_dir_path = crate::asset_dir()?.join("textures");
        let texture_key = if let Some(diffuse_texture) = &mat.diffuse_texture {
            let tex_path = base_dir_path.join(&diffuse_texture);
            let img = image::open(&tex_path)
                .map_err(|e| {
                    crate::EngineError::AssetLoadError(format!("Texture load failed: {}", e))
                })?
                .to_rgba8();

            let texture = crate::CacheStorage::get_or_create(
                &mut managers.texture_manager,
                crate::CacheKey {
                    id: diffuse_texture.clone(),
                },
                || {
                    crate::Texture::from_image(device, queue, surface_config, &img, diffuse_texture)
                        .into()
                },
            );
            let tex_key = crate::CacheKey::from(texture.label.clone());
            Some(tex_key)
        } else {
            None
        };
        if let Some(tex_key) = &texture_key {
            let texture_bind_group_layout = crate::BindGroupLayouts::texture();
            if let Some(bind_group) = managers.bind_group_manager.bind_group_for(
                &managers.texture_manager,
                &tex_key.id,
                &texture_bind_group_layout,
            ) {
                bind_groups.push(bind_group.clone());
                bind_group_layouts.push(texture_bind_group_layout);
            }
        }
        let shader_key = crate::CacheKey::from("v_texture.wgsl");
        let shader_module = crate::AssetLoader::load_shader(managers, &shader_key.id).expect(
            &format!("AssetLoader load shader failed for {}", shader_key.id),
        );
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{} layout", shader_key.id)),
            bind_group_layouts: &bind_group_layouts,
            push_constant_ranges: &[],
        });

        let _pipeline = crate::CacheStorage::get_or_create(
            &mut managers.render_pipeline_manager,
            shader_key.clone(),
            || {
                device
                    .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some(&shader_key.id),
                        layout: Some(&pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &shader_module,
                            entry_point: Some("vs_main"),
                            buffers: &[crate::VertexTexture::LAYOUT, crate::InstanceData::LAYOUT],
                            compilation_options: Default::default(),
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader_module,
                            entry_point: Some("fs_main"),
                            targets: &[Some(wgpu::ColorTargetState {
                                format: surface_config.format,
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
                        depth_stencil: Some(managers.texture_manager.depth_stencil_state.clone()),

                        multisample: wgpu::MultisampleState {
                            count: 1,
                            mask: !0,
                            alpha_to_coverage_enabled: false,
                        },
                        multiview: None,
                        cache: None,
                    })
                    .into()
            },
        );
        let material = crate::Material {
            name: mat.name.clone(),
            bind_groups,
            front_face: wgpu::FrontFace::Ccw,
            topology: wgpu::PrimitiveTopology::TriangleList,
            shader_key,
            texture_key,
            blend_state: None,
            cull_mode: Some(wgpu::Face::Back),
        };
        managers
            .material_manager
            .materials
            .insert(material.name.clone().into(), material.clone().into());
        Ok(material)
    }
}
pub struct MaterialManager {
    pub materials: crate::HashCache<std::sync::Arc<Material>>,
}

impl MaterialManager {
    pub fn new() -> Self {
        Self {
            materials: crate::HashCache::new(),
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
