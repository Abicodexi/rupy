use crate::{CacheKey, CacheStorage, EngineError, HashCache};
use image::codecs::hdr::{HdrDecoder, HdrMetadata};
use std::io::Cursor;
use std::sync::Arc;

/// A GPU-ready texture: the texture itself, a view, and a sampler.
#[derive(Debug)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub label: String,
}

impl Texture {
    pub const DEFAULT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    pub const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;

    pub const PROJECTION: [wgpu::BindingType; 2] = [
        wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        wgpu::BindingType::StorageTexture {
            access: wgpu::StorageTextureAccess::WriteOnly,
            format: wgpu::TextureFormat::Rgba32Float,
            view_dimension: wgpu::TextureViewDimension::D2Array,
        },
    ];
    pub const TEXTURE_D2_BINDING: wgpu::BindingType = wgpu::BindingType::Texture {
        multisampled: false,
        view_dimension: wgpu::TextureViewDimension::D2,
        sample_type: wgpu::TextureSampleType::Float { filterable: true },
    };
    pub const SAMPLER_FILTERING_BINDING: wgpu::BindingType =
        wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering);

    pub fn create_view(&self, desc: &wgpu::TextureViewDescriptor) -> wgpu::TextureView {
        self.texture.create_view(desc)
    }
    pub fn from_desc(device: &wgpu::Device, desc: &wgpu::TextureDescriptor<'_>) -> Self {
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
    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        img: &image::RgbaImage,
        label: impl Into<String>,
    ) -> Texture {
        let label: String = label.into();
        let (width, height) = img.dimensions();
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let bytes_per_row =
            std::num::NonZeroU32::new(4 * width).expect("Bytes per row NonZeroU32 unwrap");
        let rows_per_image =
            std::num::NonZeroU32::new(height).expect("Rows per image NonZeroU32 unwrap");

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &img, // &[u8]
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row.into()),
                rows_per_image: Some(rows_per_image.into()),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Texture {
            texture,
            view,
            sampler,
            label,
        }
    }
    pub async fn from_file(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        file_name: &str,
    ) -> Result<Texture, EngineError> {
        let path = &format!("textures/{}", file_name);
        let file_bytes = crate::Asset::read_bytes(path)?;
        let texture = Self::from_bytes(device, queue, &file_bytes, path).await?;
        Ok(texture)
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

    pub fn new(
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
}

impl Into<CacheKey> for Texture {
    fn into(self) -> CacheKey {
        CacheKey::new(crate::CacheKey::hash(self.label))
    }
}

pub struct TextureManager {
    textures: HashCache<Arc<Texture>>,
}
impl TextureManager {
    pub fn get_or_load_texture(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        texture: &str,
        format: wgpu::TextureFormat,
    ) -> Result<(Arc<Texture>, CacheKey), EngineError> {
        let base_dir = crate::asset_dir()?.join("textures");
        let cache_key = CacheKey::from(texture.to_string());
        if let Some(tex) = self.get(&cache_key) {
            Ok((tex.clone(), cache_key))
        } else {
            let img = image::open(base_dir.join(texture))
                .map_err(|e| EngineError::AssetLoadError(e.to_string()))?
                .to_rgba8();
            let tex = Texture::from_image(device, queue, format, &img, texture);
            let arc = Arc::new(tex);
            self.insert(cache_key.clone(), arc.clone());
            Ok((arc, cache_key))
        }
    }
}
impl CacheStorage<Arc<Texture>> for TextureManager {
    fn get(&self, key: &CacheKey) -> Option<&Arc<Texture>> {
        self.textures.get(key)
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
    fn remove(&mut self, key: &CacheKey) -> Option<std::sync::Arc<Texture>> {
        self.textures.remove(key)
    }
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            textures: HashCache::new(),
        }
    }

    /// Unload a texture from the manager (will free when Arc drops)
    pub fn unload<K: Into<CacheKey>>(&mut self, key: K) {
        self.textures.remove(&key.into());
    }
}

pub fn fallback_diffuse(
    queue: &wgpu::Queue,
    device: &wgpu::Device,
    textures: &mut crate::TextureManager,
) -> (Arc<crate::Texture>, crate::CacheKey) {
    let white_pixel = [255u8, 255, 255, 255];

    let diffuse_cache_key = CacheKey::from("fallback_diffuse_texture");
    if let Some(cached_diffuse_fallback) = textures.get(&diffuse_cache_key) {
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
pub fn fallback_normal(
    queue: &wgpu::Queue,
    device: &wgpu::Device,
    textures: &mut crate::TextureManager,
) -> (Arc<crate::Texture>, crate::CacheKey) {
    let flat_normal = [128u8, 128, 255, 255];

    let normal_cache_key = CacheKey::from("fallback_normal_texture");
    if let Some(cached_normal_fallback) = textures.get(&normal_cache_key) {
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
