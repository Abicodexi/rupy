use core::{
    camera::{Camera, CameraController},
    log_error, EngineError, EquirectProjection, GlyphonRenderer, Light, Managers, Renderer,
    SurfaceExt, Texture, Time, WgpuRenderer, World,
};
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
    pub uniform_bind_group: wgpu::BindGroup,
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

            let managers: Managers = gpu.into();
            (surface, surface_config, managers)
        };

        surface.configure(&managers.device, &surface_config);

        let time = Time::new();
        let wgpu_renderer = WgpuRenderer::new(&mut managers, &surface_config)?;
        let glyphon_renderer = GlyphonRenderer::new(
            &managers.device,
            &managers.queue,
            surface_config.format,
            wgpu_renderer.depth_stencil_state(),
        );
        let camera = Camera::new(&managers.device, width as f32 / height as f32);

        let light = Light::new(&managers.device)?;

        let controller = CameraController::new(0.1, 0.5);

        let equirect_projection = EquirectProjection::new(
            &mut managers,
            &surface_config,
            "equirect_src.wgsl",
            "equirect_dst.wgsl",
            "pure-sky.hdr",
            wgpu_renderer.depth_stencil_state(),
        )?;

        let uniform_bind_group =
            core::BindGroup::uniform(&managers.device, camera.buffer(), light.buffer());

        if let Some(world) = World::get() {
            match world.write().as_mut() {
                Ok(w) => {
                    let cube_obj = "cube.obj";
                    w.set_projection(equirect_projection);

                    if let Some(model_key) = World::load_object(
                        cube_obj,
                        &mut managers,
                        &surface_config,
                        wgpu_renderer.depth_stencil_state(),
                    ) {
                        let entity = w.spawn();
                        w.insert_position(entity, (1.0, 1.0).into());
                        w.insert_renderable(entity, model_key.into());
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
            uniform_bind_group,
            controller,
            last_shape_time: std::time::Instant::now(),
        })
    }

    pub fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        self.surface
            .resize(&self.managers.device, &mut self.surface_config, *new_size);
        self.glyphon_renderer
            .resize(&self.managers.queue, *new_size);

        self.wgpu_renderer.set_depth_texture(Texture::depth_texture(
            &self.managers.device,
            &self.surface_config,
        ));
    }

    pub fn draw(&mut self) {
        match self.surface.texture() {
            Ok(frame) => {
                if let Some(world) = World::get() {
                    if let Ok(w) = world.read() {
                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        let mut render_encoder = self.managers.device.create_command_encoder(
                            &wgpu::CommandEncoderDescriptor {
                                label: Some("render encoder"),
                            },
                        );

                        self.wgpu_renderer.compute_pass(&w, &mut self.managers);
                        {
                            let mut rpass =
                                render_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some("main pass"),
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view: &view,
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Load,
                                            store: wgpu::StoreOp::Store,
                                        },
                                    })],
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
                                &mut self.managers,
                                &mut rpass,
                                &w,
                                &self.camera,
                                &self.uniform_bind_group,
                            );
                            self.glyphon_renderer.render(
                                &mut self.managers,
                                &mut rpass,
                                &w,
                                &self.camera,
                                &self.uniform_bind_group,
                            );
                        }

                        self.wgpu_renderer.hdr(&mut render_encoder, &view);
                        self.managers.queue.submit(Some(render_encoder.finish()));
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
        };
    }

    fn buffer_lines(&mut self) -> Vec<glyphon::BufferLine> {
        let line_ending = glyphon::cosmic_text::LineEnding::LfCr;
        let attrs_list = glyphon::AttrsList::new(glyphon::Attrs::new());
        let shaping = glyphon::Shaping::Advanced;
        let lines = (
            self.camera.buffer_line(&line_ending, &attrs_list, &shaping),
            self.controller
                .buffer_line(&line_ending, &attrs_list, &shaping),
        );

        vec![
            glyphon::BufferLine::new(
                format!("fps: {:.1} dt: {:.4}", self.time.fps, self.time.delta_time),
                line_ending,
                attrs_list,
                shaping,
            ),
            lines.0 .0,
            lines.0 .1,
            lines.1,
        ]
    }

    pub fn upload(&mut self) {
        self.light.upload(&self.managers.queue);
        self.camera.upload(&self.managers.queue);
    }

    pub fn update(&mut self) {
        self.time.update();
        self.camera.update(&mut self.controller);
        self.light
            .orbit(self.time.elapsed * std::f32::consts::TAU / 5.0);

        if self.last_shape_time.elapsed().as_millis() > 1500 {
            self.last_shape_time = std::time::Instant::now();
            let lines = self.buffer_lines();
            let gb = core::CacheStorage::get_or_create(
                &mut self.managers.buffer_manager.g_buffer,
                "text buffer".into(),
                || {
                    core::GlyphonBuffer::new(
                        &mut self.glyphon_renderer.font_system,
                        Some(glyphon::Metrics {
                            font_size: 20.0,
                            line_height: 20.0,
                        }),
                    )
                    .into()
                },
            );
            gb.set_lines(lines);
            gb.shape(&mut self.glyphon_renderer.font_system);
            self.glyphon_renderer.prepare(
                &self.managers.device,
                &self.managers.queue,
                gb,
                &self.surface_config,
            );
        }
    }
}
