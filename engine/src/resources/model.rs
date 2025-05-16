use super::{
    CacheKey, HashCache, Material, MaterialAsset, MaterialManager, Mesh, MeshAsset, MeshInstance,
};
use crate::{log_info, log_warning, Asset, EngineError, AABB};
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
        materials: &mut MaterialManager,
        surface_configuration: &wgpu::SurfaceConfiguration,
        buffers: &[wgpu::VertexBufferLayout<'_>],
    ) -> Result<(super::MeshInstance, AABB), EngineError> {
        let (mesh, mat) = &self.asset;
        let material = if let Some(m) = mat {
            let idx = materials.materials.len() as u32 + 1;
            Some(Arc::new(Material::from_asset(
                queue,
                device,
                &mut materials.textures,
                &mut materials.shaders,
                &mut materials.pipelines,
                surface_configuration,
                buffers,
                m.clone(),
                idx,
            )?))
        } else {
            None
        };
        let aabb = AABB::from_vertices(&mesh.vertices);
        let mesh = Mesh::from_asset(queue, device, mesh.clone(), &self.name);
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
        materials: &mut MaterialManager,
        surface_configuration: &wgpu::SurfaceConfiguration,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        asset: ModelAsset,
    ) -> std::result::Result<Self, EngineError> {
        let (instance, aabb) =
            asset.load_asset(queue, device, materials, surface_configuration, buffers)?;
        Ok(Self {
            name: asset.name,
            instance,
            aabb,
        })
    }
    pub fn from_tobj(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        materials: &mut MaterialManager,
        model: &tobj::Model,
        material: Option<&tobj::Material>,
        shader: &str,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        surface_configuration: &wgpu::SurfaceConfiguration,
        primitive: wgpu::PrimitiveState,
        depth_stencil: Option<wgpu::DepthStencilState>,
        color_target: wgpu::ColorTargetState,
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
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
                    mat_asset.bind_group_layouts = bind_group_layouts;
                    mat_asset.shader = shader.to_string();

                    Some(mat_asset)
                } else {
                    log_info!("No material found");
                    None
                }
            }),
            aabb: AABB::default(),
        };
        let (instance, aabb) =
            model_asset.load_asset(queue, device, materials, surface_configuration, buffers)?;
        Ok(Self {
            name: model.name.clone(),
            instance,
            aabb,
        })
    }
}

pub struct ModelManager {
    pub models: HashCache<Arc<Model>>,
    pub materials: MaterialManager,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
}
impl ModelManager {
    pub fn new(queue: Arc<wgpu::Queue>, device: Arc<wgpu::Device>) -> Self {
        Self {
            models: HashMap::new(),
            materials: MaterialManager::new(&device),
            device,
            queue,
        }
    }
    pub async fn load_object_file(
        &mut self,
        file: &str,
        shader: &str,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
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
                &self.queue,
                &self.device,
                &mut self.materials,
                &m,
                mat,
                &shader,
                buffers,
                surface_configuration,
                primitive,
                depth_stencil.clone(),
                color_target.clone(),
                bind_group_layouts.clone(),
            )?);

            self.models.insert(m_key, model);
            log_info!("Cached model: {}", m.name);
        }

        Ok(())
    }
    pub fn load_asset(
        &mut self,
        surface_configuration: &wgpu::SurfaceConfiguration,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        asset: ModelAsset,
    ) -> Result<Arc<Model>, EngineError> {
        let m_key = CacheKey::from(asset.name.clone());
        if let Some(m) = self.models.get(&m_key) {
            return Ok(m.clone());
        }
        let model = Arc::new(Model::from_asset(
            &self.queue,
            &self.device,
            &mut self.materials,
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
    fn remove(&mut self, key: &crate::CacheKey) {
        self.models.remove(key);
    }
}
