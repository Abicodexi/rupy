use crate::{
    log_debug, log_info, log_warning, CacheKey, CacheStorage, EngineError, PipelineManager, Shader,
    ShaderManager, WgpuBuffer,
};
use std::{collections::HashMap, sync::Arc};
use wgpu::BufferUsages;

use super::{BindGroup, HashCache, Texture, TextureManager};

#[derive(Clone, Debug)]
pub struct MaterialAsset {
    pub name: String,
    pub key: crate::CacheKey,
    pub shader: String,
    pub ambient: [f32; 3],
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
    pub shininess: f32,
    pub diffuse_texture: Option<String>,
    pub normal_texture: Option<String>,
    pub primitive: wgpu::PrimitiveState,
    pub depth_stencil: Option<wgpu::DepthStencilState>,
    pub color_target: wgpu::ColorTargetState,
    pub bind_group_layouts: Vec<wgpu::BindGroupLayout>,
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

impl From<tobj::Material> for MaterialAsset {
    fn from(value: tobj::Material) -> Self {
        Self {
            name: value.name.clone(),
            key: CacheKey::from(value.name),
            shader: Shader::DEFAULT.to_string(),
            ambient: value.ambient.unwrap_or_default(),
            diffuse: value.diffuse.unwrap_or_default(),
            specular: value.specular.unwrap_or_default(),
            shininess: value.shininess.unwrap_or_default(),
            diffuse_texture: value.diffuse_texture,
            normal_texture: value.normal_texture,
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            color_target: wgpu::ColorTargetState {
                format: Texture::DEFAULT_FORMAT,
                blend: None,
                write_mask: wgpu::ColorWrites::default(),
            },
            bind_group_layouts: Vec::new(),
        }
    }
}

impl From<&tobj::Material> for MaterialAsset {
    fn from(value: &tobj::Material) -> Self {
        Self {
            name: value.name.clone(),
            key: CacheKey::from(value.name.clone()),
            shader: Shader::DEFAULT.to_string(),
            ambient: value.ambient.unwrap_or_default(),
            diffuse: value.diffuse.unwrap_or_default(),
            specular: value.specular.unwrap_or_default(),
            shininess: value.shininess.unwrap_or_default(),
            diffuse_texture: value.diffuse_texture.clone(),
            normal_texture: value.normal_texture.clone(),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            color_target: wgpu::ColorTargetState {
                format: Texture::DEFAULT_FORMAT,
                blend: None,
                write_mask: wgpu::ColorWrites::default(),
            },
            bind_group_layouts: Vec::new(),
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
    pub fn buffer(&self, queue: &wgpu::Queue, device: &wgpu::Device, idx: u64) -> WgpuBuffer {
        let binding = [self.data()];
        let data = bytemuck::cast_slice(&binding);
        let material_buffer = WgpuBuffer::from_data(
            device,
            &data, // &[u8]
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            Some(&format!("{} material storage buffer", self.key.id())),
        );
        queue.write_buffer(material_buffer.get(), idx, &data);
        material_buffer
    }
    fn fallback_diffuse(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        textures: &mut crate::TextureManager,
    ) -> (Arc<crate::Texture>, crate::CacheKey) {
        let white_pixel = [255u8, 255, 255, 255];

        let diffuse_cache_key = CacheKey::from("fallback_diffuse_texture");
        if let Some(cached_diffuse_fallback) = textures.get(diffuse_cache_key) {
            (cached_diffuse_fallback.clone(), diffuse_cache_key)
        } else {
            let diffuse = crate::Texture::from_desc(
                device,
                &wgpu::TextureDescriptor {
                    label: Some("diffuse_fallback_texture"),
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
            let texture_arc = Arc::new(diffuse);
            textures.insert(diffuse_cache_key, texture_arc.clone());
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture_arc.texture,
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
            (texture_arc, diffuse_cache_key)
        }
    }
    fn fallback_normal(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        textures: &mut crate::TextureManager,
    ) -> (Arc<crate::Texture>, crate::CacheKey) {
        let flat_normal = [128u8, 128, 255, 255];

        let normal_cache_key = CacheKey::from("fallback_normal_texture");
        if let Some(cached_normal_fallback) = textures.get(normal_cache_key) {
            (cached_normal_fallback.clone(), normal_cache_key)
        } else {
            let normal = crate::Texture::from_desc(
                device,
                &wgpu::TextureDescriptor {
                    label: Some("normal_fallback_texture"),
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
            let texture_arc = Arc::new(normal);
            textures.insert(normal_cache_key, texture_arc.clone());
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture_arc.texture,
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
            (texture_arc, normal_cache_key)
        }
    }
    pub fn load_asset(
        &self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        textures: &mut TextureManager,
        shaders: &mut ShaderManager,
        pipelines: &mut PipelineManager,
        surface_configuration: &wgpu::SurfaceConfiguration,
        buffers: &[wgpu::VertexBufferLayout<'_>],
    ) -> Result<
        (
            std::sync::Arc<wgpu::RenderPipeline>,
            std::sync::Arc<wgpu::BindGroup>,
        ),
        crate::EngineError,
    > {
        let (dt, ..) = self
            .diffuse_texture
            .as_ref()
            .map(|p| textures.get_or_load_texture(queue, device, p, surface_configuration))
            .unwrap_or_else(|| Ok(Self::fallback_diffuse(queue, device, textures)))?;
        let (nt, ..) = self
            .normal_texture
            .as_ref()
            .map(|p| textures.get_or_load_texture(queue, device, p, surface_configuration))
            .unwrap_or_else(|| Ok(Self::fallback_normal(queue, device, textures)))?;

        log_info!("DT: {}", dt.label);
        log_info!("NT: {}", nt.label);
        let shader = shaders.load(device, &self.shader)?;
        let bgl_refs: Vec<&wgpu::BindGroupLayout> = self.bind_group_layouts.iter().collect();

        let bind_group = Arc::new(crate::BindGroup::normal(
            device,
            &dt,
            &nt,
            format!("{}_texture_binding", &self.name).as_ref(),
        ));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&self.name),
            bind_group_layouts: &bgl_refs,
            push_constant_ranges: &[],
        });

        let pipeline_label = format!("{}_{}", self.name, self.shader);
        let pipeline_cache_key = crate::CacheKey::from(pipeline_label.clone());

        let pipeline = pipelines
            .render
            .get_or_create(pipeline_cache_key, || {
                Arc::new(
                    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some(&pipeline_label),
                        layout: Some(&pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &shader,
                            entry_point: Some("vs_main"),
                            buffers,
                            compilation_options: Default::default(),
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader,
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
                    }),
                )
            })
            .clone();

        Ok((pipeline, bind_group))
    }
}
#[derive(Debug)]
pub struct Material {
    pub asset: MaterialAsset,
    pub bind_group: Arc<wgpu::BindGroup>,
    pub pipeline: Arc<wgpu::RenderPipeline>,
    pub idx: u32,
}

impl Material {
    pub fn from_asset(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        textures: &mut TextureManager,
        shaders: &mut ShaderManager,
        pipelines: &mut PipelineManager,
        surface_configuration: &wgpu::SurfaceConfiguration,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        asset: MaterialAsset,
        idx: u32,
    ) -> Result<Self, EngineError> {
        let (pipeline, bind_group) = asset.load_asset(
            queue,
            device,
            textures,
            shaders,
            pipelines,
            surface_configuration,
            buffers,
        )?;

        let material = Material {
            asset: asset.clone(),
            pipeline,
            bind_group,
            idx,
        };
        Ok(material)
    }
    pub fn from_tobj<'a>(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        textures: &mut TextureManager,
        shaders: &mut ShaderManager,
        pipelines: &mut PipelineManager,
        mat: &tobj::Material,
        idx: u32,
        shader_path: &'a str,
        primitive: wgpu::PrimitiveState,
        color_target: wgpu::ColorTargetState,
        surface_configuration: &wgpu::SurfaceConfiguration,
        buffers: &'a [wgpu::VertexBufferLayout<'a>],
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> Result<Material, EngineError> {
        let mut asset: MaterialAsset = mat.into();
        asset.depth_stencil = depth_stencil.as_ref().cloned();
        asset.shader = shader_path.to_owned();
        asset.primitive = primitive;
        asset.color_target = color_target;
        asset.bind_group_layouts = bind_group_layouts;

        let material = Self::from_asset(
            queue,
            device,
            textures,
            shaders,
            pipelines,
            surface_configuration,
            buffers,
            asset,
            idx,
        )?;

        Ok(material)
    }
    pub fn from_tobj_vec<'a>(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        textures: &mut TextureManager,
        shaders: &mut ShaderManager,
        pipelines: &mut PipelineManager,
        mats: &[tobj::Material],
        base_idx: u32,
        shader: &'a str,
        primitive: wgpu::PrimitiveState,
        color_target: wgpu::ColorTargetState,
        surface_configuration: &wgpu::SurfaceConfiguration,
        buffers: &'a [wgpu::VertexBufferLayout<'a>],
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> Vec<Material> {
        let mut idx = base_idx;
        mats.iter()
            .filter_map(|m| {
                match Self::from_tobj(
                    queue,
                    device,
                    textures,
                    shaders,
                    pipelines,
                    m,
                    idx,
                    shader,
                    primitive,
                    color_target.clone(),
                    surface_configuration,
                    buffers,
                    bind_group_layouts.clone(),
                    depth_stencil.clone(),
                ) {
                    Ok(mat) => {
                        idx += 1;
                        Some(mat)
                    }
                    Err(e) => {
                        log_warning!("{}: {:?}", m.name, e);
                        None
                    }
                }
            })
            .collect()
    }
}

pub struct MaterialManager {
    pub textures: TextureManager,
    pub pipelines: PipelineManager,
    pub shaders: ShaderManager,
    pub materials: HashCache<Arc<Material>>,
    pub storage_buffer: WgpuBuffer,
    pub storage_bind_group: wgpu::BindGroup,
    pub storage: HashMap<String, MaterialData>,
    pub storage_rebuild: bool,
    pub storage_count: usize,
}

impl MaterialManager {
    pub fn new(device: &wgpu::Device) -> Self {
        let mat_data = [MaterialData::default()];
        let data: &[u8] = bytemuck::cast_slice(&mat_data);
        let storage_buffer = WgpuBuffer::from_data(
            device,
            data,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            Some(&format!("batched material storage buffer")),
        );
        let storage_bind_group = BindGroup::material_storage(
            device,
            &storage_buffer,
            Some(&format!("batched material storage buffer")),
        );
        Self {
            textures: TextureManager::new(),
            pipelines: PipelineManager::new(),
            shaders: ShaderManager::new(),
            materials: HashCache::new(),
            storage_buffer,
            storage_bind_group,
            storage: HashMap::new(),
            storage_rebuild: false,
            storage_count: 0,
        }
    }
    pub fn toggle_rebuild(&mut self) {
        self.storage_rebuild = true;
    }
    pub fn build_storage(&mut self, device: &wgpu::Device) {
        let label = "storage buffer";
        let usage = BufferUsages::STORAGE | BufferUsages::COPY_DST;
        let data: Vec<MaterialData> = self.storage.values().map(|m| m.clone()).collect();
        if !data.is_empty() {
            log_debug!("Building storage");
            let storage = WgpuBuffer::from_data(device, &data, usage, Some(label));
            let binding = BindGroup::material_storage(device, &storage, Some(label));
            self.storage_bind_group = binding;
            self.storage_buffer = storage;
            self.storage_rebuild = false;
            self.storage_count = data.len();
        }
    }
    pub fn update_storage(&mut self, material: &Material) {
        log_info!("Mat idx count on creation: {}", self.storage.len());
        self.storage_rebuild = self
            .storage
            .insert(material.asset.name.clone(), material.asset.data())
            .is_none();

        log_debug!("Should rebuild storage: {}", self.storage_rebuild);
    }

