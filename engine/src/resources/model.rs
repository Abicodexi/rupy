use super::{CacheKey, HashCache, MaterialAsset, MaterialManager, Mesh, MeshAsset, MeshInstance};
use crate::{
    fallback_diffuse, fallback_normal, log_debug, log_info, log_warning, AssetLoader,
    BindGroupManager, EngineError, PipelineManager, RenderBindGroupLayouts, ShaderManager,
    TextureManager, AABB,
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
        buffers: &[wgpu::VertexBufferLayout<'_>],
    ) -> Result<(MeshInstance, AABB), EngineError> {
        let (_mesh_asset, material_asset) = &self.asset;
        let material = if let Some(asset) = material_asset {
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
        primitive: wgpu::PrimitiveState,
        depth_stencil: Option<wgpu::DepthStencilState>,
        color_target: wgpu::ColorTargetState,
        bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
    ) -> Result<Model, EngineError> {
        let positions = model
            .mesh
            .positions
            .chunks(3)
            .map(|c| [c[0], c[1], c[2]])
            .collect::<Vec<_>>();
        let normals = model
            .mesh
            .normals
            .chunks(3)
            .map(|c| [c[0], c[1], c[2]])
            .collect::<Vec<_>>();
        let tex_coords = model
            .mesh
            .texcoords
            .chunks(2)
            .map(|c| [c[0], c[1]])
            .collect::<Vec<_>>();
        let colors = if !model.mesh.vertex_color.is_empty() {
            Some(
                model
                    .mesh
                    .vertex_color
                    .chunks(3)
                    .map(|c| [c[0], c[1], c[2]])
                    .collect::<Vec<_>>(),
            )
        } else {
            None
        };
        let vertices = MeshAsset::compute_vertex(
            &positions,
            &normals,
            &tex_coords,
            &model.mesh.indices,
            colors.as_deref(),
        );
        let indices = model.mesh.indices.clone();
        let model_asset = ModelAsset {
            name: model.name.clone(),
            asset: (MeshAsset { vertices, indices }, {
                if let Some(mat) = material {
                    let format = color_target.format.clone();
                    let diffuse_tex = mat.diffuse_texture.as_ref().cloned();
                    let normal_tex = mat.normal_texture.as_ref().cloned();
                    let mut mat_asset: MaterialAsset = mat.into();
                    mat_asset.primitive = primitive;
                    mat_asset.color_target = color_target;
                    mat_asset.depth_stencil = depth_stencil;
                    mat_asset.v_shader = v_shader.to_string();
                    mat_asset.f_shader = f_shader.to_string();
                    mat_asset.diffuse_texture = if let Some(dt) = diffuse_tex {
                        Some(
                            texture_manager
                                .get_or_load_texture(queue, device, &dt, format)
                                .unwrap_or(fallback_diffuse(queue, device, texture_manager))
                                .0,
                        )
                    } else {
                        Some(fallback_diffuse(queue, device, texture_manager).0)
                    };
                    mat_asset.normal_texture = if let Some(nt) = normal_tex {
                        Some(
                            texture_manager
                                .get_or_load_texture(queue, device, &nt, format)
                                .unwrap_or(fallback_normal(queue, device, texture_manager))
                                .0,
                        )
                    } else {
                        Some(fallback_normal(queue, device, texture_manager).0)
                    };

                    Some(mat_asset)
                } else {
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
            buffers,
        )?;
        Ok(Self {
            name: model.name.clone(),
            instance,
            aabb,
        })
    }
    pub fn from_gltf(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        material_manager: &mut MaterialManager,
        texture_manager: &mut TextureManager,
        shader_manager: &mut ShaderManager,
        pipeline_manager: &mut PipelineManager,
        bind_group_manager: &mut BindGroupManager,
        layouts: &RenderBindGroupLayouts,
        v_shader: &str,
        f_shader: &str,
        vertex_buffers: &[wgpu::VertexBufferLayout<'_>],
        format: wgpu::TextureFormat,
        primitive_state: wgpu::PrimitiveState,
        color_target: wgpu::ColorTargetState,
        depth_stencil: Option<wgpu::DepthStencilState>,
        bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
        path: &str,
    ) -> Result<Vec<Self>, EngineError> {
        let (doc, buffers, images) = gltf::import(path)?;
        let mut models = Vec::new();

        for (mesh_index, mesh) in doc.meshes().enumerate() {
            for (prim_index, primitive) in mesh.primitives().enumerate() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                let positions: Vec<[f32; 3]> = reader
                    .read_positions()
                    .ok_or_else(|| EngineError::AssetLoadError("Missing positions in glTF".into()))?
                    .collect();

                let normals: Vec<[f32; 3]> = match reader.read_normals() {
                    Some(iter) => iter.collect(),
                    None => vec![[0.0; 3]; positions.len()],
                };

                let texcoords: Vec<[f32; 2]> = reader
                    .read_tex_coords(0)
                    .map(|tc| tc.into_f32().collect())
                    .unwrap_or_else(|| vec![[0.0; 2]; positions.len()]);

                let indices: Vec<u32> = reader
                    .read_indices()
                    .map(|i| i.into_u32().collect())
                    .unwrap_or_else(|| (0..positions.len() as u32).collect());

                let vertices =
                    MeshAsset::compute_vertex(&positions, &normals, &texcoords, &indices, None);

                let mesh_asset = MeshAsset { vertices, indices };

                let material = primitive.material();
                let mat_name = format!("{}_{}_material", mesh_index, prim_index);
                log_info!(
                    "Base color tex is: {}",
                    material
                        .pbr_metallic_roughness()
                        .base_color_texture()
                        .is_some()
                );

                let base_color_texture = material
                    .pbr_metallic_roughness()
                    .base_color_texture()
                    .and_then(|info| {
                        let source = info.texture().source();
                        let image = &images[source.index()];
                        log_info!("BaseColor source idx: {}", source.index());
                        let label = format!("{}_{}_diffuse", mesh_index, prim_index);
                        texture_manager
                            .load_embedded_image(queue, device, format, image, &label)
                            .ok()
                    })
                    .or_else(|| {
                        material.emissive_texture().and_then(|info| {
                            let source = info.texture().source();
                            let image = &images[source.index()];
                            log_info!("Emissive used as diffuse, source idx: {}", source.index());
                            let label =
                                format!("{}_{}_emissive_as_diffuse", mesh_index, prim_index);
                            texture_manager
                                .load_embedded_image(queue, device, format, image, &label)
                                .ok()
                        })
                    });
                let normal_texture = material.normal_texture().and_then(|info| {
                    let source = info.texture().source();
                    log_info!("normal source idx: {}", source.index());
                    let image = &images[source.index()];
                    let label = format!("{}_{}_normal", mesh_index, prim_index);
                    texture_manager
                        .load_embedded_image(queue, device, format, image, &label)
                        .ok()
                });

                let base_color_factor = material.pbr_metallic_roughness().base_color_factor();
                let mat_asset = Some(MaterialAsset {
                    v_shader: v_shader.to_string(),
                    f_shader: f_shader.to_string(),
                    primitive: primitive_state,
                    color_target: color_target.clone(),
                    depth_stencil: depth_stencil.clone(),

                    name: mat_name.clone(),
                    key: CacheKey::from(mat_name),
                    ambient: [1.0, 1.0, 1.0],
                    diffuse: [
                        base_color_factor[0],
                        base_color_factor[1],
                        base_color_factor[2],
                    ],
                    specular: [1.0, 1.0, 1.0],
                    shininess: 32.0,
                    diffuse_texture: base_color_texture,
                    normal_texture,
                });

                let aabb = AABB::from_vertices(&mesh_asset.vertices);
                let model_asset = ModelAsset {
                    name: format!("{}_{}", mesh.name().unwrap_or("glb_mesh"), prim_index),
                    asset: (mesh_asset, mat_asset),
                    aabb,
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
                    bind_group_layouts.clone(),
                    vertex_buffers,
                )?;

                models.push(Self {
                    name: model_asset.name.clone(),
                    instance,
                    aabb,
                });
            }
        }

        Ok(models)
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
    pub async fn load_gltf(
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

        let loaded = Model::from_gltf(
            queue,
            device,
            material_manager,
            texture_manager,
            shader_manager,
            pipeline_manager,
            bind_group_manager,
            layouts,
            v_shader,
            f_shader,
            buffers,
            format,
            primitive,
            color_target.clone(),
            depth_stencil.clone(),
            bind_group_layouts.clone(),
            path.to_str().unwrap_or(file),
        )?;

        for model in loaded {
            let key = CacheKey::from(model.name.clone());
            log_info!("Loaded glTF model: {}", model.name);
            self.models.insert(key, Arc::new(model));
        }

        log_info!("Loaded glTF model: {}", file);
        Ok(())
    }
    pub async fn load_obj(
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
        primitive: wgpu::PrimitiveState,
        color_target: wgpu::ColorTargetState,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> Result<(), EngineError> {
        log_debug!("Loading object file: {}", file);
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
