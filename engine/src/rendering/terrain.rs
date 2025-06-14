use super::{InstanceBuffer, VertexInstance, CHUNK_SIZE};
use crate::{
    chunk::Chunk, EngineError, Material, Medium, MediumProperties, Mesh, MeshAsset, MeshInstance,
    Position, Renderable, Rotation, Scale, Transform, WgpuBuffer,
};
use glam::Vec3;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug)]
pub struct Terrain {
    chunk_stream: HashMap<(i32, i32, i32), (Chunk, Medium)>,
    default_medium: Medium,
    mesh_instances: Vec<MeshInstance>,
    instance_buffer: Option<InstanceBuffer>,
    last_stream_center: Option<(i32, i32)>,
}

impl Terrain {
    pub fn new(default_medium: Medium) -> Self {
        Self {
            chunk_stream: HashMap::new(),
            default_medium,
            mesh_instances: Vec::new(),
            instance_buffer: None,
            last_stream_center: None,
        }
    }

    pub fn insert_chunk_stream(&mut self, chunk: Chunk, medium: Medium) {
        self.chunk_stream.insert(chunk.pos, (chunk, medium));
    }

    pub fn get_chunk_stream(&self, pos: (i32, i32, i32)) -> Option<&(Chunk, Medium)> {
        self.chunk_stream.get(&pos)
    }

    pub fn get_chunk_stream_mut(&mut self, pos: (i32, i32, i32)) -> Option<&mut (Chunk, Medium)> {
        self.chunk_stream.get_mut(&pos)
    }

    pub fn default_medium(&self) -> Medium {
        self.default_medium
    }

    pub fn medium_at(&self, world_pos: Vec3) -> Medium {
        let chunk_pos = (
            (world_pos.x / CHUNK_SIZE as f32).floor() as i32,
            (world_pos.y / CHUNK_SIZE as f32).floor() as i32,
            (world_pos.z / CHUNK_SIZE as f32).floor() as i32,
        );

        if let Some((chunk, medium)) = self.chunk_stream.get(&chunk_pos) {
            let lx = (world_pos.x as isize % CHUNK_SIZE as isize).rem_euclid(CHUNK_SIZE as isize);
            let ly = (world_pos.y as isize % CHUNK_SIZE as isize).rem_euclid(CHUNK_SIZE as isize);
            let lz = (world_pos.z as isize % CHUNK_SIZE as isize).rem_euclid(CHUNK_SIZE as isize);
            if chunk.get_block(lx, ly, lz) == 0 {
                Medium::Air
            } else {
                *medium
            }
        } else {
            self.default_medium
        }
    }

    pub fn medium_properties_at(&self, world_pos: Vec3) -> MediumProperties {
        self.medium_at(world_pos).properties()
    }

    pub fn stream_build_meshes(&mut self) {
        for (chunk, ..) in self.chunk_stream.values_mut() {
            if chunk.dirty {
                let mesh = chunk.build_chunk_mesh();
                chunk.mesh = Some(mesh);
                chunk.dirty = false;
            }
        }
    }
    pub fn instance_buffer(&self) -> Option<&InstanceBuffer> {
        self.instance_buffer.as_ref()
    }
    pub fn update_instance_buffer(&mut self, queue: &wgpu::Queue, device: &wgpu::Device) {
        let mut instances = Vec::new();

        for ((cx, cy, cz), _chunk) in &self.chunk_stream {
            instances.push(
                Transform::from_components(
                    &Position::new(*cx as f32, *cy as f32, *cz as f32),
                    &Rotation::zero(),
                    &Scale::one(),
                )
                .into(),
            );
        }

        if let Some(instance) = &mut self.instance_buffer {
            let byte_data = VertexInstance::bytes(&instances);
            instance.buffer.write_data(queue, device, &byte_data, None);
        } else {
            let byte_data = VertexInstance::bytes(&instances);
            let buffer = WgpuBuffer::from_data(
                device,
                &byte_data,
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                Some("terrain_instance_buffer"),
            );
            self.instance_buffer = Some(InstanceBuffer {
                buffer,
                count: instances.len(),
                capacity: byte_data.len(),
                dirty: false,
            });
        }
    }
    pub fn all_meshes(&self) -> impl Iterator<Item = &MeshAsset> {
        #[allow(unused_variables)]
        self.chunk_stream
            .values()
            .filter_map(|(c, m)| c.mesh.as_ref())
    }

    fn stream_build_chunks(&mut self, center: (i32, i32), distance: i32) {
        let mut needed: std::collections::HashSet<(i32, i32, i32)> =
            std::collections::HashSet::new();
        for dx in -distance..=distance {
            for dz in -distance..=distance {
                let chunk_pos = (center.0 + dx, 0, center.1 + dz);
                if needed.insert(chunk_pos) && !self.chunk_stream.contains_key(&chunk_pos) {
                    let medium = self.medium_at(Vec3 {
                        x: chunk_pos.0 as f32,
                        y: chunk_pos.1 as f32,
                        z: chunk_pos.2 as f32,
                    });
                    self.insert_chunk_stream(Chunk::flat(chunk_pos), medium);
                }
            }
        }
        self.chunk_stream.retain(|pos, _| needed.contains(pos));
        self.last_stream_center = Some(center);
    }

    pub fn update_streaming(&mut self, camera_pos: Vec3, view_distance: i32) {
        let center = ((camera_pos.x).floor() as i32, (camera_pos.z).floor() as i32);

        if self.last_stream_center == Some(center) {
            return;
        }
        let old_center = self.last_stream_center.unwrap_or(center);
        if (old_center.0 + center.0).abs() >= view_distance
            || (old_center.1 + center.1).abs() >= view_distance
        {
            self.stream_build_chunks(center, view_distance);
            self.stream_build_meshes();
        }
    }
    pub fn chunks(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        center: Vec3,
        radius: i32,
        medium: Medium,
        material: &Arc<Material>,
    ) -> Result<Renderable, EngineError> {
        let terrain_mat = "ground";

        self.mesh_instances.clear();
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let pos = (center.x as i32 + dx, 0, center.z as i32 + dz);

                let chunk = Chunk::flat(pos);
                let mesh_asset = chunk.build_chunk_mesh();
                let mesh = Mesh::from_asset(queue, device, mesh_asset, &format!("chunk_{:?}", pos));
                let mesh_instance = MeshInstance {
                    mesh: Arc::new(mesh),
                    material: Some(material.clone()),
                };
                self.insert_chunk_stream(chunk, medium);
                self.mesh_instances.push(mesh_instance);
            }
        }
        let renderable = Renderable::new(terrain_mat.into());

        Ok(renderable)
    }
    pub fn mesh_instances(&self) -> &[MeshInstance] {
        &self.mesh_instances
    }
}