    pub fn load_tobj<'a>(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        textures: &mut TextureManager,
        shaders: &mut ShaderManager,
        pipelines: &mut PipelineManager,
        mat: &tobj::Material,
        mat_id: usize,
        shader_path: &'a str,
        primitive: wgpu::PrimitiveState,
        color_target: wgpu::ColorTargetState,
        surface_configuration: &wgpu::SurfaceConfiguration,
        buffers: &'a [wgpu::VertexBufferLayout<'a>],
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> Result<Arc<Material>, EngineError> {
        let m_key = CacheKey::from(format!("{}{}", mat.name.clone(), mat_id));
        if let Some(mat) = self.materials.get(&m_key) {
            return Ok(mat.clone());
        }
        let idx = (self.materials.len() + 1) as u32;
        let material = Material::from_tobj(
            queue,
            device,
            textures,
            shaders,
            pipelines,
            mat,
            idx,
            shader_path,
            primitive,
            color_target,
            surface_configuration,
            buffers,
            bind_group_layouts,
            depth_stencil,
        )?;

        let material = Arc::new(material);
        self.materials.insert(m_key, material.clone());
        self.toggle_rebuild();

        Ok(material)
    }
    pub fn load_tobj_vec<'a>(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        textures: &mut TextureManager,
        shaders: &mut ShaderManager,
        pipelines: &mut PipelineManager,
        mats: Vec<&tobj::Material>,
        shader_path: &'a str,
        primitive: wgpu::PrimitiveState,
        color_target: wgpu::ColorTargetState,
        surface_configuration: &wgpu::SurfaceConfiguration,
        buffers: &'a [wgpu::VertexBufferLayout<'a>],
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> Result<Vec<Arc<Material>>, EngineError> {
        let mut materials = Vec::new();
        let mut idx = (self.materials.len() + 1) as u32;
        for m in mats {
            let m_key = CacheKey::from(m.name.clone());
            if let Some(mat) = self.materials.get(&m_key) {
                materials.push(mat.clone());
                idx += 1;
                continue;
            }
            let material = Material::from_tobj(
                queue,
                device,
                textures,
                shaders,
                pipelines,
                m,
                idx,
                shader_path,
                primitive,
                color_target.clone(),
                surface_configuration,
                buffers,
                bind_group_layouts.clone(),
                depth_stencil.clone(),
            )?;

            let material = Arc::new(material);
            self.materials.insert(m_key, material.clone());
            materials.push(material);
            idx += 1;
        }
        self.toggle_rebuild();
        Ok(materials)
    }
    pub fn load_asset<'a>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        asset: crate::MaterialAsset,
        surface_configuration: &wgpu::SurfaceConfiguration,
        buffers: &'a [wgpu::VertexBufferLayout<'a>],
    ) -> Result<Arc<Material>, EngineError> {
        if let Some(mat) = self.materials.get(&asset.key) {
            return Ok(mat.clone());
        }

        let (pipeline, bind_group) = asset.load_asset(
            queue,
            device,
            &mut self.textures,
            &mut self.shaders,
            &mut self.pipelines,
            surface_configuration,
            buffers,
        )?;
        let idx = self.materials.len() as u32;
        let material = Arc::new(Material {
            asset: asset.clone(),
            pipeline,
            bind_group,
            idx,
        });
        self.materials.insert(asset.key.clone(), material.clone());
        self.toggle_rebuild();

        Ok(material)
    }
}
