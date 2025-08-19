use super::{CacheKey, HashCache, MaterialAsset, MaterialManager, Mesh, MeshAsset, MeshInstance};
use crate::{
    fallback_diffuse, fallback_normal, gfx::{bind_group::BindGroupManager, pipeline::PipelineManager}, log_debug, log_info, log_warning, AssetLoader, EngineError, ShaderManager, TextureManager, AABB
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
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
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
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
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
        model: &tobj::Model,
        material: Option<&tobj::Material>,
        v_shader: &str,
        f_shader: &str,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        primitive: wgpu::PrimitiveState,
        depth_stencil: Option<wgpu::DepthStencilState>,
        format: wgpu::TextureFormat,
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
    ) -> Result<Model, EngineError> {
        let mesh = &model.mesh;

        let positions: Vec<_> = mesh
            .positions
            .chunks(3)
            .map(|c| [c[0], c[1], c[2]])
            .collect();
        let normals: Vec<_> = mesh.normals.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();
        let tex_coords: Vec<_> = mesh.texcoords.chunks(2).map(|c| [c[0], c[1]]).collect();
        let colors = (!mesh.vertex_color.is_empty()).then(|| {
            mesh.vertex_color
                .chunks(3)
                .map(|c| [c[0], c[1], c[2]])
                .collect::<Vec<_>>()
        });

        let indices: Vec<u16> = mesh.indices.iter().map(|&i| i as u16).collect();

        let vertices = MeshAsset::compute_vertex(
            &positions,
            &normals,
            &tex_coords,
            &indices,
            colors.as_deref(),
        );

        let mat_asset = material.map(|mat| {
            let mut asset: MaterialAsset = mat.into();
            asset.primitive = primitive;
            asset.color_target = wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            };
            asset.depth_stencil = depth_stencil;
            asset.v_shader = v_shader.to_string();
            asset.f_shader = f_shader.to_string();

            asset.diffuse_texture = mat
                .diffuse_texture
                .as_deref()
                .and_then(|dt| {
                    texture_manager
                        .get_or_load_texture(queue, device, dt, format)
                        .ok()
                })
                .map(|t| t.0)
                .or_else(|| Some(fallback_diffuse(queue, device, texture_manager).0));

            asset.normal_texture = mat
                .normal_texture
                .as_deref()
                .and_then(|nt| {
                    texture_manager
                        .get_or_load_texture(queue, device, nt, format)
                        .ok()
                })
                .map(|t| t.0)
                .or_else(|| Some(fallback_normal(queue, device, texture_manager).0));

            asset
        });

        let model_asset = ModelAsset {
            name: model.name.clone(),
            asset: (MeshAsset { vertices, indices }, mat_asset),
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
            bind_group_layouts,
            buffers,
        )?;

        Ok(Model {
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
        v_shader: &str,
        f_shader: &str,
        vertex_buffers: &[wgpu::VertexBufferLayout<'_>],
        primitive_state: wgpu::PrimitiveState,
        format: wgpu::TextureFormat,
        depth_stencil: Option<wgpu::DepthStencilState>,
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
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

                let indices: Vec<u16> = reader
                    .read_indices()
                    .map(|i| i.into_u32().map(|i| i as u16).collect())
                    .unwrap_or_else(|| (0..positions.len() as u16).collect());

                let vertices =
                    MeshAsset::compute_vertex(&positions, &normals, &texcoords, &indices, None);

                let mesh_asset = MeshAsset { vertices, indices };

                let material = primitive.material();
                let mat_name = format!("{}_{}_material", mesh_index, prim_index);

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
                    color_target: wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    },

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
        file: &str,
        v_shader: &str,
        f_shader: &str,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        primitive: wgpu::PrimitiveState,
        format: wgpu::TextureFormat,
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
            v_shader,
            f_shader,
            buffers,
            primitive,
            format,
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
        file: &str,
        v_shader: &str,
        f_shader: &str,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        primitive: wgpu::PrimitiveState,
        color_target: wgpu::TextureFormat,
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
                &m,
                mat,
                v_shader,
                f_shader,
                buffers,
                primitive,
                depth_stencil.clone(),
                color_target,
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
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
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
