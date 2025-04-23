use std::sync::Arc;

use crate::{CacheKey, CacheStorage, EngineError, HashCache};

/// A GPU-ready texture: the texture itself, a view, and a sampler.
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub label: String,
}

impl Texture {
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
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
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
}

pub struct TextureManager {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    textures: HashCache<Arc<Texture>>,
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
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self {
            device,
            queue,
            textures: HashCache::new(),
        }
    }

    /// Load (or reload) a texture from disk
    pub async fn load<K: Into<CacheKey>, P: AsRef<std::path::Path>>(
        &mut self,
        key: K,
        path: P,
    ) -> Result<Arc<Texture>, EngineError> {
        let bytes = std::fs::read(&path)?;
        let tex = Texture::from_bytes(&self.device, &self.queue, &bytes, path).await?;
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
}
