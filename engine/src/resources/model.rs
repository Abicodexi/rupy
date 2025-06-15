use super::{CacheKey, HashCache, MaterialAsset, MaterialManager, Mesh, MeshAsset, MeshInstance};
use crate::{
    log_debug, log_info, log_warning, AssetLoader, BindGroupManager, EngineError, PipelineManager,
    RenderBindGroupLayouts, ShaderManager, TextureManager, AABB,
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
        bind_group_manager: &mut BindGroupManager,
        layouts: &RenderBindGroupLayouts,
        bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
        format: wgpu::TextureFormat,
        buffers: &[wgpu::VertexBufferLayout<'_>],
    ) -> Result<(super::MeshInstance, AABB), EngineError> {
        let material = if let Some(asset) = &self.asset.1 {
            let m = material_manager.load_asset(
                device,
                queue,
                texture_manager,
                shader_manager,
                pipeline_manager,
                bind_group_manager,
                layouts,
                bind_group_layouts,
                asset.clone(),
                format,
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
        bind_group_manager: &mut BindGroupManager,
        layouts: &RenderBindGroupLayouts,

        bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
        format: wgpu::TextureFormat,
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
            bind_group_manager,
            layouts,
            bind_group_layouts,
            format,
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
        bind_group_manager: &mut BindGroupManager,
        layouts: &RenderBindGroupLayouts,
        model: &tobj::Model,
        material: Option<&tobj::Material>,
        v_shader: &str,
        f_shader: &str,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        format: wgpu::TextureFormat,
        primitive: wgpu::PrimitiveState,
        depth_stencil: Option<wgpu::DepthStencilState>,
        color_target: wgpu::ColorTargetState,
        bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
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
            bind_group_manager,
            layouts,
            bind_group_layouts,
            format,
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
        bind_group_manager: &mut BindGroupManager,
        layouts: &RenderBindGroupLayouts,
        file: &str,
        v_shader: &str,
        f_shader: &str,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
        format: wgpu::TextureFormat,
        primitive: wgpu::PrimitiveState,
        color_target: wgpu::ColorTargetState,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> Result<(), EngineError> {
        let path = AssetLoader::resolve("models").join(file);
        let m_key = CacheKey::from(file);
        if self.models.contains_key(&m_key) {
            return Ok(());
        }
        let (models, m) = AssetLoader::tobj(path)?;
        let materials = match m {
            Ok(mats) => mats,
            Err(e) => {
                log_warning!("{}: {}", file, e);
                Vec::new()
            }
        };
        log_debug!("Loaded model: {}", file);

        for m in models {
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
                bind_group_manager,
                layouts,
                &m,
                mat,
                v_shader,
                f_shader,
                buffers,
                format,
                primitive,
                depth_stencil.clone(),
                color_target.clone(),
                bind_group_layouts.clone(),
            )?);

            self.models.insert(m_key, model);
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
        bind_group_manager: &mut BindGroupManager,
        layouts: &RenderBindGroupLayouts,
        bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
        format: wgpu::TextureFormat,
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
            bind_group_manager,
            layouts,
            bind_group_layouts,
            format,
            buffers,
            asset,
        )?);
        log_debug!("Loaded model asset: {}", model.name);
        self.models.insert(m_key, model.clone());
        Ok(model)
    }
}

impl crate::CacheStorage<std::sync::Arc<Model>> for ModelManager {
    fn get_resource(&self, key: &crate::CacheKey) -> Option<&std::sync::Arc<Model>> {
        self.models.get(key)
    }
    fn contains_resource(&self, key: &crate::CacheKey) -> bool {
        self.models.contains_key(key)
    }
    fn get_mut(&mut self, key: &crate::CacheKey) -> Option<&mut std::sync::Arc<Model>> {
        self.models.get_mut(key)
    }
    fn get_or_create<F>(
        &mut self,
        key: crate::CacheKey,
        create_fn: F,
    ) -> Result<Arc<Model>, EngineError>
    where
        F: FnOnce() -> Result<Arc<Model>, EngineError>,
    {
        self.models.get_or_create(key, create_fn)
    }
    fn insert_resource(&mut self, key: crate::CacheKey, resource: std::sync::Arc<Model>) {
        self.models.insert(key, resource);
    }
    fn remove_resource(&mut self, key: &crate::CacheKey) -> Option<std::sync::Arc<Model>> {
        self.models.remove(key)
    }

    fn all<'a>(&'a self) -> impl Iterator<Item = &'a std::sync::Arc<Model>>
    where
        std::sync::Arc<Model>: 'a,
    {
        self.models.values()
    }
}
