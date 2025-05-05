#[derive(Clone)]
pub struct Mesh {
    pub vertex_buffer_key: crate::CacheKey,
    pub index_buffer_key: crate::CacheKey,
    pub vertex_count: u32,
    pub index_count: u32,
}
impl Mesh {
    pub fn draw(
        &self,
        rpass: &mut wgpu::RenderPass,
        pipeline: &wgpu::RenderPipeline,
        bind_groups: &[&wgpu::BindGroup],
        w_buffers: &crate::WgpuBufferManager,
    ) {
        rpass.set_pipeline(pipeline);
        for (i, bind_group) in bind_groups.iter().enumerate() {
            rpass.set_bind_group(i as u32, *bind_group, &[]);
        }

        if let (Some(vertex_buffer), Some(index_buffer)) = (
            crate::CacheStorage::get(w_buffers, &self.vertex_buffer_key),
            crate::CacheStorage::get(w_buffers, &self.index_buffer_key),
        ) {
            rpass.set_vertex_buffer(0, vertex_buffer.buffer.slice(..));
            rpass.set_index_buffer(index_buffer.buffer.slice(..), wgpu::IndexFormat::Uint16);
            rpass.draw(0..self.vertex_count, 0..1);
        }
    }
}
#[derive(Clone)]
pub struct MeshInstance {
    pub vertex_buffer_key: crate::CacheKey,
    pub index_buffer_key: crate::CacheKey,
    pub material_key: crate::CacheKey,
}
impl MeshInstance {
    pub fn new<V: bytemuck::Pod, I: bytemuck::Pod>(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        managers: &mut crate::Managers,
        vertices: &[V],
        indices: &[I],
        material: &crate::Material,
    ) -> Self {
        let material_key = crate::CacheKey::from(material.name.clone());

        let vertex_buffer_key = crate::CacheKey {
            id: format!("{}:vertex_buffer", material_key.id),
        };
        let index_buffer_key = crate::CacheKey {
            id: format!("{}:index_buffer", material_key.id),
        };

        if !crate::CacheStorage::contains(&managers.buffer_manager.w_buffer, &vertex_buffer_key) {
            let vertex_buffer = crate::WgpuBuffer::from_data(
                queue,
                device,
                vertices,
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                Some(&format!("{} mesh vertex buffer", vertex_buffer_key.id)),
            );
            crate::CacheStorage::insert(
                &mut managers.buffer_manager.w_buffer,
                vertex_buffer_key.clone(),
                vertex_buffer.into(),
            );
        }

        if !crate::CacheStorage::contains(&managers.buffer_manager.w_buffer, &index_buffer_key) {
            let index_buffer = crate::WgpuBuffer::from_data(
                queue,
                device,
                indices,
                wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                Some(&format!("{} mesh vertex buffer", index_buffer_key.id)),
            );

            crate::CacheStorage::insert(
                &mut managers.buffer_manager.w_buffer,
                index_buffer_key.clone(),
                index_buffer.into(),
            );
        }

        if !crate::CacheStorage::contains(&managers.mesh_manager.meshes, &material_key) {
            let vertex_count = vertices.len() as u32;
            let index_count = indices.len() as u32;
            let mesh = Mesh {
                vertex_buffer_key: vertex_buffer_key.clone(),
                index_buffer_key: index_buffer_key.clone(),
                vertex_count,
                index_count,
            };
            crate::CacheStorage::insert(&mut managers.mesh_manager, material_key.clone(), mesh);
        }

        Self {
            vertex_buffer_key,
            index_buffer_key,
            material_key,
        }
    }
}

pub struct MeshManager {
    pub meshes: crate::HashCache<Mesh>,
}

impl MeshManager {
    pub fn new() -> Self {
        Self {
            meshes: crate::HashCache::new(),
        }
    }
}

impl crate::CacheStorage<Mesh> for MeshManager {
    fn get(&self, key: &crate::CacheKey) -> Option<&Mesh> {
        self.meshes.get(key)
    }
    fn contains(&self, key: &crate::CacheKey) -> bool {
        self.meshes.contains_key(key)
    }
    fn get_mut(&mut self, key: &crate::CacheKey) -> Option<&mut Mesh> {
        self.meshes.get_mut(key)
    }
    fn get_or_create<F>(&mut self, key: crate::CacheKey, create_fn: F) -> &mut Mesh
    where
        F: FnOnce() -> Mesh,
    {
        self.meshes.entry(key).or_insert_with(create_fn)
    }
    fn insert(&mut self, key: crate::CacheKey, resource: Mesh) {
        self.meshes.insert(key, resource);
    }
    fn remove(&mut self, key: &crate::CacheKey) {
        self.meshes.remove(key);
    }
}
