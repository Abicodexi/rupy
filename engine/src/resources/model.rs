use super::{CacheKey, HashCache, MaterialAsset, MaterialManager, Mesh, MeshAsset, MeshInstance};
use crate::{
    log_info, log_warning, Asset, EngineError, PipelineManager, ShaderManager, TextureManager, AABB,
};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone, Debug)]
pub struct ModelAsset {
    pub name: String,
    pub asset: (MeshAsset, Option<MaterialAsset>),
    pub aabb: AABB,
}

impl ModelAsset {
    pub fn load_asset(
        &self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        material_manager: &mut MaterialManager,
        texture_manager: &mut TextureManager,
        shader_manager: &mut ShaderManager,
        pipeline_manager: &mut PipelineManager,
        bind_group_layouts: &Vec<&wgpu::BindGroupLayout>,
        surface_configuration: &wgpu::SurfaceConfiguration,
        buffers: &[wgpu::VertexBufferLayout<'_>],
    ) -> Result<(super::MeshInstance, AABB), EngineError> {
        let material = if let Some(asset) = &self.asset.1 {
            let m = material_manager.load_asset(
                device,
                queue,
                texture_manager,
                shader_manager,
                pipeline_manager,
                bind_group_layouts,
                asset.clone(),
                surface_configuration,
                buffers,
            )?;
            Some(m)
        } else {
            None
        };

        let aabb = AABB::from_vertices(&self.asset.0.vertices);
        let mesh = Mesh::from_asset(queue, device, self.asset.0.clone(), &self.name);
        let instance = MeshInstance {
            mesh: Arc::new(mesh),
            material,
        };
        Ok((instance, aabb))
    }
}

pub struct Model {
    pub name: String,
    pub instance: MeshInstance,
    pub aabb: AABB,
}

impl Model {
    pub fn from_asset(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        material_manager: &mut MaterialManager,
        texture_manager: &mut TextureManager,
        shader_manager: &mut ShaderManager,
        pipeline_manager: &mut PipelineManager,
        bind_group_layouts: &Vec<&wgpu::BindGroupLayout>,
        surface_configuration: &wgpu::SurfaceConfiguration,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        asset: ModelAsset,
    ) -> std::result::Result<Self, EngineError> {
        let (instance, aabb) = asset.load_asset(
            queue,
            device,
            material_manager,
            texture_manager,
            shader_manager,
            pipeline_manager,
            bind_group_layouts,
            surface_configuration,
            buffers,
        )?;
        Ok(Self {
            name: asset.name,
            instance,
            aabb,
        })
    }
    pub fn from_tobj(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        material_manager: &mut MaterialManager,
        texture_manager: &mut TextureManager,
        shader_manager: &mut ShaderManager,
        pipeline_manager: &mut PipelineManager,
        model: &tobj::Model,
        material: Option<&tobj::Material>,
        v_shader: &str,
        f_shader: &str,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        surface_configuration: &wgpu::SurfaceConfiguration,
        primitive: wgpu::PrimitiveState,
        depth_stencil: Option<wgpu::DepthStencilState>,
        color_target: wgpu::ColorTargetState,
        bind_group_layouts: &Vec<&wgpu::BindGroupLayout>,
    ) -> Result<Model, EngineError> {
        let vertices = MeshAsset::compute_vertex(&model);
        let indices = model.mesh.indices.clone();
        let model_asset = ModelAsset {
            name: model.name.clone(),
            asset: (MeshAsset { vertices, indices }, {
                if let Some(mat) = material {
                    let mut mat_asset: MaterialAsset = mat.into();
                    mat_asset.primitive = primitive;
                    mat_asset.color_target = color_target;
                    mat_asset.depth_stencil = depth_stencil;
                    mat_asset.v_shader = v_shader.to_string();
                    mat_asset.f_shader = f_shader.to_string();

                    Some(mat_asset)
                } else {
                    log_info!("No material found");
                    None
                }
            }),
            aabb: AABB::default(),
        };
        let (instance, aabb) = model_asset.load_asset(
            queue,
            device,
            material_manager,
            texture_manager,
            shader_manager,
            pipeline_manager,
            bind_group_layouts,
            surface_configuration,
            buffers,
        )?;
        Ok(Self {
            name: model.name.clone(),
            instance,
            aabb,
        })
    }
}

pub struct ModelManager {
    pub models: HashCache<Arc<Model>>,
}
impl ModelManager {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    pub async fn load(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        material_manager: &mut MaterialManager,
        texture_manager: &mut TextureManager,
        shader_manager: &mut ShaderManager,
        pipeline_manager: &mut PipelineManager,
        file: &str,
        v_shader: &str,
        f_shader: &str,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        bind_group_layouts: &Vec<&wgpu::BindGroupLayout>,
        surface_configuration: &wgpu::SurfaceConfiguration,
        primitive: wgpu::PrimitiveState,
        color_target: wgpu::ColorTargetState,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> Result<(), EngineError> {
        let base_dir = Asset::base_path();
        let path = base_dir.join("models").join(file);
        let (models, mat_res) = tobj::load_obj(
            &path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )?;

        let materials = match mat_res {
            Ok(mats) => mats,
            Err(e) => {
                log_warning!("{}: {}", file, e);
                Vec::new()
            }
        };

        for m in models {
            let m_key = CacheKey::from(file);
            if self.models.contains_key(&m_key) {
                log_info!("Skipping cached model: {}", m.name);
                continue;
            }
            let mesh = &m.mesh;

            let mat = {
                if let Some(id) = mesh.material_id {
                    materials.get(id)
                } else {
                    None
                }
            };

            let model = Arc::new(Model::from_tobj(
                queue,
                device,
                material_manager,
                texture_manager,
                shader_manager,
                pipeline_manager,
                &m,
                mat,
                v_shader,
                f_shader,
                buffers,
                surface_configuration,
                primitive,
                depth_stencil.clone(),
                color_target.clone(),
                bind_group_layouts,
            )?);

            self.models.insert(m_key, model);
            log_info!("Cached model: {}", m.name);
        }
        Ok(())
    }
    pub fn load_asset(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        material_manager: &mut MaterialManager,
        texture_manager: &mut TextureManager,
        shader_manager: &mut ShaderManager,
        pipeline_manager: &mut PipelineManager,
        bind_group_layouts: &Vec<&wgpu::BindGroupLayout>,
        surface_configuration: &wgpu::SurfaceConfiguration,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        asset: ModelAsset,
    ) -> Result<Arc<Model>, EngineError> {
        let m_key = CacheKey::from(asset.name.clone());
        if let Some(m) = self.models.get(&m_key) {
            return Ok(m.clone());
        }
        let model = Arc::new(Model::from_asset(
            queue,
            device,
            material_manager,
            texture_manager,
            shader_manager,
            pipeline_manager,
            bind_group_layouts,
            surface_configuration,
            buffers,
            asset,
        )?);
        self.models.insert(m_key, model.clone());
        Ok(model)
    }
}

impl crate::CacheStorage<std::sync::Arc<Model>> for ModelManager {
    fn get(&self, key: &crate::CacheKey) -> Option<&std::sync::Arc<Model>> {
        self.models.get(key)
    }
    fn contains(&self, key: &crate::CacheKey) -> bool {
        self.models.contains_key(key)
    }
    fn get_mut(&mut self, key: &crate::CacheKey) -> Option<&mut std::sync::Arc<Model>> {
        self.models.get_mut(key)
    }
    fn get_or_create<F>(&mut self, key: crate::CacheKey, create_fn: F) -> &mut std::sync::Arc<Model>
    where
        F: FnOnce() -> std::sync::Arc<Model>,
    {
        let start = std::time::Instant::now();
        let model = self.models.entry(key).or_insert_with(create_fn);
        crate::log_debug!("Loaded in {:.2?}", start.elapsed());
        model
    }
    fn insert(&mut self, key: crate::CacheKey, resource: std::sync::Arc<Model>) {
        self.models.insert(key, resource);
    }
    fn remove(&mut self, key: &crate::CacheKey) -> Option<std::sync::Arc<Model>> {
        self.models.remove(key)
    }
}
