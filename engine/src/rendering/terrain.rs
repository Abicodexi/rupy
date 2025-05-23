use glam::Vec3;

use crate::{
    chunk::Chunk, log_info, CacheKey, Material, MaterialAsset, Mesh, MeshAsset, MeshInstance,
    Position, RenderBindGroupLayouts, Renderable, Rotation, Scale, Transform, WgpuBuffer, GRAVITY,
};
use std::{collections::HashMap, sync::Arc};

use super::{InstanceBufferData, Vertex, VertexInstance, CHUNK_SIZE};

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
                drag: 0.1,
            },
            Medium::Water => MediumProperties {
                gravity: Vec3::new(0.0, GRAVITY + 7.81, 0.0),
                drag: 0.2,
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
    chunk_stream: HashMap<(i32, i32, i32), (Chunk, Medium)>,
    default_medium: Medium,
    mesh_instances: Vec<MeshInstance>,
    instance_buffer: Option<InstanceBufferData>,
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
        for (chunk, medium) in self.chunk_stream.values_mut() {
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

        for ((cx, cy, cz), _chunk) in &self.chunk_stream {
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
        center: Vec3,
        radius: i32,
        mediums: Vec<Medium>,
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
                RenderBindGroupLayouts::uniform().clone(),
                RenderBindGroupLayouts::equirect_dst().clone(),
                RenderBindGroupLayouts::material_storage().clone(),
                RenderBindGroupLayouts::normal().clone(),
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
                idx: model_manager.materials.storage_count as u32,
            })
        };

        self.mesh_instances.clear();
        let default_medium = self.default_medium.clone();
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let pos = (center.x as i32 + dx, 0, center.z as i32 + dz);
                let medium = *mediums.get(dx.abs() as usize).unwrap_or(&default_medium);

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
                log_info!("Building medium: {:?} at pos: {:?}", medium, pos);
                self.insert_chunk_stream(chunk, medium);
                self.mesh_instances.push(mesh_instance);
            }
        }
        let renderable = Renderable::new(terrain_mat.into());

        renderable
    }
    pub fn mesh_instances(&self) -> &[MeshInstance] {
        &self.mesh_instances
    }
}
