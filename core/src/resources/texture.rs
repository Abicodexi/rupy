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
    textures: HashCache<Arc<Texture>>,
    pub depth_texture: Texture,
    pub depth_stencil_state: wgpu::DepthStencilState,
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
    pub fn new(depth_stencil_state: wgpu::DepthStencilState, depth_texture: Texture) -> Self {
        Self {
            textures: HashCache::new(),
            depth_texture,
            depth_stencil_state,
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
