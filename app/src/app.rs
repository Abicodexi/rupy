use core::{
    camera::{Camera, CameraController},
    log_error, BindGroupLayouts, CacheKey, CacheStorage, EngineError, EquirectProjection,
    GlyphonBuffer, GlyphonRenderer, Light, Managers, Renderer, SurfaceExt, Texture, Time,
    WgpuRenderer, World, GPU,
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
    pub light: Light,
    pub controller: CameraController,
    pub last_shape_time: std::time::Instant,
}

impl Rupy {
    pub fn new(event_loop: &ActiveEventLoop) -> Result<Rupy, EngineError> {
        let win_attrs = WindowAttributes::default().with_title("RupyEngine");
        let window = Arc::new(event_loop.create_window(win_attrs)?);
        let win_clone = Arc::clone(&window);
        let (width, height) = {
            let inenr_size = window.inner_size();
            (inenr_size.width, inenr_size.height)
        };

        let (surface, surface_config, mut managers) = {
            let binding = crate::GPU::get();
            let gpu = binding.read().map_err(|e| {
                crate::EngineError::GpuError(format!(
                    "Failed to acquire read lock: {}",
                    e.to_string()
                ))
            })?;

            let surface = gpu.instance().create_surface(win_clone)?;
            let surface_config = surface
                .get_default_config(&gpu.adapter(), width, height)
                .ok_or(EngineError::SurfaceConfigError(
                    "surface isn't supported by this adapter".into(),
                ))?;

            let managers = Managers::new(gpu.queue(), gpu.device());
            (surface, surface_config, managers)
        };

        BindGroupLayouts::init(&managers.device);

        surface.configure(&managers.device, &surface_config);

        let time = Time::new();
        let wgpu_renderer = WgpuRenderer::new(&mut managers, &surface_config)?;
        let glyphon_renderer = GlyphonRenderer::new(
            &managers.device,
            &managers.queue,
            surface_config.format,
            wgpu_renderer.depth_stencil_state(),
        );
        let camera = Camera::new(
            &managers.queue,
            &managers.device,
            width as f32 / height as f32,
        );

        let light = Light::new(
            &managers.queue,
            &managers.device,
            [0.0, 0.0, 0.0].into(),
            [0.0, 0.0, 0.0].into(),
        )?;

        let controller = CameraController::new(0.1, 0.5);

        let equirect_projection = EquirectProjection::new(
            &mut managers,
            &surface_config,
            "equirect_src.wgsl",
            "equirect_dst.wgsl",
            "pure-sky.hdr",
            1080,
            wgpu::TextureFormat::Rgba32Float,
            wgpu_renderer.depth_stencil_state(),
        )?;
        let bind_group_layout = core::BindGroupLayouts::uniform();

        let uniform_bind_group = managers
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    // entry 0 → camera
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera.uniform_buffer.get().as_entire_binding(),
                    },
                    // entry 1 → light
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: light.uniform_buffer.get().as_entire_binding(),
                    },
                ],
                label: Some("camera+light bind group"),
            });
        if let Some(world) = World::get() {
            match world.write().as_mut() {
                Ok(w) => {
                    let cube_obj = "cube.obj";
                    w.set_projection(equirect_projection);

                    if let Some(model_key) = World::load_object(
                        cube_obj,
                        &mut managers,
                        &uniform_bind_group,
                        &camera,
                        &light,
                        &surface_config,
                        wgpu_renderer.depth_stencil_state(),
                    ) {
                        let entity = w.spawn();
                        w.insert_position(entity, core::Position { x: 1.0, y: 1.0 });
                        w.insert_renderable(
                            entity,
                            core::Renderable {
                                model_key,
                                visible: true,
                            },
                        );
                    }
                }
                _ => (),
            }
        }

        Ok(Rupy {
            managers,
            time,
            window,
            surface,
            surface_config,
            wgpu_renderer,
            glyphon_renderer,
            camera,
            light,
            controller,
            last_shape_time: std::time::Instant::now(),
        })
    }

    pub fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        self.surface
            .resize(&self.managers.device, &mut self.surface_config, *new_size);
        self.glyphon_renderer.resize(
            &self.managers.queue,
            glyphon::Resolution {
                width: self.surface_config.width,
                height: self.surface_config.height,
            },
        );

        self.wgpu_renderer.set_depth_texture(Texture::depth_texture(
            &self.managers.device,
            &self.surface_config,
        ));
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
                                                view: &self.wgpu_renderer.depth_texture().view,
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

                            self.wgpu_renderer.process_hdr(&mut render_encoder, &view);
                            gpu.queue().submit(Some(render_encoder.finish()));
                            frame.present();
                        }
                    }
                }
                Err(e) => {
                    log_error!("SurfaceError: {}", e);
                    match e {
                        wgpu::SurfaceError::Outdated => {
                            self.resize(&self.window.inner_size());
                        }
                        _ => (),
                    }
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
                self.camera
                    .uniform_buffer
                    .write_data(gpu.queue(), &[self.camera.uniform()], None);
                self.light
                    .uniform_buffer
                    .write_data(gpu.queue(), &[self.light.uniform()], None);

                if self.last_shape_time.elapsed().as_millis() > 1000 {
                    let text_buffer = self.managers.buffer_manager.g_buffer.get_or_create(
                        CacheKey::new("debug text buffer"),
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
