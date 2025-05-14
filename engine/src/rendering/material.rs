use std::sync::Arc;

use crate::{CacheKey, CacheStorage, EngineError, Managers};

#[derive(Clone, Debug)]
pub struct MaterialDescriptor<'a> {
    pub name: &'a str,
    pub key: CacheKey,
    pub shader_path: &'a str,
    pub diffuse_texture: Option<&'a str>,
    pub normal_texture: Option<&'a str>,
    pub bind_group_layouts: Vec<wgpu::BindGroupLayout>,
    pub front_face: wgpu::FrontFace,
    pub topology: wgpu::PrimitiveTopology,
    pub polygon_mode: wgpu::PolygonMode,
    pub blend_state: Option<wgpu::BlendState>,
    pub cull_mode: Option<wgpu::Face>,
}

impl<'a> Default for MaterialDescriptor<'a> {
    fn default() -> Self {
        Self {
            name: "",
            key: CacheKey::default(),
            shader_path: "",
            diffuse_texture: None,
            normal_texture: None,
            bind_group_layouts: Vec::new(),
            front_face: wgpu::FrontFace::Ccw,
            topology: wgpu::PrimitiveTopology::TriangleList,
            polygon_mode: wgpu::PolygonMode::Fill,
            blend_state: Some(wgpu::BlendState::REPLACE),
            cull_mode: Some(wgpu::Face::Back),
        }
    }
}

pub struct Material {
    pub name: String,
    pub pipeline: Arc<wgpu::RenderPipeline>,
    pub bind_groups: Vec<Arc<wgpu::BindGroup>>,
}

pub struct MaterialManager {
    materials: crate::HashCache<Arc<Material>>,
}

impl MaterialManager {
    pub fn new() -> Self {
        Self {
            materials: crate::HashCache::new(),
        }
    }

