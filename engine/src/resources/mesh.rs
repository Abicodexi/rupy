use std::sync::Arc;

use crate::{Vertex, WgpuBuffer};

use super::Material;

/// Raw vertex & index arrays off‐CPU.
#[derive(Clone, Debug)]
pub struct MeshAsset {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
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
            let vb = crate::WgpuBuffer::from_data(
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
            let ib = crate::WgpuBuffer::from_data(
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
    pub fn compute_vertex(m: &tobj::Model) -> Vec<Vertex> {
        use std::iter::repeat;

        let mesh = &m.mesh;

        // Initial per-vertex list, zipping pos, uv, normal, color
        let mut vertices: Vec<Vertex> = {
            let positions = mesh.positions.chunks(3);
            let uvs = mesh.texcoords.chunks(2).chain(repeat(&[0.0, 0.0][..]));
            let norms = mesh.normals.chunks(3).chain(repeat(&[0.0, 0.0, 1.0][..]));
            let cols = mesh
                .vertex_color
                .chunks(3)
                .chain(repeat(&[1.0, 1.0, 1.0][..]));

            positions
                .zip(uvs)
                .zip(norms)
                .zip(cols)
                .map(|(((pos, uv), nrm), col)| Vertex {
                    position: [pos[0], pos[1], pos[2]],
                    tex_coords: [uv[0], uv[1]],
                    normal: [nrm[0], nrm[1], nrm[2]],
                    tangent: [0.0; 3],
                    color: [col[0], col[1], col[2]],
                })
                .collect()
        };

        // Accumulators
        let mut accum_normals = vec![[0.0f32; 3]; vertices.len()];
        let mut accum_tangents = vec![[0.0f32; 3]; vertices.len()];
        let mut accum_bitangents = vec![[0.0f32; 3]; vertices.len()];

        // Accumulate
        for idx in mesh.indices.chunks(3) {
            let [i0, i1, i2] = [idx[0] as usize, idx[1] as usize, idx[2] as usize];

            let v0 = vertices[i0].position;
            let v1 = vertices[i1].position;
            let v2 = vertices[i2].position;

            let uv0 = vertices[i0].tex_coords;
            let uv1 = vertices[i1].tex_coords;
            let uv2 = vertices[i2].tex_coords;

            // compute edges & UV deltas
            let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
            let duv1 = [uv1[0] - uv0[0], uv1[1] - uv0[1]];
            let duv2 = [uv2[0] - uv0[0], uv2[1] - uv0[1]];
            let r = 1.0 / (duv1[0] * duv2[1] - duv1[1] * duv2[0]);

            // tangent & bitangent
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

            // face normal
            let n_unnorm = [
                edge1[1] * edge2[2] - edge1[2] * edge2[1],
                edge1[2] * edge2[0] - edge1[0] * edge2[2],
                edge1[0] * edge2[1] - edge1[1] * edge2[0],
            ];
            let len =
                (n_unnorm[0] * n_unnorm[0] + n_unnorm[1] * n_unnorm[1] + n_unnorm[2] * n_unnorm[2])
                    .sqrt()
                    .max(1e-6);
            let normal = [n_unnorm[0] / len, n_unnorm[1] / len, n_unnorm[2] / len];

            // corners
            for &i in &[i0, i1, i2] {
                accum_normals[i]
                    .iter_mut()
                    .zip(normal.iter())
                    .for_each(|(a, &b)| *a += b);
                accum_tangents[i]
                    .iter_mut()
                    .zip(tangent.iter())
                    .for_each(|(a, &b)| *a += b);
                accum_bitangents[i]
                    .iter_mut()
                    .zip(bitangent.iter())
                    .for_each(|(a, &b)| *a += b);
            }
        }

        // Normalize + orthogonalize
        for (i, v) in vertices.iter_mut().enumerate() {
            // normalize normal
            let n = {
                let nn = accum_normals[i];
                let l = (nn[0] * nn[0] + nn[1] * nn[1] + nn[2] * nn[2])
                    .sqrt()
                    .max(1e-6);
                [nn[0] / l, nn[1] / l, nn[2] / l]
            };
            // orthogonalize tangent to n, then normalize
            let t = {
                let tt = accum_tangents[i];
                let dot = n[0] * tt[0] + n[1] * tt[1] + n[2] * tt[2];
                let ortho = [tt[0] - n[0] * dot, tt[1] - n[1] * dot, tt[2] - n[2] * dot];
                let l = (ortho[0] * ortho[0] + ortho[1] * ortho[1] + ortho[2] * ortho[2])
                    .sqrt()
                    .max(1e-6);
                [ortho[0] / l, ortho[1] / l, ortho[2] / l]
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
