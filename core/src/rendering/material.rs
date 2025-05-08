use std::sync::Arc;

use crate::{log_info, CacheStorage};

#[derive(Clone, Debug)]
pub struct MaterialDescriptor<'a> {
    pub name: &'a str,
    pub key: crate::CacheKey,
    pub shader: &'a str,
    pub bind_groups: &'a [&'a std::sync::Arc<wgpu::BindGroup>],
    pub bind_group_layouts: &'a [&'a wgpu::BindGroupLayout],
    pub diffuse_texture: Option<&'a str>,
    pub normal_texture: Option<&'a str>,
    pub front_face: wgpu::FrontFace,
    pub topology: wgpu::PrimitiveTopology,
    polygon_mode: wgpu::PolygonMode,
    pub blend_state: Option<wgpu::BlendState>,
    pub cull_mode: Option<wgpu::Face>,
}

impl<'a> Default for MaterialDescriptor<'a> {
    fn default() -> Self {
        Self {
            name: Default::default(),
            key: Default::default(),
            shader: Default::default(),
            bind_groups: Default::default(),
            bind_group_layouts: Default::default(),
            diffuse_texture: Default::default(),
            normal_texture: Default::default(),
            front_face: Default::default(),
            topology: Default::default(),
            polygon_mode: Default::default(),
            blend_state: Default::default(),
            cull_mode: Default::default(),
        }
    }
}

impl<'a> MaterialDescriptor<'a> {
    pub fn new(
        name: &'a str,
        key: crate::CacheKey,
        shader: &'a str,
        bind_groups: &'a [&'a std::sync::Arc<wgpu::BindGroup>],
        bind_group_layouts: &'a [&'a wgpu::BindGroupLayout],
        diffuse_texture: Option<&'a str>,
        normal_texture: Option<&'a str>,
        front_face: wgpu::FrontFace,
        topology: wgpu::PrimitiveTopology,
        polygon_mode: wgpu::PolygonMode,
        blend_state: Option<wgpu::BlendState>,
        cull_mode: Option<wgpu::Face>,
    ) -> Self {
        Self {
            name,
            key,
            shader,
            bind_groups,
            bind_group_layouts,
            diffuse_texture,
            normal_texture,
            front_face,
            topology,
            polygon_mode,
            blend_state,
            cull_mode,
        }
    }
}

impl<'a> Into<Material> for MaterialDescriptor<'a> {
    fn into(self) -> Material {
        let mut bind_groups = Vec::new();
        let mut bind_group_layouts = Vec::new();

        for layout in self.bind_group_layouts {
            bind_group_layouts.push(layout.to_owned().clone());
        }

        for group in self.bind_groups {
            bind_groups.push(group.to_owned().clone());
        }
        let key = crate::CacheKey::from(self.name);
        Material {
            name: self.name.into(),
            key,
            bind_groups,
            bind_group_layouts,
            front_face: self.front_face,
            topology: self.topology,
            polygon_mode: self.polygon_mode,
            shader_key: crate::CacheKey::from(self.shader),
            diffuse_texture_key: if let Some(d) = self.diffuse_texture {
                Some(crate::CacheKey::from(d))
            } else {
                None
            },
            normal_texture_key: if let Some(n) = self.normal_texture {
                Some(crate::CacheKey::from(n))
            } else {
                None
            },
            blend_state: self.blend_state,
            cull_mode: self.cull_mode,
        }
    }
}
impl<'a> Into<Material> for &MaterialDescriptor<'a> {
    fn into(self) -> Material {
        let mut bind_groups = Vec::new();
        let mut bind_group_layouts = Vec::new();

        for layout in self.bind_group_layouts {
            bind_group_layouts.push(layout.to_owned().clone());
        }

        for group in self.bind_groups {
            bind_groups.push(group.to_owned().clone());
        }

        Material {
            name: self.name.into(),
            key: crate::CacheKey::from(self.name),
            bind_groups,
            bind_group_layouts,
            front_face: self.front_face,
            topology: self.topology,
            polygon_mode: self.polygon_mode,
            shader_key: crate::CacheKey::from(self.shader),
            diffuse_texture_key: if let Some(d) = self.diffuse_texture {
                Some(crate::CacheKey::from(d))
            } else {
                None
            },
            normal_texture_key: if let Some(n) = self.normal_texture {
                Some(crate::CacheKey::from(n))
            } else {
                None
            },
            blend_state: self.blend_state,
            cull_mode: self.cull_mode,
        }
    }
}

