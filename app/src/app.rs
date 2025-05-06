use cgmath::{Deg, Point3, Vector3};
use core::{
    camera::{Camera, CameraController, CameraUniform, Frustum},
    log_error, BindGroupLayouts, BufferManager, CacheKey, CacheStorage, EngineError,
    EquirectProjection, GlyphonBuffer, GlyphonRenderer, Managers, MaterialManager, MeshManager,
    ModelManager, PipelineManager, Renderer, ShaderManager, SurfaceExt, Texture, TextureManager,
    Time, WgpuBuffer, WgpuRenderer, World, GPU,
};
use glyphon::{cosmic_text::LineEnding, Attrs, Shaping};
use std::sync::Arc;
use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

#[allow(dead_code)]
pub struct Rupy {
    pub managers: Managers,
    pub time: Time,
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub wgpu_renderer: WgpuRenderer,
    pub glyphon_renderer: GlyphonRenderer,
    pub camera: Camera,
    pub controller: CameraController,
    pub last_shape_time: std::time::Instant,
}

impl Rupy {
    pub const TEXT_BUFFER: &'static str = "text_buffer";
    pub fn new(event_loop: &ActiveEventLoop) -> Result<Rupy, EngineError> {
        let binding = GPU::get();
        let gpu = binding.read().unwrap();
        BindGroupLayouts::init(gpu.device());
        let binding = World::get().expect("Start failed, World did not exist");
        let mut world = binding.write().unwrap();
        let win_attrs = WindowAttributes::default().with_title("RupyEngine");
        let window = Arc::new(event_loop.create_window(win_attrs)?);
        let win_clone = Arc::clone(&window);

        let (width, height) = {
            let inenr_size = window.inner_size();
            (inenr_size.width, inenr_size.height)
        };

        let surface = gpu.instance().create_surface(win_clone)?;
        let mut surface_config = surface
            .get_default_config(&gpu.adapter(), width, height)
            .ok_or(EngineError::SurfaceConfigError(
                "surface isn't supported by this adapter".into(),
            ))?;
        surface_config.present_mode = wgpu::PresentMode::Mailbox;
        surface.configure(&gpu.device(), &surface_config);

        let time = Time::new();
        let depth_stencil_state = wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };

