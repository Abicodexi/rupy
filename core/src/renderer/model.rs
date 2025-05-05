use std::{collections::HashMap, sync::Arc};

use cgmath::{EuclideanSpace, Point3, Vector3};

use crate::{
    buffer::BufferManager, pipeline::PipelineManager, CacheKey, CacheStorage, HashCache,
    MeshInstance, MeshManager,
};

use super::{material::MaterialManager, VertexTexture};
#[derive(Copy, Clone, Debug)]
pub struct AABB {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}
impl AABB {
    pub fn get_positive_vertex(&self, normal: Vector3<f32>) -> Point3<f32> {
        Point3::new(
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
    pub meshes: Vec<MeshInstance>,
    pub bounding_radius: AABB,
    pub name: String,
}
impl Model {
    pub fn compute_aabb(vertices: &[VertexTexture]) -> AABB {
        let mut min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vector3::new(f32::MIN, f32::MIN, f32::MIN);
        for v in vertices {
            let pos = Vector3::new(v.position[0], v.position[1], v.position[2]);
            min = min.zip(pos, f32::min);
            max = max.zip(pos, f32::max);
        }
        AABB {
            min: Point3::from_vec(min),
            max: Point3::from_vec(max),
        }
    }
}
impl Into<CacheKey> for Model {
    fn into(self) -> CacheKey {
        CacheKey::new(self.name)
    }
}

pub struct ModelManager {
    models: HashCache<Arc<Model>>,
}
impl ModelManager {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    pub fn render(
        &self,
        rpass: &mut wgpu::RenderPass,
        pipeline_manager: &PipelineManager,
        buffer_manager: &BufferManager,
        material_manager: &MaterialManager,
        mesh_manager: &MeshManager,
    ) {
        for (.., model) in self.models.iter() {
            for mesh_instance in model.meshes.iter() {
                if let Some(material) = material_manager.get(mesh_instance.material_key.clone()) {
                    if let Some(material_pipeline) =
                        pipeline_manager.get_render_pipeline(material.pipeline_key.clone())
                    {
                        if let Some(mesh) = mesh_manager.get(mesh_instance.mesh_key.id.clone()) {
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

impl CacheStorage<Arc<Model>> for ModelManager {
    fn get<K: Into<CacheKey>>(&self, key: K) -> Option<&Arc<Model>> {
        self.models.get(&key.into())
    }
    fn contains(&self, key: &CacheKey) -> bool {
        self.models.contains_key(key)
    }
    fn get_mut(&mut self, key: &CacheKey) -> Option<&mut Arc<Model>> {
        self.models.get_mut(key)
    }
    fn get_or_create<F>(&mut self, key: CacheKey, create_fn: F) -> &mut Arc<Model>
    where
        F: FnOnce() -> Arc<Model>,
    {
        self.models.entry(key).or_insert_with(create_fn)
    }
    fn insert(&mut self, key: CacheKey, resource: Arc<Model>) {
        self.models.insert(key, resource);
    }
    fn remove(&mut self, key: &CacheKey) {
        self.models.remove(key);
    }
}
