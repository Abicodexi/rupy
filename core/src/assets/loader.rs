use std::sync::Arc;

use crate::CacheStorage;

static BASE_PATH: once_cell::sync::Lazy<std::path::PathBuf> =
    once_cell::sync::Lazy::new(|| super::asset_dir().expect("couldn’t find asset dir"));

pub struct Asset;
impl Asset {
    pub fn base_path() -> &'static std::path::PathBuf {
        &*BASE_PATH
    }
    pub fn resolve(rel_path: &str) -> std::path::PathBuf {
        Asset::base_path().join(rel_path)
    }

    pub fn read_text(rel_path: &str) -> Result<String, crate::EngineError> {
        let path = Asset::resolve(rel_path);
        std::fs::read_to_string(&path)
            .map_err(|e| crate::EngineError::FileSystemError(format!("{:?}: {}", path, e)))
    }

    pub fn shader(managers: &mut crate::Managers, file: &str) -> Result<std::sync::Arc<wgpu::ShaderModule>, crate::EngineError> {
        managers.shader_manager.load(&managers.device, file)
    }
    pub fn read_bytes<P: AsRef<std::path::Path>>(path: &P) -> Result<Vec<u8>, crate::EngineError> {
        let bytes = std::fs::read(path)?;
        Ok(bytes)
    }
    pub async fn texture(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        managers: &mut crate::Managers,
        surface_config: &wgpu::SurfaceConfiguration,
        file: &str,
    ) -> Result<std::sync::Arc<crate::Texture>, crate::EngineError> {
        let cache_key = crate::CacheKey::from(file);
        if !managers.texture_manager.contains(&cache_key) {
            let img = image::open(&file)
                .map_err(|e| crate::EngineError::AssetLoadError(format!("{}: {}", file, e)))?
                .to_rgba8();
            let tex = crate::Texture::from_image(device, queue, surface_config, &img, file);
            managers
                .texture_manager
                .insert(cache_key.clone(), tex.into());
        }

        Ok(managers.texture_manager.get(cache_key).unwrap())
    }
    pub fn tobj<P: AsRef<std::path::Path> + std::fmt::Debug>(
        obj: P,
        managers: &mut crate::Managers,
        surface_config: &wgpu::SurfaceConfiguration,
        depth_stencil_state: &Option<wgpu::DepthStencilState>,
    ) -> Result<Arc<crate::Model>, crate::EngineError> {
        let base_path = crate::asset_dir()?.join("models");
        let obj_path = base_path.join(obj.as_ref());
        let name = obj
            .as_ref()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed")
            .to_string();

        let model_cache_key = crate::CacheKey::from(name.clone());

        if let Some(cached_model) =
            crate::CacheStorage::get(&managers.model_manager, &model_cache_key)
        {
            return Ok(cached_model.clone());
        }

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
                "Error loading obj {} materials: {}",
                name, e
            )));
        }
        if let Ok(vec_m) = materials {
            let mats = crate::MaterialManager::load_tobj_materials(
                managers,
                surface_config,
                depth_stencil_state,
                &vec_m,
                "v_normal.wgsl",
                &[
                    crate::VertexNormal::LAYOUT,
                    crate::VertexNormalInstance::LAYOUT,
                ],
            );

            let mut instances = Vec::with_capacity(models.len() + 1);
            let mut aabb: Option<crate::AABB> = None;
            for m in models {
                let mesh = m.mesh;

                // 1) build base vertices with zeros
                let mut vertices: Vec<crate::VertexNormal> = mesh
                    .positions
                    .chunks(3)
                    .zip(
                        mesh.texcoords
                            .chunks(2)
                            .chain(std::iter::repeat(&[0.0, 0.0][..])),
                    )
                    .map(|(pos, uv)| crate::VertexNormal {
                        position: [pos[0], pos[1], pos[2]],
                        tex_coords: [uv[0], uv[1]],
                        normal: [0.0; 3],
                        tangent: [0.0; 3],
                        bitangent: [0.0; 3],
                    })
                    .collect();

                // 2) prepare accumulators
                let mut accum_normals = vec![[0.0f32; 3]; vertices.len()];
                let mut accum_tangents = vec![[0.0f32; 3]; vertices.len()];
                let mut accum_bitangents = vec![[0.0f32; 3]; vertices.len()];

                // 3) process each triangle
                for idx in mesh.indices.chunks(3) {
                    let i0 = idx[0] as usize;
                    let i1 = idx[1] as usize;
                    let i2 = idx[2] as usize;

                    let v0 = vertices[i0].position;
                    let v1 = vertices[i1].position;
                    let v2 = vertices[i2].position;

                    let uv0 = vertices[i0].tex_coords;
                    let uv1 = vertices[i1].tex_coords;
                    let uv2 = vertices[i2].tex_coords;

                    // edges in 3D
                    let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
                    let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

                    // UV deltas
                    let delta_uv1 = [uv1[0] - uv0[0], uv1[1] - uv0[1]];
                    let delta_uv2 = [uv2[0] - uv0[0], uv2[1] - uv0[1]];
                    let r = 1.0 / (delta_uv1[0] * delta_uv2[1] - delta_uv1[1] * delta_uv2[0]);

                    // tangent & bitangent
                    let tangent = [
                        r * (delta_uv2[1] * edge1[0] - delta_uv1[1] * edge2[0]),
                        r * (delta_uv2[1] * edge1[1] - delta_uv1[1] * edge2[1]),
                        r * (delta_uv2[1] * edge1[2] - delta_uv1[1] * edge2[2]),
                    ];
                    let bitangent = [
                        r * (-delta_uv2[0] * edge1[0] + delta_uv1[0] * edge2[0]),
                        r * (-delta_uv2[0] * edge1[1] + delta_uv1[0] * edge2[1]),
                        r * (-delta_uv2[0] * edge1[2] + delta_uv1[0] * edge2[2]),
                    ];

                    // face normal = normalize(cross(edge1, edge2))
                    let n = {
                        let n_unnorm = [
                            edge1[1] * edge2[2] - edge1[2] * edge2[1],
                            edge1[2] * edge2[0] - edge1[0] * edge2[2],
                            edge1[0] * edge2[1] - edge1[1] * edge2[0],
                        ];
                        let len = (n_unnorm[0] * n_unnorm[0]
                            + n_unnorm[1] * n_unnorm[1]
                            + n_unnorm[2] * n_unnorm[2])
                            .sqrt()
                            .max(1e-6);
                        [n_unnorm[0] / len, n_unnorm[1] / len, n_unnorm[2] / len]
                    };

                    // accumulate into each corner
                    for &i in &[i0, i1, i2] {
                        for j in 0..3 {
                            accum_normals[i][j] += n[j];
                            accum_tangents[i][j] += tangent[j];
                            accum_bitangents[i][j] += bitangent[j];
                        }
                    }
                }

                // 4) normalize and orthogonalize per‐vertex
                for i in 0..vertices.len() {
                    // normalize normal
                    let n = {
                        let nn = accum_normals[i];
                        let len = (nn[0] * nn[0] + nn[1] * nn[1] + nn[2] * nn[2])
                            .sqrt()
                            .max(1e-6);
                        [nn[0] / len, nn[1] / len, nn[2] / len]
                    };

                    let t = {
                        let tt = accum_tangents[i];
                        // remove component along n
                        let dot = n[0] * tt[0] + n[1] * tt[1] + n[2] * tt[2];
                        let tg = [tt[0] - n[0] * dot, tt[1] - n[1] * dot, tt[2] - n[2] * dot];
                        let len = (tg[0] * tg[0] + tg[1] * tg[1] + tg[2] * tg[2])
                            .sqrt()
                            .max(1e-6);
                        [tg[0] / len, tg[1] / len, tg[2] / len]
                    };
                    // bitangent = cross(n, t)
                    let b = [
                        n[1] * t[2] - n[2] * t[1],
                        n[2] * t[0] - n[0] * t[2],
                        n[0] * t[1] - n[1] * t[0],
                    ];

                    vertices[i].normal = n;
                    vertices[i].tangent = t;
                    vertices[i].bitangent = b;
                }

                if let Some(mat) = mats.get(mesh.material_id.unwrap_or(0)) {
                    instances.push(crate::MeshInstance::new(
                        managers,
                        &vertices,
                        &mesh.indices,
                        &mat,
                    ));
                }

                if aabb.is_none() {
                    aabb = Some(crate::Model::compute_aabb(&vertices));
                }
            }
            return Ok(crate::CacheStorage::get_or_create(
                &mut managers.model_manager,
                crate::CacheKey::from(name.as_ref()),
                || {
                    crate::Model {
                        meshes: instances,
                        bounding_radius: aabb.unwrap(),
                        name,
                    }
                    .into()
                },
            )
            .clone());
        }
        Err(crate::EngineError::AssetLoadError(format!(
            "Error loading obj {}",
            name,
        )))
    }

    pub async fn model<V: bytemuck::Pod, I: bytemuck::Pod>(
        managers: &mut crate::Managers,
        surface_config: &wgpu::SurfaceConfiguration,
        depth_stencil_state: &Option<wgpu::DepthStencilState>,
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        model_name: &str,
        material_name: &str,
        shader_rel_path: &str,
        diffuse_texture: Option<&str>,
        normal_texture: Option<&str>,
        blend_state: Option<wgpu::BlendState>,
        cull_mode: Option<wgpu::Face>,
        topology: wgpu::PrimitiveTopology,
        front_face: wgpu::FrontFace,
        polygon_mode: wgpu::PolygonMode,
        vertices: &[V],
        indices: &[I],
        aabb: crate::AABB,
    ) -> Result<std::sync::Arc<crate::Model>, crate::EngineError> {
        let mat_key = crate::CacheKey::from(material_name);
        if !managers.material_manager.contains(&mat_key) {
            let _ = crate::MaterialManager::create(
                managers,
                surface_config,
                depth_stencil_state,
                &crate::MaterialDescriptor {
                    name: material_name,
                    key: mat_key,
                    shader_path: shader_rel_path,
                    diffuse_texture,
                    normal_texture,
                    bind_group_layouts,
                    front_face,
                    topology,
                    polygon_mode,
                    blend_state,
                    cull_mode,
                },
                &[
                    crate::VertexNormal::LAYOUT,
                    crate::VertexNormalInstance::LAYOUT,
                ],
            );
        }
        let model_cache_key = crate::CacheKey::from(model_name);

        if let Some(cached_model) =
            crate::CacheStorage::get(&mut managers.model_manager, &model_cache_key)
        {
            return Ok(cached_model.clone());
        };

        let material = managers.material_manager.get(&mat_key).unwrap();

        let mesh_instance = {
            if let Some(cached_mesh) = crate::CacheStorage::get(&managers.mesh_manager, &mat_key) {
                crate::MeshInstance {
                    vertex_buffer_key: cached_mesh.vertex_buffer_key,
                    index_buffer_key: cached_mesh.index_buffer_key,
                    material_key: mat_key,
                    index_count: cached_mesh.index_count
                }
            } else {
                let vertex_buffer_key = crate::CacheKey::from(format!("{}:vertex", mat_key.id()));
                let index_buffer_key = crate::CacheKey::from(format!("{}:index", mat_key.id()));

                if !crate::CacheStorage::contains(
                    &managers.buffer_manager.w_buffer,
                    &vertex_buffer_key,
                ) {
                    let data = bytemuck::cast_slice(vertices);
                    let vertex_buffer = crate::WgpuBuffer::from_data(
                        &managers.device,
                        data,
                        wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        Some(&format!("{} mesh vertex buffer", material.name)),
                    );
                    managers.queue.write_buffer(vertex_buffer.get(), 0, data);

                    crate::CacheStorage::insert(
                        &mut managers.buffer_manager.w_buffer,
                        vertex_buffer_key.clone(),
                        vertex_buffer.into(),
                    );
                }

                if !crate::CacheStorage::contains(
                    &managers.buffer_manager.w_buffer,
                    &index_buffer_key,
                ) {
                    let data = bytemuck::cast_slice(indices);
                    let index_buffer = crate::WgpuBuffer::from_data(
                        &managers.device,
                        data,
                        wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                        Some(&format!("{} mesh index buffer", material.name)),
                    );
                    managers.queue.write_buffer(index_buffer.get(), 0, data);
                    crate::CacheStorage::insert(
                        &mut managers.buffer_manager.w_buffer,
                        index_buffer_key.clone(),
                        index_buffer.into(),
                    );
                }

                let vertex_count = vertices.len() as u32;
                let index_count = indices.len() as u32;
                crate::CacheStorage::insert(
                    &mut managers.mesh_manager,
                    mat_key,
                    crate::Mesh {
                        vertex_buffer_key: vertex_buffer_key,
                        index_buffer_key: index_buffer_key,
                        vertex_count,
                        index_count,
                    },
                );

                crate::MeshInstance {
                    vertex_buffer_key,
                    index_buffer_key,
                    material_key: mat_key,
                    index_count
                }
            }
        };
        let m = std::sync::Arc::new(crate::Model {
            meshes: vec![mesh_instance],
            bounding_radius: aabb,
            name: model_name.to_string(),
        });
        crate::CacheStorage::insert(&mut managers.model_manager, model_cache_key, m.clone());

        Ok(m)
    }
}
