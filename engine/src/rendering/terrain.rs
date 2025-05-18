use glam::Vec3;

use crate::{
    chunk::Chunk, CacheKey, Material, MaterialAsset, Mesh, MeshAsset, MeshInstance, Position,
    Renderable, Rotation, Scale, Transform, WgpuBuffer,
};
use std::{collections::HashMap, sync::Arc};

use super::{InstanceBufferData, Vertex, VertexInstance, CHUNK_SIZE};

pub const GRAVITY: f32 = -9.81;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Medium {
    Air,
    Water,
    Ground,
    Vacuum,
}
#[derive(Debug, Clone, Copy)]
pub struct MediumProperties {
    pub gravity: Vec3,
    pub drag: f32,
}

impl Medium {
    pub fn properties(self) -> MediumProperties {
        match self {
            Medium::Air => MediumProperties {
                gravity: Vec3::new(0.0, GRAVITY, 0.0),
                drag: 0.05,
            },
            Medium::Water => MediumProperties {
                gravity: Vec3::new(0.0, GRAVITY + 7.81, 0.0),
                drag: 0.1,
            },
            Medium::Ground => MediumProperties {
                gravity: Vec3::new(0.0, GRAVITY, 0.0),
                drag: 0.01,
            },
            Medium::Vacuum => MediumProperties {
                gravity: Vec3::ZERO,
                drag: 0.9,
            },
        }
    }
    pub fn is_solid(self) -> bool {
        matches!(self, Medium::Ground)
    }

    pub fn is_fluid(self) -> bool {
        matches!(self, Medium::Air | Medium::Water)
    }
}
#[derive(Debug)]
pub struct Terrain {
    chunks: HashMap<(i32, i32, i32), Chunk>,
    default_medium: Medium,
    mesh_instances: Vec<MeshInstance>,
    instance_buffer: Option<InstanceBufferData>,
    last_stream_center: Option<(i32, i32)>,
}

impl Terrain {
    pub fn new(default_medium: Medium) -> Self {
        Self {
            chunks: HashMap::new(),
            default_medium,
            mesh_instances: Vec::new(),
            instance_buffer: None,
            last_stream_center: None,
        }
    }

    pub fn generate_flat_ground(&mut self, zfar: f32) {
        let chunk_radius = ((zfar / CHUNK_SIZE as f32).ceil() as i32).max(1);

        for cx in -chunk_radius..=chunk_radius {
            for cz in -chunk_radius..=chunk_radius {
                let pos = (cx, 0, cz);
                let chunk = Chunk::flat(pos);
                self.chunks.insert(pos, chunk);
            }
        }

        self.update_meshes();
    }

    pub fn insert_chunk(&mut self, chunk: Chunk) {
        self.chunks.insert(chunk.pos, chunk);
    }

    pub fn get_chunk(&self, pos: (i32, i32, i32)) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    pub fn get_chunk_mut(&mut self, pos: (i32, i32, i32)) -> Option<&mut Chunk> {
        self.chunks.get_mut(&pos)
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

        if let Some(chunk) = self.chunks.get(&chunk_pos) {
            let lx = (world_pos.x as isize % CHUNK_SIZE as isize).rem_euclid(CHUNK_SIZE as isize);
            let ly = (world_pos.y as isize % CHUNK_SIZE as isize).rem_euclid(CHUNK_SIZE as isize);
            let lz = (world_pos.z as isize % CHUNK_SIZE as isize).rem_euclid(CHUNK_SIZE as isize);

            if chunk.get_block(lx, ly, lz) == 0 {
                Medium::Air
            } else {
                Medium::Ground
            }
        } else {
            self.default_medium
        }
    }

    pub fn medium_properties_at(&self, world_pos: Vec3) -> MediumProperties {
        self.medium_at(world_pos).properties()
    }

    pub fn update_meshes(&mut self) {
        for chunk in self.chunks.values_mut() {
            if chunk.dirty {
                chunk.mesh = Some(chunk.build_chunk_mesh());
                chunk.dirty = false;
            }
        }
    }
    pub fn instance_buffer(&self) -> Option<&InstanceBufferData> {
        self.instance_buffer.as_ref()
    }
    pub fn update_instance_buffer(&mut self, queue: &wgpu::Queue, device: &wgpu::Device) {
        let mut instances = Vec::new();

        for ((cx, cy, cz), _) in &self.chunks {
            let transform = Transform::from_components(
                &Position::new(*cx as f32, *cy as f32, *cz as f32),
                &Rotation::zero(),
                &Scale::one(),
            );
            let vertex_instances: VertexInstance = transform.to_vertex_instance(0);
            instances.push(vertex_instances);
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
            self.instance_buffer = Some(InstanceBufferData {
                buffer,
                count: instances.len(),
                capacity: byte_data.len(),
                dirty: false,
            });
        }
    }

