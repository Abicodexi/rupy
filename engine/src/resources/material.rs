use super::{HashCache, Texture, TextureManager};
use crate::{
    fallback_diffuse, fallback_normal, gfx::{bind_group::{material_storage_group, material_storage_layout, normal_texture_layout, normal_textures_group, BindGroupManager}, buffer::WgpuBuffer, pipeline::PipelineManager}, log_debug, CacheKey, CacheStorage, EngineError, ShaderManager
};
use std::{collections::HashMap, sync::Arc};
use wgpu::BufferUsages;

#[derive(Clone, Debug)]
pub struct MaterialAsset {
    pub name: String,
    pub key: crate::CacheKey,
    pub v_shader: String,
    pub f_shader: String,
    pub ambient: [f32; 3],
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
    pub shininess: f32,
    pub diffuse_texture: Option<Arc<Texture>>,
    pub normal_texture: Option<Arc<Texture>>,
    pub primitive: wgpu::PrimitiveState,
    pub depth_stencil: Option<wgpu::DepthStencilState>,
    pub color_target: wgpu::ColorTargetState,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct MaterialData {
    pub ambient: [f32; 3],
    pub _pad0: f32,
    pub diffuse: [f32; 3],
    pub _pad1: f32,
    pub specular: [f32; 3],
    pub _pad2: f32,
    pub shininess: f32,
    pub _pad3: [f32; 3],
}
impl MaterialData {
    pub fn bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl From<&tobj::Material> for MaterialAsset {
    fn from(value: &tobj::Material) -> Self {
        Self {
            name: value.name.clone(),
            key: CacheKey::from(value.name.clone()),
            v_shader: "normal.vert.wgsl".to_string(),
            f_shader: "normal.frag.wgsl".to_string(),
            ambient: value.ambient.unwrap_or_default(),
            diffuse: value.diffuse.unwrap_or_default(),
            specular: value.specular.unwrap_or_default(),
            shininess: value.shininess.unwrap_or_default(),
            diffuse_texture: None,
            normal_texture: None,
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            color_target: wgpu::ColorTargetState {
                format: Texture::DEFAULT_FORMAT,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            },
        }
    }
}

impl From<tobj::Material> for MaterialAsset {
    fn from(value: tobj::Material) -> Self {
        Self {
            name: value.name.clone(),
            key: CacheKey::from(value.name.as_ref()),
            v_shader: "normal.vert.wgsl".to_string(),
            f_shader: "normal.frag.wgsl".to_string(),
            ambient: value.ambient.unwrap_or_default(),
            diffuse: value.diffuse.unwrap_or_default(),
            specular: value.specular.unwrap_or_default(),
            shininess: value.shininess.unwrap_or_default(),
            diffuse_texture: None,
            normal_texture: None,
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            color_target: wgpu::ColorTargetState {
                format: Texture::DEFAULT_FORMAT,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            },
        }
    }
}

impl MaterialAsset {
    pub fn data(&self) -> MaterialData {
        MaterialData {
            ambient: self.ambient,
            _pad0: 0.0,
            diffuse: self.diffuse,
            _pad1: 0.0,
            specular: self.specular,
            _pad2: 0.0,
            shininess: self.shininess,
            _pad3: [0.0; 3],
        }
    }
    pub fn build(
        &self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        textures: &mut TextureManager,
        shaders: &mut ShaderManager,
        pipelines: &mut PipelineManager,
        bind_groups: &mut BindGroupManager,
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        buffers: &[wgpu::VertexBufferLayout<'_>],
    ) -> Result<(Arc<wgpu::RenderPipeline>, Arc<wgpu::BindGroup>), EngineError> {
        let dt = self
            .diffuse_texture
            .clone()
            .unwrap_or_else(|| fallback_diffuse(queue, device, textures).0);
        let nt = self
            .normal_texture
            .clone()
            .unwrap_or_else(|| fallback_normal(queue, device, textures).0);

        let v_shader = shaders.load(device, &self.v_shader)?;
        let f_shader = shaders.load(device, &self.f_shader)?;

        let label = self.name.clone();
        let cache_key = CacheKey::from(label.clone());

        let bind_group = bind_groups
            .get_or_create(cache_key, || {
                Ok(normal_textures_group(device, &normal_texture_layout(device), &dt, &nt, label.as_ref()).into())
            })?
            .clone();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&self.name),
            bind_group_layouts: &bind_group_layouts
                .iter()
                .map(|bgl| bgl)
                .collect::<Vec<_>>(),
            push_constant_ranges: &[],
        });