    pub fn create<'a>(
        managers: &mut Managers,
        surface_config: &wgpu::SurfaceConfiguration,
        depth_stencil: &Option<wgpu::DepthStencilState>,
        desc: &mut MaterialDescriptor<'a>,
        vertex_buffers: &[wgpu::VertexBufferLayout<'a>],
    ) -> Result<Arc<Material>, EngineError> {
        if let Some(mat) = managers.material_manager.get(&desc.key) {
            return Ok(mat.clone());
        }
        let shader = managers
            .shader_manager
            .load(&managers.device, desc.shader_path)?;

        let mut bind_groups = Vec::new();
        let mut bgl_refs = Vec::new();
        for bgl in &desc.bind_group_layouts {
            bgl_refs.push(bgl);
        }
        bgl_refs.push(&crate::BindGroupLayouts::normal());

        let diffuse_texture = if let Some(diffuse_path) = desc.diffuse_texture {
            let (dt, ..) = managers.texture_manager.get_or_load_texture(
                &managers.queue,
                &managers.device,
                diffuse_path,
                surface_config,
                &crate::asset_dir()?.join("textures"),
            )?;
            dt
        } else {
            let diffuse_fallback = "diffuse_fallback";
            let fallback_key: crate::CacheKey = diffuse_fallback.into();
            desc.diffuse_texture = Some("diffuse_fallback");
            if !managers.texture_manager.contains(&fallback_key) {
                let white_pixel = [255u8, 255, 255, 255];
                let diffuse = crate::Texture::from_desc(
                    &managers.device,
                    &wgpu::TextureDescriptor {
                        label: Some("diffuse_fallback"),
                        size: wgpu::Extent3d {
                            width: 1,
                            height: 1,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                        view_formats: &[],
                    },
                );
                managers.queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &diffuse.texture,
                        mip_level: 0,
                        origin: Default::default(),
                        aspect: wgpu::TextureAspect::All,
                    },
                    &white_pixel,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(4),
                        rows_per_image: None,
                    },
                    wgpu::Extent3d {
                        width: 1,
                        height: 1,
                        depth_or_array_layers: 1,
                    },
                );

                let dt_arc = std::sync::Arc::new(diffuse);
                managers
                    .texture_manager
                    .insert(fallback_key, dt_arc.clone());
                dt_arc
            } else {
                managers.texture_manager.get(fallback_key).unwrap()
            }
        };

        let normal_texture = if let Some(norm) = desc.normal_texture {
            let (nt, ..) = managers.texture_manager.get_or_load_texture(
                &managers.queue,
                &managers.device,
                norm,
                surface_config,
                &crate::asset_dir()?.join("textures"),
            )?;
            nt
        } else {
            let normal_fallback = "normal_fallback";
            let fallback_key: crate::CacheKey = normal_fallback.into();
            desc.normal_texture = Some(normal_fallback);
            if !managers.texture_manager.contains(&fallback_key) {
                let flat_normal = [128u8, 128, 255, 255];
                let normal = crate::Texture::from_desc(
                    &managers.device,
                    &wgpu::TextureDescriptor {
                        label: Some(normal_fallback),
                        size: wgpu::Extent3d {
                            width: 1,
                            height: 1,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                        view_formats: &[],
                    },
                );
                managers.queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &normal.texture,
                        mip_level: 0,
                        origin: Default::default(),
                        aspect: wgpu::TextureAspect::All,
                    },
                    &flat_normal,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(4),
                        rows_per_image: None,
                    },
                    wgpu::Extent3d {
                        width: 1,
                        height: 1,
                        depth_or_array_layers: 1,
                    },
                );
                let n_arc = std::sync::Arc::new(normal);
                managers.texture_manager.insert(fallback_key, n_arc.clone());
                n_arc
            } else {
                managers.texture_manager.get(fallback_key).unwrap()
            }
        };

        bind_groups.push(
            crate::BindGroup::normal(
                &managers.device,
                &diffuse_texture,
                &normal_texture,
                &format!("{} normal map", desc.name),
            )
            .into(),
        );

        let pipeline_layout =
            managers
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(desc.name),
                    bind_group_layouts: &bgl_refs,
                    push_constant_ranges: &[],
                });

        let pipeline =
            managers
                .pipeline_manager
                .render
                .get_or_create(desc.key.clone(), || {
                    Arc::new(managers.device.create_render_pipeline(
                        &wgpu::RenderPipelineDescriptor {
                            label: Some(desc.shader_path),
                            layout: Some(&pipeline_layout),
                            vertex: wgpu::VertexState {
                                module: &shader,
                                entry_point: Some("vs_main"),
                                buffers: vertex_buffers,
                                compilation_options: Default::default(),
                            },
                            fragment: Some(wgpu::FragmentState {
                                module: &shader,
                                entry_point: Some("fs_main"),
                                targets: &[Some(wgpu::ColorTargetState {
                                    format: surface_config.format,
                                    blend: desc.blend_state,
                                    write_mask: wgpu::ColorWrites::ALL,
                                })],
                                compilation_options: Default::default(),
                            }),
                            primitive: wgpu::PrimitiveState {
                                topology: desc.topology,
                                strip_index_format: None,
                                front_face: desc.front_face,
                                cull_mode: desc.cull_mode,
                                polygon_mode: desc.polygon_mode,
                                unclipped_depth: false,
                                conservative: false,
                            },
                            depth_stencil: depth_stencil.as_ref().cloned(),

                            multisample: wgpu::MultisampleState {
                                count: 1,
                                mask: !0,
                                alpha_to_coverage_enabled: false,
                            },
                            multiview: None,
                            cache: None,
                        },
                    ))
                })
                .clone();

        let mat = Arc::new(Material {
            name: desc.name.to_string(),
            pipeline,
            bind_groups,
        });
        managers
            .material_manager
            .insert(desc.key.clone(), mat.clone());
        Ok(mat)
    }
    pub fn load_tobj_material<'a>(
        managers: &mut Managers,
        surface_config: &wgpu::SurfaceConfiguration,
        depth_stencil: &Option<wgpu::DepthStencilState>,
        mesh_mat: &tobj::Material,
        shader_path: &'a str,
        vertex_buffers: &'a [wgpu::VertexBufferLayout<'a>],
    ) -> Result<Arc<Material>, EngineError> {
        let mut desc = MaterialDescriptor::default();
        desc.name = &mesh_mat.name;
        desc.key = CacheKey::from(mesh_mat.name.clone());
        desc.shader_path = shader_path;
        desc.diffuse_texture = mesh_mat.diffuse_texture.as_deref();
        desc.normal_texture = mesh_mat.normal_texture.as_deref();
        desc.bind_group_layouts = vec![
            crate::BindGroupLayouts::uniform().clone(),
            crate::BindGroupLayouts::equirect_dst().clone(),
        ];
        desc.front_face = wgpu::FrontFace::Ccw;
        desc.topology = wgpu::PrimitiveTopology::TriangleList;
        desc.polygon_mode = wgpu::PolygonMode::Fill;
        desc.blend_state = Some(wgpu::BlendState::REPLACE);
        desc.cull_mode = Some(wgpu::Face::Back);

        let material = Self::create(
            managers,
            surface_config,
            depth_stencil,
            &mut desc,
            vertex_buffers,
        )?;

        Ok(material)
    }

    /// Load *all* of the tobj::Material entries for a mesh
    pub fn load_tobj_materials<'a>(
        managers: &mut Managers,
        surface_config: &wgpu::SurfaceConfiguration,
        depth_stencil: &Option<wgpu::DepthStencilState>,
        mats: &[tobj::Material],
        shader_path: &'a str,
        vertex_buffers: &'a [wgpu::VertexBufferLayout<'a>],
    ) -> Vec<Arc<Material>> {
        mats.iter()
            .filter_map(|m| {
                // ignore materials we can’t load
                match Self::load_tobj_material(
                    managers,
                    surface_config,
                    depth_stencil,
                    m,
                    shader_path,
                    vertex_buffers,
                ) {
                    Ok(mat) => Some(mat),
                    Err(e) => {
                        eprintln!("warning: failed to load material {}: {:?}", m.name, e);
                        None
                    }
                }
            })
            .collect()
    }
}

impl CacheStorage<Arc<Material>> for MaterialManager {
    fn get(&self, key: &CacheKey) -> Option<&Arc<Material>> {
        self.materials.get(key)
    }
    fn contains(&self, key: &CacheKey) -> bool {
        self.materials.contains_key(key)
    }
    fn get_mut(&mut self, key: &CacheKey) -> Option<&mut Arc<Material>> {
        self.materials.get_mut(key)
    }
    fn get_or_create<F>(&mut self, key: CacheKey, create_fn: F) -> &mut Arc<Material>
    where
        F: FnOnce() -> Arc<Material>,
    {
        self.materials.get_or_create(key, create_fn)
    }
    fn insert(&mut self, key: CacheKey, resource: Arc<Material>) {
        self.materials.insert(key, resource);
    }
    fn remove(&mut self, key: &CacheKey) {
        self.materials.remove(key);
    }
}
