use crate::CacheStorage;

static BASE_PATH: once_cell::sync::Lazy<std::path::PathBuf> =
    once_cell::sync::Lazy::new(|| super::asset_dir().expect("couldn’t find asset dir"));

pub struct AssetLoader;
impl AssetLoader {
    pub fn base_path() -> &'static std::path::PathBuf {
        &*BASE_PATH
    }
    pub fn resolve(rel_path: &str) -> std::path::PathBuf {
        AssetLoader::base_path().join(rel_path)
    }

    pub fn load_text(rel_path: &str) -> Result<String, crate::EngineError> {
        let path = AssetLoader::resolve(rel_path);
        std::fs::read_to_string(&path).map_err(|e| {
            crate::EngineError::FileSystemError(format!("Failed to read {:?}: {}", path, e))
        })
    }

    pub fn load_shader(
        managers: &mut crate::Managers,
        rel_path: &str,
    ) -> Result<std::sync::Arc<wgpu::ShaderModule>, crate::EngineError> {
        if let Ok(gpu) = crate::GPU::get().read() {
            let cache_key = crate::CacheKey::from(rel_path);

            if !managers.shader_manager.contains(&cache_key) {
                let path = AssetLoader::base_path().join("shaders").join(rel_path);

                let shader_source = std::fs::read_to_string(&path)?;
                let shader_module =
                    gpu.device()
                        .create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: Some(&cache_key.id),
                            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
                        });
                managers
                    .shader_manager
                    .insert(cache_key.clone(), shader_module.into());
            }

            Ok(managers.shader_manager.get(&cache_key).unwrap().clone())
        } else {
            Err(crate::EngineError::RwLockError(
                "Loading shader failed. Could not acquire read lock on gpu device".into(),
            ))
        }
    }
    pub fn read_bytes<P: AsRef<std::path::Path>>(path: &P) -> Result<Vec<u8>, crate::EngineError> {
        let bytes = std::fs::read(path)?;
        Ok(bytes)
    }
    pub async fn load_texture(
        managers: &mut crate::Managers,
        rel_path: &str,
    ) -> Result<std::sync::Arc<crate::Texture>, crate::EngineError> {
        if let Ok(gpu) = crate::GPU::get().read() {
            let cache_key = crate::CacheKey::from(rel_path);
            if !managers.texture_manager.contains(&cache_key) {
                let path = AssetLoader::base_path()
                    .join("textures")
                    .join(cache_key.id.clone());
                let bytes = Self::read_bytes(&path)?;
                let tex = crate::Texture::from_bytes(
                    gpu.device(),
                    gpu.queue(),
                    &bytes,
                    cache_key.id.clone(),
                )
                .await?;
                managers
                    .texture_manager
                    .insert(cache_key.clone(), tex.into());
            }

            Ok(managers.texture_manager.get(cache_key).unwrap())
        } else {
            Err(crate::EngineError::RwLockError(
                "Loading shader failed. Could not acquire read lock on gpu device".into(),
            ))
        }
    }
    pub fn load_tobj<P: AsRef<std::path::Path>>(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        obj: P,
        managers: &mut crate::Managers,
        camera: &crate::camera::Camera,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> Result<crate::Model, crate::EngineError> {
        crate::Model::from_obj(queue, device, obj, managers, camera, surface_config)
    }

    pub async fn load_model<V: bytemuck::Pod, I: bytemuck::Pod>(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        managers: &mut crate::Managers,
        config: &wgpu::SurfaceConfiguration,
        bind_group_layouts: Vec<&wgpu::BindGroupLayout>,
        bind_groups: Vec<std::sync::Arc<wgpu::BindGroup>>,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        model_name: &str,
        material_name: &str,
        shader_rel_path: &str,
        texture_rel_path: Option<&str>,
        texture_bind_group_layout: Option<&wgpu::BindGroupLayout>,
        blend_state: Option<wgpu::BlendState>,
        cull_mode: Option<wgpu::Face>,
        topology: wgpu::PrimitiveTopology,
        front_face: wgpu::FrontFace,
        polygon_mode: wgpu::PolygonMode,
        vertices: &[V],
        indices: &[I],
        aabb: crate::AABB,
    ) -> Result<std::sync::Arc<crate::Model>, crate::EngineError> {
        crate::Model::load(
            queue,
            device,
            managers,
            config,
            bind_group_layouts,
            bind_groups,
            buffers,
            model_name,
            material_name,
            shader_rel_path,
            texture_rel_path,
            texture_bind_group_layout,
            blend_state,
            cull_mode,
            topology,
            front_face,
            polygon_mode,
            vertices,
            indices,
            aabb,
        )
        .await
    }
}
