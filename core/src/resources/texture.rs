use crate::{CacheKey, CacheStorage, EngineError, HashCache};
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
    pub const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;

    pub const D2: [super::BindGroupBindingType; 2] = [
        super::BindGroupBindingType {
            binding: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
        },
        super::BindGroupBindingType {
            binding: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        },
    ];

    pub const PROJECTION: [super::BindGroupBindingType; 2] = [
        super::BindGroupBindingType {
            binding: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
        },
        crate::BindGroupBindingType {
            binding: wgpu::BindingType::StorageTexture {
                access: wgpu::StorageTextureAccess::WriteOnly,
                format: wgpu::TextureFormat::Rgba32Float,
                view_dimension: wgpu::TextureViewDimension::D2Array,
            },
        },
    ];
    pub const NORMAL: [super::BindGroupBindingType; 4] = [
        super::BindGroupBindingType {
            binding: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
        },
        super::BindGroupBindingType {
            binding: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        },
        super::BindGroupBindingType {
            binding: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
        },
        super::BindGroupBindingType {
            binding: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        },
    ];
    pub fn create_view(&self, desc: &wgpu::TextureViewDescriptor) -> wgpu::TextureView {
        self.texture.create_view(desc)
    }
    pub  fn from_desc(device: &wgpu::Device, desc: &wgpu::TextureDescriptor<'_>) -> Self {
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
        surface_config: &wgpu::SurfaceConfiguration,
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
            format: surface_config.format,
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
    pub fn depth_stencil_state() -> wgpu::DepthStencilState {
        wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }
    }
    
    pub fn equirect_projection_src_texture(
        device: &wgpu::Device,
        texture: &str,
        format: &wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Texture {
        crate::Texture::new(
            device,
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            *format,
            1,
            wgpu::TextureViewDimension::D2,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            None,
            wgpu::FilterMode::Linear,
            None,
            Some(&format!("{} source texture", texture)),
        )
    }
    pub fn equirect_projection_dst_texture(
        device: &wgpu::Device,
        texture: &str,
        format: &wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Texture {
        crate::Texture::new(
            device,
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 6,
            },
            *format,
            1,
            wgpu::TextureViewDimension::Cube,
            wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            Some(wgpu::AddressMode::ClampToEdge),
            wgpu::FilterMode::Nearest,
            None,
            Some(&format!("{} destination texture", texture)),
        )
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
        key: &str,
        surface_config: &wgpu::SurfaceConfiguration,
        base_dir: &std::path::Path,
    ) -> Result<(Arc<Texture>, CacheKey), EngineError> {
        let cache_key = CacheKey::from(key.to_string());
        if let Some(tex) = self.get(cache_key.clone()) {
            Ok((tex.clone(), cache_key))
        } else {
            let img = image::open(base_dir.join(key))
                .map_err(|e| EngineError::AssetLoadError(e.to_string()))?
                .to_rgba8();
            let tex = Texture::from_image(device, queue, surface_config, &img, key);
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
    fn remove(&mut self, key: &CacheKey) {
        self.textures.remove(key);
    }
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            textures: HashCache::new(),
        }
    }

    /// Retrieve a previously loaded texture
    pub fn get<K: Into<CacheKey>>(&self, key: K) -> Option<Arc<Texture>> {
        self.textures.get(&key.into()).cloned()
    }

    /// Unload a texture from the manager (will free when Arc drops)
    pub fn unload<K: Into<CacheKey>>(&mut self, key: K) {
        self.textures.remove(&key.into());
    }
}