    pub fn all_meshes(&self) -> impl Iterator<Item = &MeshAsset> {
        self.chunks.values().filter_map(|c| c.mesh.as_ref())
    }

    pub fn update_streaming(&mut self, camera_pos: Vec3, view_distance: i32) {
        let new_center = ((camera_pos.x).floor() as i32, (camera_pos.z).floor() as i32);

        if self.last_stream_center == Some(new_center) {
            return;
        }
        self.last_stream_center = Some(new_center);

        let mut needed_chunks = std::collections::HashSet::new();

        for dx in -view_distance..=view_distance {
            for dz in -view_distance..=view_distance {
                let chunk_pos = (new_center.0 + dx, 0, new_center.1 + dz);
                needed_chunks.insert(chunk_pos);
                if !self.chunks.contains_key(&chunk_pos) {
                    let chunk = Chunk::flat(chunk_pos);
                    self.insert_chunk(chunk);
                }
            }
        }

        self.chunks.retain(|pos, _| needed_chunks.contains(pos));
        self.update_meshes();
    }

    pub fn generate_initial_chunks(
        &mut self,
        center: Vec3,
        radius: i32,
        surface_config: &wgpu::SurfaceConfiguration,
        depth_stencil: &wgpu::DepthStencilState,
        model_manager: &mut crate::ModelManager,
    ) -> Renderable {
        let terrain_mat = "ground";

        let mat_shader = "v_normal.wgsl";
        let vec3_zero = [0.0; 3];
        let mat_key = CacheKey::from(terrain_mat);
        let mat_asset = MaterialAsset {
            name: terrain_mat.to_string(),
            key: mat_key,
            shader: mat_shader.to_string(),
            ambient: vec3_zero,
            diffuse: vec3_zero,
            specular: vec3_zero,
            shininess: 32.0,
            diffuse_texture: Some("cube-diffuse.jpg".to_string()),
            normal_texture: Some("cube-normal.png".to_string()),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Front),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(depth_stencil.clone()),
            color_target: wgpu::ColorTargetState {
                format: surface_config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::all(),
            },
            bind_group_layouts: vec![
                crate::BindGroupLayouts::uniform().clone(),
                crate::BindGroupLayouts::equirect_dst().clone(),
                crate::BindGroupLayouts::material_storage().clone(),
                crate::BindGroupLayouts::normal().clone(),
            ],
        };

        let (pipeline, bind_group) = mat_asset
            .load_asset(
                &model_manager.queue,
                &model_manager.device,
                &mut model_manager.materials.textures,
                &mut model_manager.materials.shaders,
                &mut model_manager.materials.pipelines,
                surface_config,
                &[Vertex::LAYOUT, VertexInstance::LAYOUT],
            )
            .expect("Failed to load terrain material");

        let mat = if let Some(cached_mat) = model_manager.materials.materials.get(&mat_asset.key) {
            cached_mat.clone()
        } else {
            Arc::new(Material {
                asset: mat_asset,
                bind_group,
                pipeline,
                idx: model_manager.materials.materials.len() as u32,
            })
        };

        self.mesh_instances.clear();

        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let pos = (center.x as i32 + dx, 0, center.z as i32 + dz);
                let chunk = Chunk::flat(pos);
                let mesh_asset = chunk.build_chunk_mesh();
                let mesh = Mesh::from_asset(
                    &model_manager.queue,
                    &model_manager.device,
                    mesh_asset,
                    &format!("chunk_{:?}", pos),
                );
                let mesh_instance = MeshInstance {
                    mesh: Arc::new(mesh),
                    material: Some(mat.clone()),
                };
                self.insert_chunk(chunk);
                self.mesh_instances.push(mesh_instance);
            }
        }
        let renderable = Renderable::new(terrain_mat.into());

        model_manager.materials.update_storage(&mat);
        renderable
    }
    pub fn mesh_instances(&self) -> &[MeshInstance] {
        &self.mesh_instances
    }
}
