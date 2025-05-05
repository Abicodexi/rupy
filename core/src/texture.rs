use crate::assets::loader::AssetLoader;
use crate::{BindGroupLayouts, CacheKey, CacheStorage, EngineError, GpuContext, HashCache};
use image::codecs::hdr::{HdrDecoder, HdrMetadata};
use std::io::Cursor;
use std::sync::Arc;
/// A GPU-ready texture: the texture itself, a view, and a sampler.
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub label: String,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    pub async fn from_desc(device: &wgpu::Device, desc: &wgpu::TextureDescriptor<'_>) -> Self {
        let texture = device.create_texture(desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: desc.label,
            dimension: match desc.size.depth_or_array_layers {
                6 => Some(wgpu::TextureViewDimension::Cube),
                _ => Some(wgpu::TextureViewDimension::D2),
            },
            array_layer_count: if desc.size.depth_or_array_layers == 6 {
                Some(6)
            } else {
                None
            },
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: desc.label,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
            label: desc.label.map(|l| l.to_string()).unwrap_or_default(),
        }
    }

    pub async fn from_bytes<P: AsRef<std::path::Path>>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: P,
    ) -> Result<Self, EngineError> {
        let img = image::load_from_memory(bytes)?;
        let rgba = img.to_rgba8();
        let (width, height) = image::GenericImageView::dimensions(&img);

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label.as_ref().to_str().unwrap()),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("default_sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
            label: label.as_ref().to_string_lossy().into_owned(),
        })
    }

    pub fn create(
        device: &wgpu::Device,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        mip_level_count: u32,
        view_dim: wgpu::TextureViewDimension,
        usage: wgpu::TextureUsages,
        address_mode: Option<wgpu::AddressMode>,
        mag_filter: wgpu::FilterMode,
        sampler: Option<wgpu::Sampler>,
        label: Option<&str>,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label,
            dimension: Some(view_dim),
            array_layer_count: if view_dim == wgpu::TextureViewDimension::Cube {
                Some(6)
            } else {
                None
            },
            ..Default::default()
        });

        let sampler = sampler.unwrap_or(device.create_sampler(&wgpu::SamplerDescriptor {
            label,
            address_mode_u: address_mode.unwrap_or(wgpu::AddressMode::Repeat),
            address_mode_v: address_mode.unwrap_or(wgpu::AddressMode::Repeat),
            address_mode_w: address_mode.unwrap_or(wgpu::AddressMode::ClampToEdge),
            mag_filter,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        }));

        Self {
            texture,
            view,
            sampler,
            label: label.unwrap_or("").to_string(),
        }
    }

    pub fn create_projection_view(&self) -> wgpu::TextureView {
        self.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Cubemap projection view"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        })
    }
}

impl Into<CacheKey> for Texture {
    fn into(self) -> CacheKey {
        CacheKey::new(self.label)
    }
}

pub struct TextureManager {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    textures: HashCache<Arc<Texture>>,
    texture_bgs: std::collections::HashMap<String, wgpu::BindGroup>,
    pub depth_texture: Texture,
    pub depth_stencil_state: wgpu::DepthStencilState,
}

impl CacheStorage<Arc<Texture>> for TextureManager {
    fn get<K: Into<CacheKey>>(&self, key: K) -> Option<&Arc<Texture>> {
        self.textures.get(&key.into())
    }

    fn contains(&self, key: &CacheKey) -> bool {
        self.textures.contains_key(key)
    }
    fn get_mut(&mut self, key: &CacheKey) -> Option<&mut Arc<Texture>> {
        self.textures.get_mut(key)
    }
    fn get_or_create<F>(&mut self, key: CacheKey, create_fn: F) -> &mut Arc<Texture>
    where
        F: FnOnce() -> Arc<Texture>,
    {
        self.textures.entry(key).or_insert_with(create_fn)
    }
    fn insert(&mut self, key: CacheKey, resource: Arc<Texture>) {
        self.textures.insert(key, resource);
    }
    fn remove(&mut self, key: &CacheKey) {
        self.textures.remove(key);
    }
}

impl TextureManager {
    pub fn new(
        resources: &std::sync::Arc<GpuContext>,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let depth_stencil_state = wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };

