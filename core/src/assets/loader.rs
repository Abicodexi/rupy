use std::sync::Arc;

use crate::CacheStorage;

static BASE_PATH: once_cell::sync::Lazy<std::path::PathBuf> =
    once_cell::sync::Lazy::new(|| super::asset_dir().expect("couldn’t find asset dir"));

pub struct Asset;
impl Asset {
    pub fn base_path() -> &'static std::path::PathBuf {
        &*BASE_PATH
    }
    pub fn resolve(rel_path: &str) -> std::path::PathBuf {
        Asset::base_path().join(rel_path)
    }

    pub fn read_text(rel_path: &str) -> Result<String, crate::EngineError> {
        let path = Asset::resolve(rel_path);
        std::fs::read_to_string(&path).map_err(|e| {
            crate::EngineError::FileSystemError(format!("Failed to read {:?}: {}", path, e))
        })
    }

    pub fn shader(
        managers: &mut crate::Managers,
        rel_path: &str,
    ) -> Result<std::sync::Arc<wgpu::ShaderModule>, crate::EngineError> {
        if let Ok(gpu) = crate::GPU::get().read() {
            let cache_key = crate::CacheKey::from(rel_path);

            if !managers.shader_manager.contains(&cache_key) {
                let path = Asset::base_path().join("shaders").join(rel_path);

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
    pub async fn texture(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        managers: &mut crate::Managers,
        surface_config: &wgpu::SurfaceConfiguration,
        rel_path: &str,
    ) -> Result<std::sync::Arc<crate::Texture>, crate::EngineError> {
        let cache_key = crate::CacheKey::from(rel_path);
        if !managers.texture_manager.contains(&cache_key) {
            let img = image::open(&cache_key.id)
                .map_err(|e| {
                    crate::EngineError::AssetLoadError(format!("Texture load failed: {}", e))
                })?
                .to_rgba8();
            let tex = crate::Texture::from_image(
                device,
                queue,
                surface_config,
                &img,
                cache_key.id.clone(),
            );
            managers
                .texture_manager
                .insert(cache_key.clone(), tex.into());
        }

        Ok(managers.texture_manager.get(cache_key).unwrap())
    }
    pub fn tobj<P: AsRef<std::path::Path>>(
        obj: P,
        managers: &mut crate::Managers,
        uniform_bind_group: &wgpu::BindGroup,
        camera: &crate::camera::Camera,
        light: &crate::Light,
        surface_config: &wgpu::SurfaceConfiguration,
        depth_stencil_state: &Option<wgpu::DepthStencilState>,
    ) -> Result<Arc<crate::Model>, crate::EngineError> {
        if let Ok(Some(model)) = crate::Model::from_obj(
            obj,
            managers,
            uniform_bind_group,
            camera,
            light,
            surface_config,
            depth_stencil_state,
        ) {
            Ok(model)
        } else {
            Err(crate::EngineError::AssetLoadError(
                "Failed to load model {} from obj".into(),
            ))
        }
    }

    pub async fn model<V: bytemuck::Pod, I: bytemuck::Pod>(
        managers: &mut crate::Managers,
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        bind_groups: Vec<std::sync::Arc<wgpu::BindGroup>>,
        model_name: &str,
        material_name: &str,
        shader_rel_path: &str,
        diffuse_texture: Option<&str>,
        normal_texture: Option<&str>,
        blend_state: Option<wgpu::BlendState>,
        cull_mode: Option<wgpu::Face>,
        topology: wgpu::PrimitiveTopology,
        front_face: wgpu::FrontFace,
        polygon_mode: wgpu::PolygonMode,
        vertices: &[V],
        indices: &[I],
        aabb: crate::AABB,
    ) -> Result<std::sync::Arc<crate::Model>, crate::EngineError> {
        let material = crate::Material::new(
            bind_group_layouts,
            bind_groups,
            material_name,
            shader_rel_path,
            diffuse_texture,
            normal_texture,
            topology,
            front_face,
            polygon_mode,
            blend_state,
            cull_mode,
        );
        crate::Model::from_material(managers, material, model_name, vertices, indices, aabb).await
    }
}
