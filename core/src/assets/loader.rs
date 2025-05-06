static BASE_PATH: once_cell::sync::Lazy<std::path::PathBuf> =
    once_cell::sync::Lazy::new(|| super::asset_dir().expect("couldn’t find asset dir"));
pub struct AssetLoader {}
impl AssetLoader {
    pub fn base_path() -> &'static std::path::PathBuf {
        &*BASE_PATH
    }
    pub fn resolve(rel_path: &str) -> std::path::PathBuf {
        AssetLoader::base_path().join(rel_path)
    }

    pub fn load_text(rel_path: &str) -> Result<String, crate::EngineError> {
        let path = AssetLoader::resolve(rel_path);
        std::fs::read_to_string(&path).map_err(|e| {
            crate::EngineError::FileSystemError(format!("Failed to read {:?}: {}", path, e))
        })
    }

    pub fn load_shader(
        device: &wgpu::Device,
        rel_path: &str,
    ) -> Result<wgpu::ShaderModule, crate::EngineError> {
        let path = AssetLoader::base_path().join("shaders").join(rel_path);

        let shader_source = std::fs::read_to_string(&path)?;

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(rel_path),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        Ok(shader_module)
    }
    pub fn read_bytes<P: AsRef<std::path::Path>>(path: &P) -> Result<Vec<u8>, crate::EngineError> {
        let bytes = std::fs::read(path)?;
        Ok(bytes)
    }
    pub async fn load_texture(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        rel_path: &str,
    ) -> Result<crate::Texture, crate::EngineError> {
        let path = AssetLoader::base_path().join("textures").join(rel_path);

        let bytes = Self::read_bytes(&path)?;
        let tex = crate::Texture::from_bytes(device, queue, &bytes, path).await?;
        Ok(tex)
    }
    pub fn load_tobj<P: AsRef<std::path::Path>>(
        obj: P,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        managers: &mut crate::Managers,
        camera: &crate::camera::Camera,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> Result<crate::Model, crate::EngineError> {
        let base_path = crate::asset_dir()?.join("models");
        let obj_path = base_path.join(obj.as_ref());
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
            let mats = AssetLoader::load_tobj_materials(
                queue,
                device,
                managers,
                camera,
                surface_config,
                &vec_m,
            )?;

            let mut instances = Vec::with_capacity(models.len());
            let mut aabb: Option<crate::AABB> = None;
            for m in models {
                let mesh = m.mesh;

                let vertices: Vec<_> = mesh
                    .positions
                    .chunks(3)
                    .zip(
                        mesh.texcoords
                            .chunks(2)
                            .chain(std::iter::repeat(&[0.0, 0.0][..])),
                    )
                    .map(|(pos, uv)| crate::VertexTexture {
                        position: [pos[0], pos[1], pos[2]],
                        color: [1.0, 1.0, 1.0],
                        tex_coords: [uv[0], uv[1]],
                    })
                    .collect();
                let mat_id = mesh.material_id.unwrap_or(0);
                let mat = mats
                    .get(mat_id)
                    .cloned()
                    .unwrap_or_else(|| crate::Material::default());

                instances.push(crate::MeshInstance::new(
                    queue,
                    device,
                    managers,
                    &vertices,
                    &mesh.indices,
                    &mat,
                ));
                if aabb.is_none() {
                    aabb = Some(crate::Model::compute_aabb(&vertices));
                }
            }

            let name = obj
                .as_ref()
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unnamed")
                .to_string();

            return Ok(crate::Model {
                meshes: instances,
                bounding_radius: aabb.unwrap(),
                name,
            });
        }
        Err(crate::EngineError::AssetLoadError(format!(
            "Error loading obj",
        )))
    }

    fn load_tobj_materials(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        managers: &mut crate::Managers,
        camera: &crate::camera::Camera,
        surface_config: &wgpu::SurfaceConfiguration,
        mats: &[tobj::Material],
    ) -> Result<Vec<crate::Material>, crate::EngineError> {
        mats.iter()
            .map(|m| {
                AssetLoader::load_tobj_material(queue, device, managers, camera, surface_config, m)
            })
            .collect()
    }

    fn load_tobj_material(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        managers: &mut crate::Managers,
        camera: &crate::camera::Camera,
        surface_config: &wgpu::SurfaceConfiguration,
        mat: &tobj::Material,
    ) -> Result<crate::Material, crate::EngineError> {
        let mut bind_groups = Vec::new();
        let mut bind_group_layouts = vec![crate::BindGroupLayouts::camera()];
        bind_groups.push(camera.bind_group.clone());

        let base_dir_path = crate::asset_dir()?.join("textures");
        let texture_key = if let Some(diffuse_texture) = &mat.diffuse_texture {
            let tex_path = base_dir_path.join(&diffuse_texture);
            let img = image::open(&tex_path)
                .map_err(|e| {
                    crate::EngineError::AssetLoadError(format!("Texture load failed: {}", e))
                })?
                .to_rgba8();

            let texture = crate::CacheStorage::get_or_create(
                &mut managers.texture_manager,
                crate::CacheKey {
                    id: diffuse_texture.clone(),
                },
                || crate::Texture::from_image(&device, &queue, &img, diffuse_texture).into(),
            );
            let tex_key = crate::CacheKey::from(texture.label.clone());
            Some(tex_key)
        } else {
            None
        };
        if let Some(tex_key) = &texture_key {
            let texture_bind_group_layout = crate::BindGroupLayouts::texture();
            if let Some(bind_group) = managers.texture_manager.bind_group_for(
                &device,
                &tex_key.id,
                &texture_bind_group_layout,
            ) {
                bind_groups.push(bind_group.clone());
                bind_group_layouts.push(texture_bind_group_layout);
            }
        }
        let shader_key = crate::CacheKey::from("v_texture.wgsl");
        let shader = managers
            .shader_manager
            .get_or_create(shader_key.clone(), || {
                let shader_module = AssetLoader::load_shader(&device, &shader_key.id)?;
                Ok(std::sync::Arc::new(shader_module))
            });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{} layout", shader_key.id)),
            bind_group_layouts: &bind_group_layouts,
            push_constant_ranges: &[],
        });

        let _pipeline =
            managers
                .pipeline_manager
                .get_or_create_render_pipeline(shader_key.clone(), || {
                    Ok(device
                        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                            label: Some(&shader_key.id),
                            layout: Some(&pipeline_layout),
                            vertex: wgpu::VertexState {
                                module: &shader,
                                entry_point: Some("vs_main"),
                                buffers: &[
                                    crate::VertexTexture::LAYOUT,
                                    crate::InstanceData::LAYOUT,
                                ],
                                compilation_options: Default::default(),
                            },
                            fragment: Some(wgpu::FragmentState {
                                module: &shader,
                                entry_point: Some("fs_main"),
                                targets: &[Some(wgpu::ColorTargetState {
                                    format: surface_config.format,
                                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                                    write_mask: wgpu::ColorWrites::ALL,
                                })],
                                compilation_options: Default::default(),
                            }),
                            primitive: wgpu::PrimitiveState {
                                topology: wgpu::PrimitiveTopology::TriangleList,
                                strip_index_format: None,
                                front_face: wgpu::FrontFace::Ccw,
                                cull_mode: None,
                                polygon_mode: wgpu::PolygonMode::Fill,
                                unclipped_depth: false,
                                conservative: false,
                            },
                            depth_stencil: Some(
                                managers.texture_manager.depth_stencil_state.clone(),
                            ),

                            multisample: wgpu::MultisampleState {
                                count: 1,
                                mask: !0,
                                alpha_to_coverage_enabled: false,
                            },
                            multiview: None,
                            cache: None,
                        })
                        .into())
                });
        let material = crate::Material {
            name: mat.name.clone(),
            bind_groups,
            front_face: wgpu::FrontFace::Ccw,
            topology: wgpu::PrimitiveTopology::TriangleList,
            shader_key,
            texture_key,
            blend_state: None,
            cull_mode: Some(wgpu::Face::Back),
        };
        managers
            .material_manager
            .materials
            .insert(material.name.clone().into(), material.clone().into());
        Ok(material)
    }
    pub async fn load_model<V: bytemuck::Pod, I: bytemuck::Pod>(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        managers: &mut crate::Managers,
        config: &wgpu::SurfaceConfiguration,
        bind_group_layouts: Vec<&wgpu::BindGroupLayout>,
        bind_groups: Vec<wgpu::BindGroup>,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        model_name: &str,
        material_name: &str,
        shader_rel_path: &str,
        texture_rel_path: Option<&str>,
        texture_bind_group_layout: Option<&wgpu::BindGroupLayout>,
        blend_state: Option<wgpu::BlendState>,
        cull_mode: Option<wgpu::Face>,
        topology: wgpu::PrimitiveTopology,
        front_face: wgpu::FrontFace,
        polygon_mode: wgpu::PolygonMode,
        vertices: &[V],
        indices: &[I],
        aabb: crate::AABB,
    ) -> Result<(), crate::EngineError> {
        use crate::cache::CacheStorage;

        let material = managers
            .material_manager
            .create_material(
                queue,
                device,
                &mut managers.shader_manager,
                &mut managers.texture_manager,
                &mut managers.pipeline_manager,
                &config,
                bind_group_layouts,
                bind_groups,
                buffers,
                &material_name,
                shader_rel_path,
                texture_rel_path,
                texture_bind_group_layout,
                topology,
                front_face,
                polygon_mode,
                blend_state,
                cull_mode,
            )
            .await?;

        let mesh_instance =
            crate::MeshInstance::new(queue, device, managers, vertices, indices, &material);

        managers.model_manager.insert(
            model_name.into(),
            crate::Model {
                meshes: vec![mesh_instance],
                bounding_radius: aabb,
                name: model_name.to_string(),
            }
            .into(),
        );

        Ok(())
    }
}
