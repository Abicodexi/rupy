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
    models: crate::HashCache<std::sync::Arc<Model>>,
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
        pipeline_manager: &crate::PipelineManager,
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
                        pipeline_manager.get_render_pipeline(material.pipeline_key.clone())
                    {
                        if let Some(mesh) = <crate::MeshManager as crate::CacheStorage<
                            super::Mesh,
                        >>::get(
                            &mesh_manager, &mesh_instance.material_key
                        ) {
                            let ref_bind_groups: Vec<&wgpu::BindGroup> =
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