        let depth_texture = Texture::create(
            gpu.device(),
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
            Some(gpu.device().create_sampler(&wgpu::SamplerDescriptor {
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
            })),
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

        let camera_uniform = CameraUniform::new();
        let camera_uniform_cache_key = CacheKey::new("camera_uniform_buffer");
        let camera_bind_group = gpu.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bg"),
            layout: &BindGroupLayouts::camera(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: managers
                    .buffer_manager
                    .w_buffer
                    .get_or_create(camera_uniform_cache_key.clone(), || {
                        WgpuBuffer::from_data(
                            gpu.queue(),
                            gpu.device(),
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
            gpu.queue(),
            gpu.device(),
            &mut managers,
            &surface_config,
            "equirect_src.wgsl",
            "equirect_dst.wgsl",
            "pure-sky.hdr",
            1080,
            wgpu::TextureFormat::Rgba32Float,
        )?;
        world.set_projection(equirect_projection);

        let cube_obj = "cube.obj";
        if let Some(model_key) = World::load_object(
            cube_obj,
            gpu.queue(),
            gpu.device(),
            &mut managers,
            &camera,
            &surface_config,
        ) {
            let entity = world.spawn();
            world.insert_position(entity, core::Position { x: 1.0, y: 1.0 });
            let angle_deg = 00.0 % 360.0;
            let angle = Deg(angle_deg);
            world.insert_rotation(
                entity,
                core::Rotation {
                    quat: <cgmath::Quaternion<f32> as cgmath::Rotation3>::from_angle_z(angle),
                },
            );
            world.insert_scale(
                entity,
                core::Scale {
                    value: cgmath::Vector3 {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    },
                },
            );
            world.insert_renderable(
                entity,
                core::Renderable {
                    model_key,
                    visible: true,
                },
            );
        }

        let wgpu_renderer = WgpuRenderer::new();
        let glyphon_renderer = GlyphonRenderer::new(
            gpu.device(),
            gpu.queue(),
            surface_config.format,
            &managers.texture_manager.depth_stencil_state,
        );

        Ok(Rupy {
            managers,
            time,
            window,
            surface,
            surface_config,
            wgpu_renderer,
            glyphon_renderer,
            camera,
            controller,
            last_shape_time: std::time::Instant::now(),
        })
    }

    pub fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        let binding = GPU::get();
        let _ = match binding.read() {
            Ok(gpu) => {
                self.surface
                    .resize(gpu.device(), &mut self.surface_config, *new_size);
                self.glyphon_renderer.resize(
                    gpu.queue(),
                    glyphon::Resolution {
                        width: self.surface_config.width,
                        height: self.surface_config.height,
                    },
                );
                self.managers.texture_manager.depth_texture = Texture::create(
                    gpu.device(),
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
                    Some(gpu.device().create_sampler(&wgpu::SamplerDescriptor {
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
                    })),
                    Some("Depth texture"),
                );
                true
            }
            Err(e) => {
                log_error!("Resize failed, could not acquire gpu lock: {}", e);
                false
            }
        };
    }
    pub fn load_shader(&mut self, rel_path: &str) {
        let binding = GPU::get();
        let _ = match binding.read() {
            Ok(gpu) => {
                self.managers
                    .shader_manager
                    .reload_shader(&gpu.device(), &rel_path);
                self.wgpu_renderer = WgpuRenderer::new();
                true
            }
            Err(e) => {
                log_error!("Shader load failed, could not acquire gpu lock: {}", e);
                false
            }
        };
    }
    pub fn draw(&mut self) {
        let binding = GPU::get();
        match binding.read() {
            Ok(gpu) => match self.surface.texture() {
                Ok(frame) => {
                    if let Some(world) = World::get() {
                        if let Ok(w) = world.read() {
                            let view = frame
                                .texture
                                .create_view(&wgpu::TextureViewDescriptor::default());
                            let mut render_encoder = gpu.device().create_command_encoder(
                                &wgpu::CommandEncoderDescriptor {
                                    label: Some("render encoder"),
                                },
                            );

                            self.wgpu_renderer.compute_pass(
                                gpu.device(),
                                gpu.queue(),
                                &w,
                                &mut self.managers,
                            );
                            {
                                let mut rpass =
                                    render_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                        label: Some("main pass"),
                                        color_attachments: &[Some(
                                            wgpu::RenderPassColorAttachment {
                                                view: &view,
                                                resolve_target: None,
                                                ops: wgpu::Operations {
                                                    load: wgpu::LoadOp::Load,
                                                    store: wgpu::StoreOp::Store,
                                                },
                                            },
                                        )],
                                        depth_stencil_attachment: Some(
                                            wgpu::RenderPassDepthStencilAttachment {
                                                view: &self
                                                    .managers
                                                    .texture_manager
                                                    .depth_texture
                                                    .view,
                                                depth_ops: Some(wgpu::Operations {
                                                    load: wgpu::LoadOp::Clear(1.0),
                                                    store: wgpu::StoreOp::Store,
                                                }),
                                                stencil_ops: None,
                                            },
                                        ),
                                        timestamp_writes: None,
                                        occlusion_query_set: None,
                                    });
                                self.wgpu_renderer.render(
                                    &gpu.queue(),
                                    &gpu.device(),
                                    &mut self.managers,
                                    &mut rpass,
                                    &w,
                                    &self.camera,
                                );
                                self.glyphon_renderer.render(
                                    &gpu.queue(),
                                    &gpu.device(),
                                    &mut self.managers,
                                    &mut rpass,
                                    &w,
                                    &self.camera,
                                );
                            }
                            gpu.queue().submit(Some(render_encoder.finish()));
                            frame.present();
                        }
                    }
                }
                Err(e) => {
                    log_error!("SurfaceError: {}", e);
                }
            },
            Err(e) => {
                log_error!("Draw failed, could not to acquire gpu lock: {}", e);
            }
        };
    }

    pub fn update(&mut self) {
        let binding = GPU::get();
        self.time.update();
        let _ = match binding.read() {
            Ok(gpu) => {
                self.camera.update(&mut self.controller);
                let camera_uniform_buffer = self.managers.buffer_manager.w_buffer.get_or_create(
                    self.camera.uniform_cache_key.clone(),
                    || {
                        WgpuBuffer::from_data(
                            gpu.queue(),
                            gpu.device(),
                            &[self.camera.uniform],
                            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                            Some(&format!("camera uniform buffer")),
                        )
                        .into()
                    },
                );
                camera_uniform_buffer.write_data(gpu.queue(), &[self.camera.uniform], None);

                if self.last_shape_time.elapsed().as_millis() > 1000 {
                    let text_buffer = self.managers.buffer_manager.g_buffer.get_or_create(
                        CacheKey::new(Self::TEXT_BUFFER),
                        || {
                            let metrics = glyphon::Metrics {
                                font_size: 20.0,
                                line_height: 20.0,
                            };
                            GlyphonBuffer::new(
                                &mut self.glyphon_renderer.font_system,
                                Some(metrics),
                            )
                            .into()
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
                                    self.camera.target.x,
                                    self.camera.target.y,
                                    self.camera.target.z
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
                        &gpu.device(),
                        &gpu.queue(),
                        text_buffer,
                        &self.surface_config,
                    );
                }
                true
            }
            Err(e) => {
                log_error!("App update failed, could not acquire gpu lock: {}", e);
                false
            }
        };
    }
}