        let depth_texture = Texture::create(
            &resources.device,
            wgpu::Extent3d {
                width: surface_config.width,
                height: surface_config.height,
                depth_or_array_layers: 1,
            },
            Texture::DEPTH_FORMAT,
            1,
            wgpu::TextureViewDimension::D2,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            Some(wgpu::AddressMode::ClampToEdge),
            wgpu::FilterMode::Linear,
            Some(resources.device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual),
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            })),
            Some("Depth texture"),
        );
        Self {
            device: resources.device.clone(),
            queue: resources.queue.clone(),
            textures: HashCache::new(),
            texture_bgs: std::collections::HashMap::new(),
            depth_texture,
            depth_stencil_state,
        }
    }
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
    /// Load (or reload) a texture from disk
    pub async fn load<K: Into<CacheKey>>(
        &mut self,
        key: K,
        asset_loader: &AssetLoader,
        rel_path: &str,
    ) -> Result<Arc<Texture>, EngineError> {
        let tex = asset_loader.load_texture(&self.queue, rel_path).await?;

        let arc = Arc::new(tex);
        self.textures.insert(key.into(), arc.clone());
        Ok(arc)
    }

    /// Retrieve a previously loaded texture
    pub fn get<K: Into<CacheKey>>(&self, key: K) -> Option<Arc<Texture>> {
        self.textures.get(&key.into()).cloned()
    }

    /// Unload a texture from the manager (will free when Arc drops)
    pub fn unload<K: Into<CacheKey>>(&mut self, key: K) {
        self.textures.remove(&key.into());
    }

    pub fn bind_group_for(
        &mut self,
        key: &str,
        layout: &wgpu::BindGroupLayout,
    ) -> Option<&wgpu::BindGroup> {
        if !self.texture_bgs.contains_key(key) {
            let tex = self.get(key)?;
            let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("tex_bg:{}", key)),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&tex.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&tex.sampler),
                    },
                ],
            });
            self.texture_bgs.insert(key.to_string(), bg);
        }
        self.texture_bgs.get(key)
    }
    pub fn decode_hdr(data: &[u8]) -> Result<(Vec<[f32; 4]>, HdrMetadata), EngineError> {
        let decoder = HdrDecoder::new(Cursor::new(data))?;
        let meta = decoder.metadata();
        let mut pixels = vec![[0.0; 4]; (meta.width * meta.height) as usize];

        decoder.read_image_transform(
            |pix| {
                let rgb = pix.to_hdr();
                [rgb.0[0], rgb.0[1], rgb.0[2], 1.0]
            },
            &mut pixels[..],
        )?;

        Ok((pixels, meta))
    }

    pub fn insert_texture_bind_group(&mut self, key: &CacheKey, bind_group: wgpu::BindGroup) {
        self.texture_bgs.insert(key.id.clone(), bind_group);
    }
    pub fn prepare_equirect_projection_textures(
        &mut self,
        asset_loader: &AssetLoader,
        bind_group_layouts: &BindGroupLayouts,
        rel_path: &str,
        dst_size: u32,
        format: wgpu::TextureFormat,
    ) -> Result<(CacheKey, wgpu::BindGroup, CacheKey, wgpu::BindGroup), EngineError> {
        let path = asset_loader.resolve(&format!("hdr\\{}", rel_path));
        let bytes = AssetLoader::read_bytes(&path)?;
        let (pixels, meta) = Self::decode_hdr(&bytes)?;

        let src_key = CacheKey::new("equirect_projection_src");
        let src = Texture::create(
            &self.device,
            wgpu::Extent3d {
                width: meta.width,
                height: meta.height,
                depth_or_array_layers: 1,
            },
            format,
            1,
            wgpu::TextureViewDimension::D2,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            None,
            wgpu::FilterMode::Linear,
            None,
            Some(&format!("src:{}", rel_path)),
        );

        let dst_key = CacheKey::new("equirect_projection_dst");
        let dst = Texture::create(
            &self.device,
            wgpu::Extent3d {
                width: dst_size,
                height: dst_size,
                depth_or_array_layers: 6,
            },
            format,
            1,
            wgpu::TextureViewDimension::Cube,
            wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            Some(wgpu::AddressMode::ClampToEdge),
            wgpu::FilterMode::Nearest,
            None,
            Some(&format!("dst:{}", rel_path)),
        );

        let equirect_src_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layouts.equirect_src,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&dst.create_projection_view()),
                },
            ],
            label: Some("Equirect projection bind group"),
        });

        let equirect_dst_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layouts.equirect_dst,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&dst.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&dst.sampler),
                },
            ],
            label: Some("Skybox bind group"),
        });

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &src.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&pixels),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(meta.width * std::mem::size_of::<[f32; 4]>() as u32),
                rows_per_image: Some(meta.height),
            },
            src.texture.size(),
        );

        self.texture_bgs
            .insert(src_key.id.clone(), equirect_src_bind_group.clone());
        self.texture_bgs
            .insert(dst_key.id.clone(), equirect_dst_bind_group.clone());

        self.textures.insert(src_key.clone(), src.into());
        self.textures.insert(dst_key.clone(), dst.into());

        Ok((
            src_key,
            equirect_src_bind_group,
            dst_key,
            equirect_dst_bind_group,
        ))
    }
}
