use crate::CacheStorage;

#[derive(Copy, Clone, Debug)]
pub struct AABB {
    pub min: cgmath::Point3<f32>,
    pub max: cgmath::Point3<f32>,
}
impl AABB {
    pub fn get_positive_vertex(&self, normal: cgmath::Vector3<f32>) -> cgmath::Point3<f32> {
        cgmath::Point3::new(
            if normal.x >= 0.0 {
                self.max.x
            } else {
                self.min.x
            },
            if normal.y >= 0.0 {
                self.max.y
            } else {
                self.min.y
            },
            if normal.z >= 0.0 {
                self.max.z
            } else {
                self.min.z
            },
        )
    }
}
#[derive(Clone)]
pub struct Model {
    pub meshes: Vec<super::MeshInstance>,
    pub bounding_radius: AABB,
    pub name: String,
}

impl Model {
    pub async fn load<V: bytemuck::Pod, I: bytemuck::Pod>(
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
        let material = crate::Material::create(
            device,
            managers,
            config,
            bind_group_layouts,
            bind_groups,
            buffers,
            material_name,
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

        let mesh_instance =
            crate::MeshInstance::new(queue, device, managers, vertices, indices, &material);
        let model = std::sync::Arc::new(crate::Model {
            meshes: vec![mesh_instance],
            bounding_radius: aabb,
            name: model_name.to_string(),
        });
        crate::CacheStorage::insert(
            &mut managers.model_manager,
            model_name.into(),
            model.clone(),
        );

        Ok(model)
    }
    pub fn from_obj<P: AsRef<std::path::Path>>(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        obj: P,
        managers: &mut crate::Managers,
        camera: &crate::camera::Camera,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> Result<crate::Model, crate::EngineError> {
        let base_path = crate::asset_dir()?.join("models");
        let obj_path = base_path.join(obj.as_ref());
        let (models, materials) = tobj::load_obj(
            &obj_path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )
        .map_err(|e| crate::EngineError::AssetLoadError(format!("OBJ parse error: {}", e)))?;

        if let Err(e) = materials {
            crate::log_error!("Error loading obj materials: {}", e);
            return Err(crate::EngineError::AssetLoadError(format!(
                "Error loading obj materials: {}",
                e
            )));
        }
        if let Ok(vec_m) = materials {
            let mats = crate::Material::load_tobj_materials(
                queue,
                device,
                managers,
                camera,
                surface_config,
                &vec_m,
            )?;

            let mut instances = Vec::with_capacity(models.len());
            let mut aabb: Option<crate::AABB> = None;
            for m in models {
                let mesh = m.mesh;

                let vertices: Vec<_> = mesh
                    .positions
                    .chunks(3)
                    .zip(
                        mesh.texcoords
                            .chunks(2)
                            .chain(std::iter::repeat(&[0.0, 0.0][..])),
                    )
                    .map(|(pos, uv)| crate::VertexTexture {
                        position: [pos[0], pos[1], pos[2]],
                        color: [1.0, 1.0, 1.0],
                        tex_coords: [uv[0], uv[1]],
                    })
                    .collect();
                let mat_id = mesh.material_id.unwrap_or(0);
                let mat = mats
                    .get(mat_id)
                    .cloned()
                    .unwrap_or_else(|| crate::Material::default());

                instances.push(crate::MeshInstance::new(
                    queue,
                    device,
                    managers,
                    &vertices,
                    &mesh.indices,
                    &mat,
                ));
                if aabb.is_none() {
                    aabb = Some(crate::Model::compute_aabb(&vertices));
                }
            }

            let name = obj
                .as_ref()
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unnamed")
                .to_string();

            return Ok(crate::Model {
                meshes: instances,
                bounding_radius: aabb.unwrap(),
                name,
            });
        }
        Err(crate::EngineError::AssetLoadError(format!(
            "Error loading obj",
        )))
    }
    pub fn compute_aabb(vertices: &[crate::VertexTexture]) -> AABB {
        let mut min = cgmath::Vector3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = cgmath::Vector3::new(f32::MIN, f32::MIN, f32::MIN);
        for v in vertices {
            let pos = cgmath::Vector3::new(v.position[0], v.position[1], v.position[2]);
            min = min.zip(pos, f32::min);
            max = max.zip(pos, f32::max);
        }
        AABB {
            min: <cgmath::Point3<f32> as cgmath::EuclideanSpace>::from_vec(min),
            max: <cgmath::Point3<f32> as cgmath::EuclideanSpace>::from_vec(max),
        }
    }
}

impl Into<crate::CacheKey> for Model {
    fn into(self) -> crate::CacheKey {
        crate::CacheKey::new(self.name)
    }
}

pub struct ModelManager {
    pub models: crate::HashCache<std::sync::Arc<Model>>,
}
impl ModelManager {
    pub fn new() -> Self {
        Self {
            models: std::collections::HashMap::new(),
        }
    }

    pub fn render(
        &self,
        rpass: &mut wgpu::RenderPass,
        render_pipeline_manager: &crate::RenderPipelineManager,
        buffer_manager: &crate::BufferManager,
        material_manager: &super::MaterialManager,
        mesh_manager: &crate::MeshManager,
    ) {
        for (.., model) in self.models.iter() {
            for mesh_instance in model.meshes.iter() {
                if let Some(material) =
                    crate::CacheStorage::get(material_manager, &mesh_instance.material_key)
                {
                    if let Some(material_pipeline) =
                        render_pipeline_manager.get(&material.shader_key)
                    {
                        if let Some(mesh) = <crate::MeshManager as crate::CacheStorage<
                            super::Mesh,
                        >>::get(
                            &mesh_manager, &mesh_instance.material_key
                        ) {
                            let ref_bind_groups: Vec<&std::sync::Arc<wgpu::BindGroup>> =
                                material.bind_groups.iter().collect();
                            mesh.draw(
                                rpass,
                                &material_pipeline,
                                &ref_bind_groups,
                                &buffer_manager.w_buffer,
                            );
                        }
                    }
                }
            }
        }
    }
}

impl crate::CacheStorage<std::sync::Arc<Model>> for ModelManager {
    fn get(&self, key: &crate::CacheKey) -> Option<&std::sync::Arc<Model>> {
        self.models.get(key)
    }
    fn contains(&self, key: &crate::CacheKey) -> bool {
        self.models.contains_key(key)
    }
    fn get_mut(&mut self, key: &crate::CacheKey) -> Option<&mut std::sync::Arc<Model>> {
        self.models.get_mut(key)
    }
    fn get_or_create<F>(&mut self, key: crate::CacheKey, create_fn: F) -> &mut std::sync::Arc<Model>
    where
        F: FnOnce() -> std::sync::Arc<Model>,
    {
        self.models.entry(key).or_insert_with(create_fn)
    }
    fn insert(&mut self, key: crate::CacheKey, resource: std::sync::Arc<Model>) {
        self.models.insert(key, resource);
    }
    fn remove(&mut self, key: &crate::CacheKey) {
        self.models.remove(key);
    }
}
