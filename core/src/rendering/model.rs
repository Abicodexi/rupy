use std::sync::Arc;

use pollster::FutureExt;

use crate::{log_info, CacheStorage};

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
    pub async fn from_material<V: bytemuck::Pod, I: bytemuck::Pod>(
        managers: &mut crate::Managers,
        material: crate::Material,
        model_name: &str,
        vertices: &[V],
        indices: &[I],
        aabb: crate::AABB,
    ) -> Result<std::sync::Arc<crate::Model>, crate::EngineError> {
        let model_cache_key = crate::CacheKey {
            id: model_name.into(),
        };
        let material_cache_key = crate::CacheKey {
            id: model_name.into(),
        };

        let model = if let Some(cached_model) =
            crate::CacheStorage::get(&mut managers.model_manager, &model_cache_key)
        {
            cached_model.clone()
        } else {
            let mesh_instance = crate::MeshInstance::new(managers, vertices, indices, &material);
            let m = std::sync::Arc::new(crate::Model {
                meshes: vec![mesh_instance],
                bounding_radius: aabb,
                name: model_name.to_string(),
            });
            crate::CacheStorage::insert(&mut managers.model_manager, model_cache_key, m.clone());
            m
        };

        if !crate::CacheStorage::contains(&mut managers.material_manager, &material_cache_key) {
            crate::CacheStorage::insert(
                &mut managers.material_manager,
                material_cache_key,
                material.into(),
            );
        }

        Ok(model)
    }
    pub fn from_obj<P: AsRef<std::path::Path>>(
        obj: P,
        managers: &mut crate::Managers,
        uniform_bind_group: &wgpu::BindGroup,
        camera: &crate::camera::Camera,
        light: &crate::Light,
        surface_config: &wgpu::SurfaceConfiguration,
        depth_stencil_state: &Option<wgpu::DepthStencilState>,
    ) -> Result<Option<Arc<crate::Model>>, crate::EngineError> {
        let base_path = crate::asset_dir()?.join("models");
        let obj_path = base_path.join(obj.as_ref());
        let name = obj
            .as_ref()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed")
            .to_string();

        let model_cache_key = crate::CacheKey::from(name.clone());

        if let Some(cached_model) = managers.model_manager.get(&model_cache_key) {
            log_info!("Returning cached model: {}", cached_model.name);
            return Ok(Some(cached_model.clone()));
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
                "Error loading obj materials: {}",
                e
            )));
        }
        if let Ok(vec_m) = materials {
            let mats = crate::Material::load_tobj_materials(
                managers,
                uniform_bind_group,
                surface_config,
                depth_stencil_state,
                camera,
                light,
                &vec_m,
                "v_normal.wgsl",
            )?;

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

                // now you have full VertexNormal with correct N/T/B
                let mat_id = mesh.material_id.unwrap_or(0);
                let mat = mats.get(mat_id).cloned().unwrap_or_default();
                instances.push(crate::MeshInstance::new(
                    managers,
                    &vertices,
                    &mesh.indices,
                    &mat,
                ));

                if aabb.is_none() {
                    aabb = Some(crate::Model::compute_aabb(&vertices));
                }
            }

            let model_cache_key = crate::CacheKey { id: name.clone() };
            managers.model_manager.insert(
                model_cache_key.clone(),
                crate::Model {
                    meshes: instances,
                    bounding_radius: aabb.unwrap(),
                    name,
                }
                .into(),
            );

            return Ok(managers
                .model_manager
                .get(&model_cache_key)
                .clone()
                .cloned());
        }
        Err(crate::EngineError::AssetLoadError(format!(
            "Error loading obj",
        )))
    }
    pub fn compute_aabb(vertices: &[crate::VertexNormal]) -> AABB {
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
    pub models: crate::HashCache<std::sync::Arc<Model>>,
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
        render_pipeline_manager: &crate::RenderPipelineManager,
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
                        render_pipeline_manager.get(&material.shader_key)
                    {
                        if let Some(mesh) = <crate::MeshManager as crate::CacheStorage<
                            super::Mesh,
                        >>::get(
                            &mesh_manager, &mesh_instance.material_key
                        ) {
                            let ref_bind_groups: Vec<&std::sync::Arc<wgpu::BindGroup>> =
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
