use super::Material;
use crate::{gfx::buffer::WgpuBuffer, Vertex };
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct MeshAsset {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}
impl MeshAsset {
    pub fn load_asset(
        &self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        label: &str,
    ) -> (WgpuBuffer, WgpuBuffer, u32) {
        let vertex_buffer = {
            let data: &[u8] = bytemuck::cast_slice(&self.vertices);
            let vb = WgpuBuffer::from_data(
                device,
                data,
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                Some(&format!("{}_vertex_buffer", label)),
            );
            queue.write_buffer(vb.get(), 0, data);
            vb
        };

        let index_buffer = {
            let data: &[u8] = bytemuck::cast_slice(&self.indices);
            let ib = WgpuBuffer::from_data(
                device,
                data,
                wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                Some(&format!("{}_index_buffer", label)),
            );
            queue.write_buffer(ib.get(), 0, data);
            ib
        };

        let index_count = self.indices.len() as u32;

        (vertex_buffer, index_buffer, index_count)
    }
    pub fn compute_vertex(
        positions: &[[f32; 3]],
        normals: &[[f32; 3]],
        tex_coords: &[[f32; 2]],
        indices: &[u16],
        vertex_colors: Option<&[[f32; 3]]>,
    ) -> Vec<Vertex> {
        let num_vertices = positions.len();
        let mut vertices = Vec::with_capacity(num_vertices);

        // Build initial vertex buffer with placeholder tangents
        for i in 0..num_vertices {
            vertices.push(Vertex {
                position: positions[i],
                normal: *normals.get(i).unwrap_or(&[0.0, 0.0, 1.0]),
                tex_coords: *tex_coords.get(i).unwrap_or(&[0.0, 0.0]),
                tangent: [0.0; 3],
                color: vertex_colors
                    .and_then(|c| c.get(i).copied())
                    .unwrap_or([1.0, 1.0, 1.0]),
            });
        }

        // Initialize tangent accumulators
        let mut accum_normals = vec![[0.0f32; 3]; num_vertices];
        let mut accum_tangents = vec![[0.0f32; 3]; num_vertices];
        let mut accum_bitangents = vec![[0.0f32; 3]; num_vertices];

        for tri in indices.chunks_exact(3) {
            let [i0, i1, i2] = [tri[0] as usize, tri[1] as usize, tri[2] as usize];

            let v0 = vertices[i0].position;
            let v1 = vertices[i1].position;
            let v2 = vertices[i2].position;

            let uv0 = vertices[i0].tex_coords;
            let uv1 = vertices[i1].tex_coords;
            let uv2 = vertices[i2].tex_coords;

            let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
            let duv1 = [uv1[0] - uv0[0], uv1[1] - uv0[1]];
            let duv2 = [uv2[0] - uv0[0], uv2[1] - uv0[1]];

            let r = 1.0 / (duv1[0] * duv2[1] - duv1[1] * duv2[0]).max(1e-6);

            let tangent = [
                r * (duv2[1] * edge1[0] - duv1[1] * edge2[0]),
                r * (duv2[1] * edge1[1] - duv1[1] * edge2[1]),
                r * (duv2[1] * edge1[2] - duv1[1] * edge2[2]),
            ];
            let bitangent = [
                r * (-duv2[0] * edge1[0] + duv1[0] * edge2[0]),
                r * (-duv2[0] * edge1[1] + duv1[0] * edge2[1]),
                r * (-duv2[0] * edge1[2] + duv1[0] * edge2[2]),
            ];

            let face_normal = {
                let n = [
                    edge1[1] * edge2[2] - edge1[2] * edge2[1],
                    edge1[2] * edge2[0] - edge1[0] * edge2[2],
                    edge1[0] * edge2[1] - edge1[1] * edge2[0],
                ];
                let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-6);
                [n[0] / len, n[1] / len, n[2] / len]
            };

            for &i in &[i0, i1, i2] {
                accum_normals[i]
                    .iter_mut()
                    .zip(face_normal)
                    .for_each(|(a, b)| *a += b);
                accum_tangents[i]
                    .iter_mut()
                    .zip(tangent)
                    .for_each(|(a, b)| *a += b);
                accum_bitangents[i]
                    .iter_mut()
                    .zip(bitangent)
                    .for_each(|(a, b)| *a += b);
            }
        }

        // Normalize and orthogonalize
        for (i, v) in vertices.iter_mut().enumerate() {
            let n = {
                let n = accum_normals[i];
                let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-6);
                [n[0] / len, n[1] / len, n[2] / len]
            };

            let t = {
                let t = accum_tangents[i];
                let dot = n[0] * t[0] + n[1] * t[1] + n[2] * t[2];
                let ortho = [t[0] - n[0] * dot, t[1] - n[1] * dot, t[2] - n[2] * dot];
                let len = (ortho[0] * ortho[0] + ortho[1] * ortho[1] + ortho[2] * ortho[2])
                    .sqrt()
                    .max(1e-6);
                [ortho[0] / len, ortho[1] / len, ortho[2] / len]
            };

            v.normal = n;
            v.tangent = t;
        }

        vertices
    }
}
#[derive(Debug)]
pub struct Mesh {
    pub vertex_buffer: std::sync::Arc<WgpuBuffer>,
    pub index_buffer: std::sync::Arc<WgpuBuffer>,
    pub index_count: u32,
}

impl Mesh {
    pub fn from_asset(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        asset: MeshAsset,
        label: &str,
    ) -> Self {
        let (vertex_buffer, index_buffer, index_count) = asset.load_asset(queue, device, label);
        Self {
            vertex_buffer: Arc::new(vertex_buffer),
            index_buffer: Arc::new(index_buffer),
            index_count,
        }
    }
}
#[derive(Debug)]
pub struct MeshInstance {
    pub mesh: std::sync::Arc<Mesh>,
    pub material: Option<std::sync::Arc<Material>>,
}
