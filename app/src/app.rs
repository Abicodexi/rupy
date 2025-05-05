use core::{
    camera::{Camera, CameraController, CameraUniform, Frustum},
    log_error, BindGroupLayouts, BufferManager, CacheKey, CacheStorage, EngineError, Entity,
    Environment, EquirectProjection, GlyphonBuffer, GlyphonRenderer, InstanceData, Managers,
    MaterialManager, MeshManager, Model, ModelManager, PipelineManager, Position, Renderable,
    Renderer, Resources, Rotation, Scale, ShaderManager, SurfaceExt, Texture, TextureManager, Time,
    VertexTexture, WgpuBuffer, WgpuRenderer, World,
};
use std::sync::Arc;

use cgmath::{Deg, Point3, Rotation3, Vector3};
use glyphon::{cosmic_text::LineEnding, Attrs, Shaping};
use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

const VERTICES: [VertexTexture; 5] = [
    // Base (y = 0), CCW winding when viewed from above:
    VertexTexture {
        position: [-1.0, 0.0, -1.0], // corner 0
        color: [1.0, 0.0, 0.0],      // red
        tex_coords: [0.0, 0.0],
    },
    VertexTexture {
        position: [1.0, 0.0, -1.0], // corner 1
        color: [0.0, 1.0, 0.0],     // green
        tex_coords: [1.0, 0.0],
    },
    VertexTexture {
        position: [1.0, 0.0, 1.0], // corner 2
        color: [0.0, 0.0, 1.0],    // blue
        tex_coords: [1.0, 1.0],
    },
    VertexTexture {
        position: [-1.0, 0.0, 1.0], // corner 3
        color: [1.0, 1.0, 0.0],     // yellow
        tex_coords: [0.0, 1.0],
    },
    // Apex, centered above the base:
    VertexTexture {
        position: [0.0, 1.0, 0.0], // corner 4
        color: [1.0, 1.0, 1.0],    // white
        tex_coords: [0.5, 0.5],
    },
];

const INDICES: [u16; 18] = [
    // base (two triangles)
    0, 1, 2, 0, 2, 3, // four side faces
    0, 1, 4, 1, 2, 4, 2, 3, 4, 3, 0, 4,
];

#[allow(dead_code)]
pub struct Rupy<'a> {
    pub resources: Arc<Resources>,
    pub managers: Managers,
    pub world: World,
    pub time: Time,
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'a>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub wgpu_renderer: WgpuRenderer,
    pub glyphon_renderer: GlyphonRenderer,
    pub bind_group_layouts: BindGroupLayouts,
    pub camera: Camera,
    pub controller: CameraController,
    pub last_shape_time: std::time::Instant,
    pub last_transform_time: std::time::Instant,
}

impl<'a> Rupy<'a> {
    pub const TEXT_BUFFER: &'static str = "text_buffer";
    pub async fn new(
        event_loop: &ActiveEventLoop,
        resources: Arc<Resources>,
    ) -> Result<Self, EngineError> {
        let win_attrs = WindowAttributes::default().with_title("RupyEngine");
        let window = Arc::new(event_loop.create_window(win_attrs)?);
        let win_clone = Arc::clone(&window);

        let (width, height) = {
            let inenr_size = window.inner_size();
            (inenr_size.width, inenr_size.height)
        };

        let surface = resources.gpu.instance.create_surface(win_clone)?;
        let mut surface_config = surface
            .get_default_config(&resources.gpu.adapter, width, height)
            .ok_or(EngineError::SurfaceConfigError(
                "surface isn't supported by this adapter".into(),
            ))?;
        surface_config.present_mode = wgpu::PresentMode::Mailbox;
        surface.configure(&resources.gpu.device, &surface_config);

        let time = Time::new();
        let depth_stencil_state = wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };

        let depth_texture = Texture::create(
            &resources.gpu.device,
            wgpu::Extent3d {
                width: surface_config.width,
                height: surface_config.height,
                depth_or_array_layers: 1,
            },
            Texture::DEPTH_FORMAT,
            1,
            wgpu::TextureViewDimension::D2,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            Some(wgpu::AddressMode::ClampToEdge),
            wgpu::FilterMode::Linear,
            Some(
                resources
                    .gpu
                    .device
                    .create_sampler(&wgpu::SamplerDescriptor {
                        address_mode_u: wgpu::AddressMode::ClampToEdge,
                        address_mode_v: wgpu::AddressMode::ClampToEdge,
                        address_mode_w: wgpu::AddressMode::ClampToEdge,
                        mag_filter: wgpu::FilterMode::Linear,
                        min_filter: wgpu::FilterMode::Linear,
                        mipmap_filter: wgpu::FilterMode::Nearest,
                        compare: Some(wgpu::CompareFunction::LessEqual),
                        lod_min_clamp: 0.0,
                        lod_max_clamp: 100.0,
                        ..Default::default()
                    }),
            ),
            Some("Depth texture"),
        );
        let shader_manager = ShaderManager::new();
        let texture_manager = TextureManager::new(depth_stencil_state, depth_texture);
        let pipeline_manager = PipelineManager::new();
        let buffer_manager = BufferManager::new();
        let mesh_manager = MeshManager::new();
        let material_manager = MaterialManager::new();
        let model_manager = ModelManager::new();
        let mut managers = Managers {
            shader_manager,
            pipeline_manager,
            buffer_manager,
            texture_manager,
            mesh_manager,
            material_manager,
            model_manager,
        };
        let bind_group_layouts = BindGroupLayouts::new(&resources.gpu.device);
        let camera_uniform = CameraUniform::new();
        let camera_uniform_cache_key = CacheKey::new("camera_uniform_buffer");
        let camera_bind_group =
            resources
                .gpu
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("camera_bg"),
                    layout: &bind_group_layouts.camera,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: managers
                            .buffer_manager
                            .w_buffer
                            .get_or_create(camera_uniform_cache_key.clone(), || {
                                WgpuBuffer::from_data(
                                    &resources.gpu.queue,
                                    &resources.gpu.device,
                                    &[camera_uniform],
                                    wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                                    Some(&format!("camera uniform buffer")),
                                )
                                .into()
                            })
                            .buffer
                            .as_entire_binding(),
                    }],
                });
        let camera = Camera {
            eye: Point3::new(0.0, 1.0, 2.0),
            target: Point3::new(0.0, 0.0, 0.0),
            up: Vector3::unit_y(),
            aspect: width as f32 / height as f32,
            fovy: Deg(45.0),
            znear: 0.1,
            zfar: 1000.0,
            uniform: camera_uniform,
            frustum: Frustum::new(),
            bind_group: camera_bind_group.clone(),
            uniform_cache_key: camera_uniform_cache_key,
        };
        let controller = CameraController::new(1.0, 0.5);

        let equirect_projection = EquirectProjection::new(
            &resources,
            &mut managers,
            &surface_config,
            &bind_group_layouts,
            "equirect_src.wgsl",
            "equirect_dst.wgsl",
            "pure-sky.hdr",
            1080,
            wgpu::TextureFormat::Rgba32Float,
        )?;

        let mut world = World::new(Environment {
            equirect_projection,
        });
        world.compute_equirect_projection(
            resources.gpu.queue(),
            resources.gpu.device(),
            &mut managers,
            &bind_group_layouts,
        );
        let wgpu_renderer = WgpuRenderer::new();
        let glyphon_renderer = GlyphonRenderer::new(
            &resources.gpu.device,
            &resources.gpu.queue,
            surface_config.format,
            &managers.texture_manager.depth_stencil_state,
        );
        let triangle_key = CacheKey::from("triangle");
        let entity = world.spawn();
        world.insert_position(entity, Position { x: 1.0, y: 1.0 });
        let angle_deg = 00.0 % 360.0;
        let angle = Deg(angle_deg);
        world.insert_rotation(
            entity,
            Rotation {
                quat: cgmath::Quaternion::from_angle_z(angle),
            },
        );
        world.insert_scale(
            entity,
            Scale {
                value: cgmath::Vector3 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                },
            },
        );
        world.insert_renderable(
            entity,
            Renderable {
                model_key: "2d_triangle".into(),
                visible: true,
            },
        );

        let grid_width = 50;
        let spacing = 2.0;

        for idx in 0..50 {
            let ent = world.spawn();
            let x = (idx % grid_width) as f32 * spacing;
            let y = (idx / grid_width) as f32 * spacing;

            world.insert_position(ent, Position { x, y });

            let angle_deg = (0.0 * idx as f32) % 360.0;
            let angle = Deg(angle_deg);
            world.insert_rotation(
                ent,
                Rotation {
                    quat: cgmath::Quaternion::from_angle_z(angle),
                },
            );
            world.insert_scale(
                ent,
                Scale {
                    value: cgmath::Vector3 {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    },
                },
            );
            world.update_transforms(time.delta_time as f64, camera.frustum);
            if let Some(t) = world.get_transform(Entity(idx)) {
                world.add_instance_to_batch(entity, Entity(idx), *t);
            }
        }

        // END TEST
        let abb = Model::compute_aabb(&VERTICES);
        resources
            .asset_loader
            .load_model(
                &resources,
                &mut managers,
                &surface_config,
                vec![&bind_group_layouts.camera],
                vec![camera_bind_group],
                &[VertexTexture::LAYOUT, InstanceData::LAYOUT],
                "2d_triangle",
                &triangle_key.id,
                "v_texture.wgsl",
                Some("cube-diffuse.jpg"),
                Some(&bind_group_layouts.texture),
                None,
                None,
                wgpu::PrimitiveTopology::TriangleList,
                wgpu::FrontFace::Ccw,
                wgpu::PolygonMode::Fill,
                &VERTICES,
                &INDICES,
                abb,
            )
            .await?;

        Ok(Rupy {
            managers,
            resources,
            world,
            time,
            window,
            surface,
            surface_config,
            wgpu_renderer,
            glyphon_renderer,
            bind_group_layouts,
            camera,
            controller,
            last_shape_time: std::time::Instant::now(),
            last_transform_time: std::time::Instant::now(),
        })
    }

    pub fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        self.surface.resize(
            self.resources.gpu.device(),
            &mut self.surface_config,
            *new_size,
        );
        self.glyphon_renderer.resize(
            &self.resources.gpu.queue,
            glyphon::Resolution {
                width: self.surface_config.width,
                height: self.surface_config.height,
            },
        );
        self.managers.texture_manager.depth_texture = Texture::create(
            &self.resources.gpu.device,
            wgpu::Extent3d {
                width: self.surface_config.width,
                height: self.surface_config.height,
                depth_or_array_layers: 1,
            },
            Texture::DEPTH_FORMAT,
            1,
            wgpu::TextureViewDimension::D2,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            Some(wgpu::AddressMode::ClampToEdge),
            wgpu::FilterMode::Linear,
            Some(
                self.resources
                    .gpu
                    .device
                    .create_sampler(&wgpu::SamplerDescriptor {
                        address_mode_u: wgpu::AddressMode::ClampToEdge,
                        address_mode_v: wgpu::AddressMode::ClampToEdge,
                        address_mode_w: wgpu::AddressMode::ClampToEdge,
                        mag_filter: wgpu::FilterMode::Linear,
                        min_filter: wgpu::FilterMode::Linear,
                        mipmap_filter: wgpu::FilterMode::Nearest,
                        compare: Some(wgpu::CompareFunction::LessEqual),
                        lod_min_clamp: 0.0,
                        lod_max_clamp: 100.0,
                        ..Default::default()
                    }),
            ),
            Some("Depth texture"),
        )
    }

    pub fn draw(&mut self) {
        match self.surface.texture() {
            Ok(frame) => {
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = self.resources.gpu.device.create_command_encoder(
                    &wgpu::CommandEncoderDescriptor {
                        label: Some("render encoder"),
                    },
                );
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("main pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &self.managers.texture_manager.depth_texture.view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    self.wgpu_renderer.render(
                        &self.resources.gpu.queue,
                        &self.resources.gpu.device,
                        &mut self.managers,
                        &mut rpass,
                        &self.bind_group_layouts,
                        &mut self.world,
                        &self.camera,
                    );
                    self.glyphon_renderer.render(
                        &self.resources.gpu.queue,
                        &self.resources.gpu.device,
                        &mut self.managers,
                        &mut rpass,
                        &self.bind_group_layouts,
                        &mut self.world,
                        &self.camera,
                    );
                }

                self.resources.gpu.queue.submit(Some(encoder.finish()));
                frame.present();
            }
            Err(e) => {
                log_error!("SurfaceError: {}", e);
            }
        };
    }

    pub fn update(&mut self) {
        self.time.update();
        const TRANSFORM_UPDATE_INTERVAL_MS: u128 = 16; // ~60Hz
        self.camera.update(&mut self.controller);
        let camera_uniform_buffer = self.managers.buffer_manager.w_buffer.get_or_create(
            self.camera.uniform_cache_key.clone(),
            || {
                WgpuBuffer::from_data(
                    self.resources.gpu.queue(),
                    self.resources.gpu.device(),
                    &[self.camera.uniform],
                    wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    Some(&format!("camera uniform buffer")),
                )
                .into()
            },
        );
        camera_uniform_buffer.write_data(self.resources.gpu.queue(), &[self.camera.uniform], None);

        if self.last_transform_time.elapsed().as_millis() > TRANSFORM_UPDATE_INTERVAL_MS {
            self.world
                .update_transforms(self.time.delta_time as f64, self.camera.frustum);
            self.last_transform_time = std::time::Instant::now();
        }

        if self.last_shape_time.elapsed().as_millis() > 1000 {
            let text_buffer = self.managers.buffer_manager.g_buffer.get_or_create(
                CacheKey::new(Self::TEXT_BUFFER),
                || {
                    let metrics = glyphon::Metrics {
                        font_size: 20.0,
                        line_height: 20.0,
                    };
                    GlyphonBuffer::new(&mut self.glyphon_renderer.font_system, Some(metrics)).into()
                },
            );

            let line_ending = LineEnding::LfCr;
            let attrs_list = glyphon::AttrsList::new(Attrs::new());
            let shaping = Shaping::Advanced;
            text_buffer.clear_buffer_lines();

            #[cfg(debug_assertions)]
            {
                let debug_buffer_lines = vec![
                    glyphon::BufferLine::new(
                        format!(
                            "Eye: x: {:.2} y: {:.2} z: {:.2}",
                            self.camera.eye.x, self.camera.eye.y, self.camera.eye.z
                        ),
                        line_ending,
                        attrs_list.clone(),
                        shaping,
                    ),
                    glyphon::BufferLine::new(
                        format!(
                            "Target: x: {:.2} y: {:.2} z: {:.2}",
                            self.camera.target.x, self.camera.target.y, self.camera.target.z
                        ),
                        line_ending,
                        attrs_list.clone(),
                        shaping,
                    ),
                    glyphon::BufferLine::new(
                        format!(
                            "yaw: {:.2} pitch: {:.2}",
                            self.controller.yaw, self.controller.pitch
                        ),
                        line_ending,
                        attrs_list.clone(),
                        shaping,
                    ),
                ];

                text_buffer.set_buffer_lines(debug_buffer_lines);
            }
            text_buffer.push_buffer_line(glyphon::BufferLine::new(
                format!("fps: {:.1} dt: {:.4}", self.time.fps, self.time.delta_time),
                line_ending,
                attrs_list,
                shaping,
            ));
            text_buffer
                .buffer
                .shape_until_scroll(&mut self.glyphon_renderer.font_system, false);
            self.last_shape_time = std::time::Instant::now();
            self.glyphon_renderer.prepare(
                &self.resources.gpu.device,
                &self.resources.gpu.queue,
                text_buffer,
                &self.surface_config,
            );
        }
    }
}
