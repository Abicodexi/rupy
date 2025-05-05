use super::VertexTexture;
use crate::{
    log_info, AssetLoader, CacheKey, CacheStorage, EngineError, GpuContext, HashCache,
    InstanceData, Managers,
};
use std::sync::Arc;
use wgpu::SurfaceConfiguration;

#[derive(Clone)]
pub struct Material {
    pub name: String,
    pub bind_groups: Vec<wgpu::BindGroup>,
    pub shader_key: CacheKey,
    pub pipeline_key: CacheKey,
    pub texture_key: Option<CacheKey>,
    pub blend_state: wgpu::BlendState,
    pub cull_mode: wgpu::Face,
    pub front_face: wgpu::FrontFace,
    pub topology: wgpu::PrimitiveTopology,
}

pub struct MaterialManager {
    materials: HashCache<Arc<Material>>,
}

impl MaterialManager {
    pub fn new() -> Self {
        Self {
            materials: HashCache::new(),
        }
    }
    pub async fn create_material(
        gpu: &GpuContext,
        asset_loader: &AssetLoader,
        managers: &mut Managers,
        config: &SurfaceConfiguration,
        mut bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        mut bind_groups: Vec<wgpu::BindGroup>,
        material_name: &str,
        shader_rel_path: &str,
        texture_rel_path: Option<&str>,
        texture_bind_group_layout: Option<&wgpu::BindGroupLayout>,
        blend_state: wgpu::BlendState,
        cull_mode: wgpu::Face,
        topology: wgpu::PrimitiveTopology,
        front_face: wgpu::FrontFace,
        polygon_mode: wgpu::PolygonMode,
    ) -> Result<Arc<Material>, EngineError> {
        let material_key: CacheKey = material_name.into();

        if let Some(cached_material) = managers.material_manager.get(material_key.clone()) {
            log_info!("Returning cached material: {}", material_key.id);
            return Ok(cached_material.clone());
        } else {
            let shader_key = CacheKey::from(shader_rel_path);
            let default_shader = managers
                .shader_manager
                .get_or_create(shader_key.clone(), || {
                    let shader_module = asset_loader.load_shader(shader_rel_path)?;
                    Ok(Arc::new(shader_module))
                });

            let texture_key = if let (Some(texture_path), Some(texture_layout)) =
                (texture_rel_path, texture_bind_group_layout)
            {
                managers
                    .texture_manager
                    .load(material_name, &asset_loader, texture_path)
                    .await?;
                if let Some(texture_bind_group) = managers
                    .texture_manager
                    .bind_group_for(material_name, texture_layout)
                {
                    bind_groups.push(texture_bind_group.clone());
                    bind_group_layouts.push(texture_layout.clone())
                };
                Some(CacheKey::from(material_name))
            } else {
                None
            };
            let ref_layouts: Vec<&wgpu::BindGroupLayout> = bind_group_layouts.iter().collect();
            let default_pipeline_layout =
                gpu.device()
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("default pipeline layout"),
                        bind_group_layouts: &ref_layouts,
                        push_constant_ranges: &[],
                    });
            let pipeline_key = CacheKey::from(material_name);
            let default_pipeline: Arc<wgpu::RenderPipeline> = gpu
                .device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("default pipeline"),
                    layout: Some(&default_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &default_shader,
                        entry_point: Some("vs_main"),
                        buffers: &[VertexTexture::LAYOUT, InstanceData::LAYOUT],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &default_shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: config.format,
                            blend: Some(blend_state),
                            write_mask: wgpu::ColorWrites::default(),
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: topology,
                        strip_index_format: None,
                        front_face: front_face,
                        cull_mode: Some(cull_mode),
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

            managers
                .pipeline_manager
                .render_pipelines
                .insert(pipeline_key.clone(), default_pipeline.clone());

            let material = Arc::new(Material {
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
            let material_clone = material.clone();
            managers.material_manager.insert(material_key, material);

            return Ok(material_clone);
        }
    }
}
impl CacheStorage<Arc<Material>> for MaterialManager {
    fn get<K: Into<CacheKey>>(&self, key: K) -> Option<&Arc<Material>> {
        self.materials.get(&key.into())
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
        self.materials.entry(key).or_insert_with(create_fn)
    }
    fn insert(&mut self, key: CacheKey, resource: Arc<Material>) {
        self.materials.insert(key, resource);
    }
    fn remove(&mut self, key: &CacheKey) {
        self.materials.remove(key);
    }
}