        let pipeline_label = format!("{} render pipeline", label);

        let pipeline = pipelines
            .render
            .get_or_create(cache_key, || {
                Ok(Arc::new(device.create_render_pipeline(
                    &wgpu::RenderPipelineDescriptor {
                        label: Some(&pipeline_label),
                        layout: Some(&pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &v_shader,
                            entry_point: Some("vs_main"),
                            buffers,
                            compilation_options: Default::default(),
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &f_shader,
                            entry_point: Some("fs_main"),
                            targets: &[Some(self.color_target.clone())],
                            compilation_options: Default::default(),
                        }),
                        primitive: self.primitive,
                        depth_stencil: self.depth_stencil.clone(),
                        multisample: wgpu::MultisampleState {
                            count: 1,
                            mask: !0,
                            alpha_to_coverage_enabled: false,
                        },
                        multiview: None,
                        cache: None,
                    },
                )))
            })?
            .clone();

        Ok((pipeline, bind_group))
    }
}
#[derive(Debug)]
pub struct Material {
    pub asset: MaterialAsset,
    pub bind_group: Arc<wgpu::BindGroup>,
    pub pipeline: Arc<wgpu::RenderPipeline>,
    pub storage_id: Option<usize>,
}

impl Material {
    pub fn new(
        asset: MaterialAsset,
        bind_group: Arc<wgpu::BindGroup>,
        pipeline: Arc<wgpu::RenderPipeline>,
    ) -> Self {
        Self {
            asset,
            bind_group,
            pipeline,
            storage_id: None,
        }
    }
    pub fn from_asset(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        textures: &mut TextureManager,
        shaders: &mut ShaderManager,
        pipelines: &mut PipelineManager,
        bind_groups: &mut BindGroupManager,
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        asset: MaterialAsset,
    ) -> Result<Self, EngineError> {
        let (pipeline, bind_group) = asset.build(
            queue,
            device,
            textures,
            shaders,
            pipelines,
            bind_groups,
            bind_group_layouts,
            buffers,
        )?;

        Ok(Material {
            asset,
            pipeline,
            bind_group,
            storage_id: None,
        })
    }
    pub fn from_tobj<'a>(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        textures: &mut TextureManager,
        shaders: &mut ShaderManager,
        pipelines: &mut PipelineManager,
        bind_groups: &mut BindGroupManager,
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        mat: tobj::Material,
        v_shader: &'a str,
        f_shader: &'a str,
        primitive: wgpu::PrimitiveState,
        format: wgpu::TextureFormat,
        buffers: &'a [wgpu::VertexBufferLayout<'a>],
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> Result<Material, EngineError> {
        let diffuse_tex = mat.diffuse_texture.as_ref().cloned();
        let normal_tex = mat.normal_texture.as_ref().cloned();
        let mut asset: MaterialAsset = mat.into();
        asset.depth_stencil = depth_stencil.as_ref().cloned();
        asset.v_shader = v_shader.to_owned();
        asset.f_shader = f_shader.to_owned();
        asset.primitive = primitive;
        asset.color_target = wgpu::ColorTargetState {
            format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        };
        asset.diffuse_texture = if let Some(dt) = diffuse_tex {
            Some(
                textures
                    .get_or_load_texture(queue, device, &dt, format)
                    .unwrap_or(fallback_diffuse(queue, device, textures))
                    .0,
            )
        } else {
            Some(fallback_diffuse(queue, device, textures).0)
        };
        asset.normal_texture = if let Some(nt) = normal_tex {
            Some(
                textures
                    .get_or_load_texture(queue, device, &nt, format)
                    .unwrap_or(fallback_normal(queue, device, textures))
                    .0,
            )
        } else {
            Some(fallback_normal(queue, device, textures).0)
        };

        let material = Self::from_asset(
            queue,
            device,
            textures,
            shaders,
            pipelines,
            bind_groups,
            bind_group_layouts,
            buffers,
            asset,
        )?;

        Ok(material)
    }
}

