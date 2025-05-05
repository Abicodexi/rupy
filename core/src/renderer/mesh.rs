use std::sync::Arc;

use crate::{
    CacheKey, CacheStorage, HashCache, Managers, MeshInstance, WgpuBuffer, WgpuBufferManager,
};

use super::Material;

#[derive(Clone)]
pub struct Mesh {
    pub key: CacheKey,
    pub vertex_count: u32,
    pub index_count: u32,
}
impl Mesh {
    pub fn draw(
        &self,
        rpass: &mut wgpu::RenderPass,
        pipeline: &wgpu::RenderPipeline,
        bind_groups: &[&wgpu::BindGroup],
        w_buffers: &WgpuBufferManager,
    ) {
        rpass.set_pipeline(pipeline);
        for (i, bind_group) in bind_groups.iter().enumerate() {
            rpass.set_bind_group(i as u32, *bind_group, &[]);
        }

        if let Some(vb) = w_buffers.get(self.key.clone()) {
            rpass.set_vertex_buffer(0, vb.buffer.slice(..));
            rpass.draw(0..self.vertex_count, 0..1);
        }
    }
}

pub struct MeshManager {
    meshes: HashCache<Mesh>,
    device: Arc<wgpu::Device>,
}

impl MeshManager {
    pub fn new(device: Arc<wgpu::Device>) -> Self {
        Self {
            meshes: HashCache::new(),
            device,
        }
    }
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
    pub fn create_instance<V: bytemuck::Pod, I: bytemuck::Pod, K: Into<CacheKey>>(
        managers: &mut Managers,
        key: K,
        vertices: &[V],
        indices: &[I],
        material: &Material,
    ) -> MeshInstance {
        let cache_key: CacheKey = key.into();
        if managers.mesh_manager.contains(&cache_key) {
            return MeshInstance {
                mesh_key: cache_key,
                material_key: CacheKey::from(material.name.clone()),
            };
        }

        let vertex_buffer = WgpuBuffer::from_data(
            &managers.texture_manager.queue(),
            &managers.texture_manager.device(),
            vertices,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            Some(&format!("{} mesh vertex buffer", material.name)),
        );

        let index_buffer = WgpuBuffer::from_data(
            &managers.texture_manager.queue(),
            &managers.texture_manager.device(),
            indices,
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            Some(&format!("{} mesh vertex buffer", material.name)),
        );

        managers.buffer_manager.w_buffer.insert(
            CacheKey {
                id: format!("{}:vertex_buffer", cache_key.id),
            },
            vertex_buffer.into(),
        );
        managers.buffer_manager.w_buffer.insert(
            CacheKey {
                id: format!("{}:index_buffer", cache_key.id),
            },
            index_buffer.into(),
        );
        let mesh = Mesh {
            key: cache_key.clone(),
            vertex_count: vertices.len() as u32,
            index_count: indices.len() as u32,
        };
        managers
            .mesh_manager
            .insert(cache_key.clone(), mesh.clone());

        MeshInstance {
            mesh_key: cache_key,
            material_key: CacheKey::from(material.name.clone()),
        }
    }
}

impl CacheStorage<Mesh> for MeshManager {
    fn get<K: Into<CacheKey>>(&self, key: K) -> Option<&Mesh> {
        self.meshes.get(&key.into())
    }
    fn contains(&self, key: &CacheKey) -> bool {
        self.meshes.contains_key(key)
    }
    fn get_mut(&mut self, key: &CacheKey) -> Option<&mut Mesh> {
        self.meshes.get_mut(key)
    }
    fn get_or_create<F>(&mut self, key: CacheKey, create_fn: F) -> &mut Mesh
    where
        F: FnOnce() -> Mesh,
    {
        self.meshes.entry(key).or_insert_with(create_fn)
    }
    fn insert(&mut self, key: CacheKey, resource: Mesh) {
        self.meshes.insert(key, resource);
    }
    fn remove(&mut self, key: &CacheKey) {
        self.meshes.remove(key);
    }
}