#[derive(Clone)]
pub struct Material {
    pub name: String,
    pub key: crate::CacheKey,
    pub bind_groups: Vec<std::sync::Arc<wgpu::BindGroup>>,
    pub bind_group_layouts: Vec<wgpu::BindGroupLayout>,
    pub front_face: wgpu::FrontFace,
    pub topology: wgpu::PrimitiveTopology,
    pub polygon_mode: wgpu::PolygonMode,
    pub shader_key: crate::CacheKey,
    pub diffuse_texture_key: Option<crate::CacheKey>,
    pub normal_texture_key: Option<crate::CacheKey>,
    pub blend_state: Option<wgpu::BlendState>,
    pub cull_mode: Option<wgpu::Face>,
}
impl Default for Material {
    fn default() -> Self {
        MaterialDescriptor::default().into()
    }
}
impl Material {
    pub fn from_desc(desc: MaterialDescriptor) -> Self {
        desc.into()
    }
    pub fn new(
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        bind_groups: Vec<std::sync::Arc<wgpu::BindGroup>>,
        material_name: &str,
        shader_rel_path: &str,
        diffuse_texture: Option<&str>,
        normal_texture: Option<&str>,
        topology: wgpu::PrimitiveTopology,
        front_face: wgpu::FrontFace,
        polygon_mode: wgpu::PolygonMode,
        blend_state: Option<wgpu::BlendState>,
        cull_mode: Option<wgpu::Face>,
    ) -> Material {
        Material {
            name: material_name.to_string(),
            key: crate::CacheKey::from(material_name),
            bind_groups,
            bind_group_layouts,
            shader_key: crate::CacheKey::from(shader_rel_path),
            diffuse_texture_key: if let Some(d) = diffuse_texture {
                Some(crate::CacheKey::from(d))
            } else {
                None
            },
            normal_texture_key: if let Some(n) = normal_texture {
                Some(crate::CacheKey::from(n))
            } else {
                None
            },
            blend_state,
            cull_mode,
            front_face,
            topology,
            polygon_mode,
        }
    }
    pub fn load_tobj_materials(
        managers: &mut crate::Managers,
        uniform_bind_group: &wgpu::BindGroup,
        surface_config: &wgpu::SurfaceConfiguration,
        depth_stencil_state: &Option<wgpu::DepthStencilState>,
        camera: &crate::camera::Camera,
        light: &crate::Light,
        mats: &[tobj::Material],
        shader_rel_path: &str,
    ) -> Result<Vec<crate::Material>, crate::EngineError> {
        mats.iter()
            .map(|m| {
                crate::Material::from_tobj_material(
                    managers,
                    uniform_bind_group,
                    light,
                    surface_config,
                    depth_stencil_state,
                    m,
                    shader_rel_path,
                )
            })
            .collect()
    }

    pub fn from_tobj_material(
        managers: &mut crate::Managers,
        uniform_bind_group: &wgpu::BindGroup,
        light: &crate::Light,
        surface_config: &wgpu::SurfaceConfiguration,
        depth_stencil_state: &Option<wgpu::DepthStencilState>,
        mat: &tobj::Material,
        shader_rel_path: &str,
    ) -> Result<crate::Material, crate::EngineError> {
        let mut bind_groups = Vec::new();
        let mut bind_group_layouts = vec![];

        bind_groups.push(Arc::new(uniform_bind_group.clone()));
        bind_group_layouts.push(crate::BindGroupLayouts::uniform().clone());

        let base_dir = crate::asset_dir()?.join("textures");

        let (d_key, n_key) = if let (Some(d), Some(n)) =
            (mat.diffuse_texture.as_ref(), mat.normal_texture.as_ref())
        {
            let (diffuse_tex, diffuse_key) = managers.texture_manager.get_or_load_texture(
                &managers.queue,
                &managers.device,
                d,
                surface_config,
                &base_dir,
            )?;
            let (normal_tex, normal_key) = managers.texture_manager.get_or_load_texture(
                &managers.queue,
                &managers.device,
                n,
                surface_config,
                &base_dir,
            )?;

            bind_groups.push(
                crate::BindGroup::normal_map(
                    &managers.device,
                    &diffuse_tex,
                    &normal_tex,
                    &format!("{} normal map", mat.name),
                )
                .into(),
            );
            bind_group_layouts.push(crate::BindGroupLayouts::normal_map().clone());
            (Some(diffuse_key), Some(normal_key))
        } else {
            (None, None)
        };

        if let Some(bind_group) = managers.bind_group_manager.bind_group_for(
            &managers.texture_manager,
            &"equirect projection destination",
            &crate::BindGroupLayouts::equirect_dst(),
        ) {
            bind_groups.push(bind_group.clone());
            bind_group_layouts.push(crate::BindGroupLayouts::equirect_dst().clone());
        }

        let shader_key = crate::CacheKey::from(shader_rel_path);
        let shader_module = crate::Asset::shader(managers, &shader_key.id).expect(&format!(
            "AssetLoader load shader failed for {}",
            shader_key.id
        ));
        let bind_group_layout_refs: Vec<&wgpu::BindGroupLayout> =
            bind_group_layouts.iter().collect();
        log_info!("LAYOUTS: {:?}", bind_group_layout_refs.len());
        let pipeline_layout =
            managers
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(&format!("{} layout", shader_key.id)),
                    bind_group_layouts: &bind_group_layout_refs,
                    push_constant_ranges: &[],
                });
        crate::CacheStorage::get_or_create(
            &mut managers.render_pipeline_manager,
            crate::CacheKey::from(mat.name.clone()),
            || {
                managers
                    .device
                    .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some(&shader_key.id),
                        layout: Some(&pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &shader_module,
                            entry_point: Some("vs_main"),
                            buffers: &[
                                crate::VertexNormal::LAYOUT,
                                crate::VertexNormalInstance::LAYOUT,
                            ],
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
                            cull_mode: Some(wgpu::Face::Back),
                            polygon_mode: wgpu::PolygonMode::Fill,
                            unclipped_depth: false,
                            conservative: false,
                        },
                        depth_stencil: depth_stencil_state.as_ref().cloned(),

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
            key: crate::CacheKey::from(mat.name.clone()),
            bind_groups,
            bind_group_layouts: bind_group_layouts.into(),
            front_face: wgpu::FrontFace::Ccw,
            topology: wgpu::PrimitiveTopology::TriangleList,
            shader_key,
            diffuse_texture_key: d_key,
            normal_texture_key: n_key,
            blend_state: None,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
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