pub struct MaterialStorage {
    buffer: Option<WgpuBuffer>,
    bind_group: Option<wgpu::BindGroup>,
    storage: HashMap<String, MaterialData>,
    rebuild: bool,
    count: usize,
}
impl MaterialStorage {
    pub fn new() -> Self {
        Self {
            buffer: None,
            bind_group: None,
            storage: HashMap::new(),
            rebuild: false,
            count: 0,
        }
    }
    pub fn bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.bind_group.as_ref()
    }
    pub fn count(&self) -> usize {
        self.count
    }
    pub fn id(&mut self) -> usize {
        let count = self.count;
        self.count += 1;
        count
    }
    fn build(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
    ) {
        if self.storage.is_empty() {
            return;
        }

        let data: Vec<MaterialData> = self.storage.values().cloned().collect();
        let recreate_bind_group = if self.buffer.is_none() {
            self.buffer = Some(WgpuBuffer::from_data(
                device,
                &data,
                BufferUsages::STORAGE | BufferUsages::COPY_DST,
                Some("batched material storage buffer"),
            ));
            true
        } else if self.rebuild {
            self.buffer
                .as_mut()
                .unwrap()
                .write_data(queue, device, &data, None);
            true
        } else {
            false
        };

        if recreate_bind_group {
            self.bind_group = Some(material_storage_group(
                device,
                &material_storage_layout(device),
                self.buffer.as_ref().unwrap(),
                Some("batched material storage buffer"),
            ));
        }

        self.rebuild = false;
        self.count = data.len();
        log_debug!("Storage rebuilt");
    }
    fn insert(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        material: &mut Material,
    ) {
        material.storage_id = Some(self.id());

        self.rebuild = self
            .storage
            .insert(material.asset.name.clone(), material.asset.data())
            .is_none();
        self.build(queue, device);
    }
}
impl Default for MaterialStorage {
    fn default() -> Self {
        Self::new()
    }
}
pub struct MaterialManager {
    materials: HashCache<Arc<Material>>,
    storage: MaterialStorage,
}

impl MaterialManager {
    pub fn new() -> Self {
        let storage = MaterialStorage::new();
        Self {
            materials: HashCache::new(),
            storage,
        }
    }
    pub fn storage(&self) -> &MaterialStorage {
        &self.storage
    }
    fn insert(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        mut material: Material,
    ) -> Arc<Material> {
        let key = material.asset.key;
        self.storage.insert(queue, device, &mut material);
        self.materials.insert(key, material.into());
        self.materials.get(&key).unwrap().clone()
    }

