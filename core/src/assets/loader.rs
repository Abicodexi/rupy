pub struct AssetLoader {
    base_path: std::path::PathBuf,
    device: std::sync::Arc<wgpu::Device>,
}

impl AssetLoader {
    pub fn new(device: std::sync::Arc<wgpu::Device>) -> Result<Self, crate::EngineError> {
        let base_path = super::asset_dir()?;
        Ok(Self { device, base_path })
    }

    pub fn resolve(&self, rel_path: &str) -> std::path::PathBuf {
        self.base_path.join(rel_path)
    }

    pub fn load_text(&self, rel_path: &str) -> Result<String, crate::EngineError> {
        let path = self.resolve(rel_path);
        std::fs::read_to_string(&path).map_err(|e| {
            crate::EngineError::FileSystemError(format!("Failed to read {:?}: {}", path, e))
        })
    }

    pub fn load_shader(&self, rel_path: &str) -> Result<wgpu::ShaderModule, crate::EngineError> {
        let path = self.base_path.join("shaders").join(rel_path);

        let shader_source = std::fs::read_to_string(&path)?;

        let shader_module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(rel_path),
                source: wgpu::ShaderSource::Wgsl(shader_source.into()),
            });

        Ok(shader_module)
    }
    pub fn read_bytes<P: AsRef<std::path::Path>>(path: &P) -> Result<Vec<u8>, crate::EngineError> {
        let bytes = std::fs::read(path)?;
        Ok(bytes)
    }
    pub async fn load_texture(
        &self,
        queue: &wgpu::Queue,
        rel_path: &str,
    ) -> Result<crate::Texture, crate::EngineError> {
        let path = self.base_path.join("textures").join(rel_path);

        let bytes = Self::read_bytes(&path)?;
        let tex = crate::Texture::from_bytes(&self.device, queue, &bytes, path).await?;
        Ok(tex)
    }
    pub async fn load_model<V: bytemuck::Pod, I: bytemuck::Pod>(
        &self,
        resources: &crate::Resources,
        managers: &mut crate::Managers,
        config: &wgpu::SurfaceConfiguration,
        bind_group_layouts: Vec<&wgpu::BindGroupLayout>,
        bind_groups: Vec<wgpu::BindGroup>,
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
    ) -> Result<(), crate::EngineError> {
        use crate::cache::CacheStorage;

        let material = managers
            .material_manager
            .create_material(
                resources,
                &mut managers.shader_manager,
                &mut managers.texture_manager,
                &mut managers.pipeline_manager,
                &config,
                bind_group_layouts,
                bind_groups,
                buffers,
                &material_name,
                shader_rel_path,
                texture_rel_path,
                texture_bind_group_layout,
                topology,
                front_face,
                polygon_mode,
                blend_state,
                cull_mode,
            )
            .await?;

        let mesh_instance = crate::MeshInstance::new(
            &resources.gpu.queue,
            &resources.gpu.device,
            managers,
            vertices,
            indices,
            &material,
        );

        managers.model_manager.insert(
            model_name.into(),
            crate::Model {
                meshes: vec![mesh_instance],
                bounding_radius: aabb,
                name: model_name.to_string(),
            }
            .into(),
        );

        Ok(())
    }
}