    pub fn load_tobj_sync<'a>(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        textures: &mut TextureManager,
        shaders: &mut ShaderManager,
        pipelines: &mut PipelineManager,
        bind_groups: &mut BindGroupManager,
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        mat: tobj::Material,
        v_shader: &'a str,
        f_shader: &'a str,
        primitive: wgpu::PrimitiveState,
        format: wgpu::TextureFormat,
        buffers: &'a [wgpu::VertexBufferLayout<'a>],
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> Result<Arc<Material>, EngineError> {
        if let Some(mat) = self.materials.get(&CacheKey::from(mat.name.to_string())) {
            return Ok(mat.clone());
        }
        let material = Material::from_tobj(
            queue,
            device,
            textures,
            shaders,
            pipelines,
            bind_groups,
            bind_group_layouts,
            mat,
            v_shader,
            f_shader,
            primitive,
            format,
            buffers,
            depth_stencil.clone(),
        )?;

        let material = self.insert(queue, device,  material);

        Ok(material)
    }
    pub async fn load_tobj_async<'a>(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        textures: &mut TextureManager,
        shaders: &mut ShaderManager,
        pipelines: &mut PipelineManager,
        bind_groups: &mut BindGroupManager,
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        mat: tobj::Material,
        v_shader: &'a str,
        f_shader: &'a str,
        primitive: wgpu::PrimitiveState,
        format: wgpu::TextureFormat,
        buffers: &'a [wgpu::VertexBufferLayout<'a>],
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> Result<Arc<Material>, EngineError> {
        if let Some(mat) = self.materials.get(&CacheKey::from(mat.name.to_string())) {
            return Ok(mat.clone());
        }
        let material = Material::from_tobj(
            queue,
            device,
            textures,
            shaders,
            pipelines,
            bind_groups,
            bind_group_layouts,
            mat,
            v_shader,
            f_shader,
            primitive.clone(),
            format,
            buffers,
            depth_stencil.clone(),
        )?;

        let material = self.insert(queue, device,  material);

        Ok(material)
    }
    pub fn load_asset<'a>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        textures: &mut TextureManager,
        shaders: &mut ShaderManager,
        pipelines: &mut PipelineManager,
        bind_groups: &mut BindGroupManager,
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        asset: MaterialAsset,
        buffers: &'a [wgpu::VertexBufferLayout<'a>],
    ) -> Result<Arc<Material>, EngineError> {
        if let Some(mat) = self.materials.get(&asset.key) {
            return Ok(mat.clone());
        }

        let (pipeline, bind_group) = asset.build(
            queue,
            device,
            textures,
            shaders,
            pipelines,
            bind_groups,
            bind_group_layouts,
            buffers,
        )?;

        let material = self.insert(
            queue,
            device,
            Material {
                asset,
                pipeline,
                bind_group,
                storage_id: None,
            },
        );

        Ok(material)
    }
    pub async fn load_asset_async<'a>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        textures: &mut TextureManager,
        shaders: &mut ShaderManager,
        pipelines: &mut PipelineManager,
        bind_groups: &mut BindGroupManager,
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        asset: MaterialAsset,
        buffers: &'a [wgpu::VertexBufferLayout<'a>],
    ) -> Result<Arc<Material>, EngineError> {
        if let Some(mat) = self.materials.get(&asset.key) {
            return Ok(mat.clone());
        }

        let (pipeline, bind_group) = asset.build(
            queue,
            device,
            textures,
            shaders,
            pipelines,
            bind_groups,
            bind_group_layouts,
            buffers,
        )?;

        let material = self.insert(
            queue,
            device,
            Material {
                asset,
                pipeline,
                bind_group,
                storage_id: None,
            },
        );

        Ok(material)
    }
}
impl Default for MaterialManager {
    fn default() -> Self {
        Self::new()
    }
}
impl crate::CacheStorage<std::sync::Arc<Material>> for MaterialManager {
    fn get_resource(&self, key: &crate::CacheKey) -> Option<&std::sync::Arc<Material>> {
        self.materials.get(key)
    }
    fn contains_resource(&self, key: &crate::CacheKey) -> bool {
        self.materials.contains_key(key)
    }
    fn get_mut(&mut self, key: &crate::CacheKey) -> Option<&mut std::sync::Arc<Material>> {
        self.materials.get_mut(key)
    }
    fn get_or_create<F>(
        &mut self,
        key: crate::CacheKey,
        create_fn: F,
    ) -> Result<Arc<Material>, EngineError>
    where
        F: FnOnce() -> Result<Arc<Material>, EngineError>,
    {
        self.materials.get_or_create(key, create_fn)
    }
    fn insert_resource(&mut self, key: crate::CacheKey, resource: std::sync::Arc<Material>) {
        self.materials.insert(key, resource);
    }
    fn remove_resource(&mut self, key: &crate::CacheKey) -> Option<std::sync::Arc<Material>> {
        self.materials.remove(key)
    }

    fn all<'a>(&'a self) -> impl Iterator<Item = &'a std::sync::Arc<Material>>
    where
        std::sync::Arc<Material>: 'a,
    {
        self.materials.values()
    }
}
