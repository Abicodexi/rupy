use engine::{
    camera::{Camera, CameraController},
    log_error, BindGroup, BindGroupLayouts, EngineError, FrameBuffer, GlyphonRenderer, Light,
    RenderTargetKind, RenderTargetManager, Renderer, Scale, SurfaceExt, Texture, Time, Vertex,
    VertexInstance, WgpuRenderer, World,
};
use std::sync::Arc;
use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

#[allow(dead_code)]
pub struct Rupy {
    pub time: Time,
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub world: World,
    pub wgpu_renderer: WgpuRenderer,
    pub render_targets: RenderTargetManager,
    pub glyphon_renderer: GlyphonRenderer,
    pub camera: Camera,
    pub light: Light,
    pub controller: CameraController,
    pub last_shape_time: std::time::Instant,
    pub uniform_bind_group: wgpu::BindGroup,
    pub model_manager: engine::ModelManager,
    pub text_buffer: engine::GlyphonBuffer,
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
        let binding = crate::GPU::get();
        let (surface, surface_config, gpu) = {
            let gpu = binding.read().map_err(|e| {
                crate::EngineError::GpuError(format!(
                    "Failed to acquire read lock: {}",
                    e.to_string()
                ))
            })?;

            let surface = gpu.instance().create_surface(win_clone)?;
            let mut surface_config = surface
                .get_default_config(&gpu.adapter(), width, height)
                .ok_or(EngineError::SurfaceConfigError(
                    "surface isn't supported by this adapter".into(),
                ))?;
            surface_config.present_mode = wgpu::PresentMode::AutoVsync;
            (surface, surface_config, gpu)
        };

        let device = gpu.device();
        let queue = gpu.queue();
        let mut model_manager = engine::ModelManager::new(queue.clone(), device.clone());

        surface.configure(&device, &surface_config);

        let time = Time::new();
        let wgpu_renderer = WgpuRenderer::new(&device, &surface_config)?;
        let depth_stencil = wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };
        let mut glyphon_renderer = GlyphonRenderer::new(
            &device,
            &queue,
            surface_config.format,
            &Some(depth_stencil.clone()),
        );
        let camera = Camera::new(&device, width as f32 / height as f32);
        let light = Light::new(&device)?;
        let controller = CameraController::new(4.0, 0.5);

        let mut world = World::new(queue, device, &surface_config, Some(depth_stencil.clone()))?;

        let size = 10;
        let wall_height = 15;
        let wall_y_offset = 0.0;

        let cube_obj = "goblin.obj";

        if let Some(model_key) = World::load_object(
            &mut model_manager,
            cube_obj,
            "v_normal.wgsl",
            &[Vertex::LAYOUT, VertexInstance::LAYOUT],
            vec![
                BindGroupLayouts::uniform().clone(),
                BindGroupLayouts::equirect_dst().clone(),
                BindGroupLayouts::material_storage().clone(),
                BindGroupLayouts::normal().clone(),
            ],
            &surface_config,
            wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            wgpu::ColorTargetState {
                format: surface_config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::all(),
            },
            Some(depth_stencil.clone()),
        ) {
            let entity = world.spawn();
            world.insert_rotation(entity, cgmath::Quaternion::new(0.0, 0.0, 0.0, 0.0).into());
            world.insert_scale(entity, Scale::new(10.0, 10.0, 10.0));
            world.insert_position(entity, (5.0 as f32, 5.5, 3.0 as f32).into());
            world.insert_renderable(entity, model_key.into());
        }
        if let Some(model_key) = World::load_object(
            &mut model_manager,
            "cube.obj",
            "v_normal.wgsl",
            &[Vertex::LAYOUT, VertexInstance::LAYOUT],
            vec![
                BindGroupLayouts::uniform().clone(),
                BindGroupLayouts::equirect_dst().clone(),
                BindGroupLayouts::material_storage().clone(),
                BindGroupLayouts::normal().clone(),
            ],
            &surface_config,
            wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            wgpu::ColorTargetState {
                format: surface_config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::all(),
            },
            Some(depth_stencil),
        ) {
            for x in 0..(size + 10) {
                for z in 0..(size + 10) {
                    let entity = world.spawn();
                    world.insert_rotation(
                        entity,
                        cgmath::Quaternion::new(0.0, 0.0, 0.0, 0.0).into(),
                    );
                    world.insert_scale(entity, Scale::new(0.5, 0.5, 0.5));
                    world.insert_position(entity, ((14.0 - x as f32), 0.0, z as f32).into());
                    world.insert_renderable(entity, model_key.into());
                }
            }

            //  Ceiling at y = wall_height – 1
            for x in 0..size {
                for z in 0..size {
                    let entity = world.spawn();
                    world.insert_rotation(
                        entity,
                        cgmath::Quaternion::new(0.0, 0.0, 0.0, 0.0).into(),
                    );
                    world.insert_scale(entity, Scale::new(0.5, 0.5, 0.5));
                    world.insert_position(
                        entity,
                        (x as f32, (wall_height - 1) as f32, z as f32).into(),
                    );
                    world.insert_renderable(entity, model_key.into());
                }
            }

            // Front & Back walls (vary x & y, fix z)

            for x in 0..size {
                for y in 0..wall_height {
                    // front wall at z = 0
                    let e1 = world.spawn();
                    world.insert_rotation(e1, cgmath::Quaternion::new(0.0, 0.0, 0.0, 0.0).into());
                    world.insert_scale(e1, Scale::new(0.5, 0.5, 0.5));
                    world.insert_position(e1, (x as f32, y as f32 + wall_y_offset, 0.0).into());
                    world.insert_renderable(e1, model_key.into());

                    // back wall at z = size - 1

                    // let e2 = w.spawn();
                    // w.insert_rotation(
                    //     e2,
                    //     cgmath::Quaternion::new(0.0, 0.0, 0.0, 0.0).into(),
                    // );
                    // w.insert_scale(e2, Scale::new(0.5, 0.5, 0.5));
                    // w.insert_position(
                    //     e2,
                    //     (x as f32, y as f32 + wall_y_offset, (size - 1) as f32).into(),
                    // );
                    // w.insert_renderable(e2, model_key.into());
                }
            }

            //  Left & Right walls (vary z & y, fix x)
            for z in 0..size {
                for y in 0..wall_height {
                    // left wall at x = 0
                    let e1 = world.spawn();
                    world.insert_rotation(e1, cgmath::Quaternion::new(0.0, 0.0, 0.0, 0.0).into());
                    world.insert_scale(e1, Scale::new(0.5, 0.5, 0.5));
                    world.insert_position(e1, (0.0, y as f32 + wall_y_offset, z as f32).into());
                    world.insert_renderable(e1, model_key.into());

                    // right wall at x = size - 1
                    let e2 = world.spawn();
                    world.insert_rotation(e2, cgmath::Quaternion::new(0.0, 0.0, 0.0, 0.0).into());
                    world.insert_scale(e2, Scale::new(0.5, 0.5, 0.5));
                    world.insert_position(
                        e2,
                        ((size - 1) as f32, y as f32 + wall_y_offset, z as f32).into(),
                    );
                    world.insert_renderable(e2, model_key.into());
                }
            }
            world.update_transforms(time.delta_time as f64);
        }
        let mut render_targets = RenderTargetManager::new();
        render_targets.insert(
            FrameBuffer::new_with_depth(
                &device,
                (surface_config.width, surface_config.height).into(),
                surface_config.format,
                Texture::DEPTH_FORMAT,
                "scene buffer",
            ),
            RenderTargetKind::Scene,
        );
        render_targets.insert(
            FrameBuffer::new_color_only(
                &device,
                (surface_config.width, surface_config.height).into(),
                surface_config.format,
                "hdr buffer",
            ),
            RenderTargetKind::Hdr,
        );
        let uniform_bind_group = BindGroup::uniform(&device, camera.buffer(), light.buffer());

        let text_buffer = engine::GlyphonBuffer::new(&mut glyphon_renderer.font_system, None);

        Ok(Rupy {
            time,
            window,
            surface,
            surface_config,
            world,
            wgpu_renderer,
            glyphon_renderer,
            camera,
            light,
            controller,
            render_targets,
            last_shape_time: std::time::Instant::now(),
            uniform_bind_group,
            model_manager,
            text_buffer,
        })
    }

    pub fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        self.surface.resize(
            &self.model_manager.device,
            &mut self.surface_config,
            *new_size,
        );
        self.glyphon_renderer
            .resize(&self.model_manager.queue, *new_size);
        self.render_targets
            .resize(&self.model_manager.device, *new_size);
    }

    pub fn draw(&mut self) {
        match self.surface.texture() {
            Ok(frame) => {
                let surface_view = frame.texture.create_view(&Default::default());

                // === 1. Render scene to scene framebuffer ===
                let mut encoder = self.model_manager.device.create_command_encoder(
                    &wgpu::CommandEncoderDescriptor {
                        label: Some("Scene Encoder"),
                    },
                );

                if let Some(frame) = self.render_targets.get(&RenderTargetKind::Scene) {
                    let projection = self.world.projection();
                    projection.compute_projection(
                        &self.model_manager.queue,
                        &self.model_manager.device,
                        Some("Equirect Projection Pass"),
                    );

                    let mut rpass: wgpu::RenderPass<'_> =
                        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Scene Pass"),
                            color_attachments: &[Some(frame.color_attachment())],
                            depth_stencil_attachment: frame.depth_attachment(),
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });

                    self.wgpu_renderer.render(
                        &mut self.model_manager,
                        &mut rpass,
                        &self.world,
                        &self.uniform_bind_group,
                    );

                    self.glyphon_renderer.render(
                        &mut self.model_manager,
                        &mut rpass,
                        &self.world,
                        &self.uniform_bind_group,
                    );
                }

                // === 2. Postprocess Scene -> HDR ===
                if let Some(scene_fb) = self.render_targets.get(&RenderTargetKind::Scene) {
                    if let Some(hdr_fb) = self.render_targets.get(&RenderTargetKind::Hdr) {
                        self.wgpu_renderer.hdr(
                            &mut encoder,
                            &self.model_manager,
                            &scene_fb.color(),
                            hdr_fb,
                        );
                    }
                }

                // === 3. Final HDR -> swapchain ===
                if let Some(hdr_fb) = self.render_targets.get(&RenderTargetKind::Hdr) {
                    self.wgpu_renderer.final_blit_to_surface(
                        &self.model_manager.device,
                        &mut encoder,
                        hdr_fb.color(),
                        &surface_view,
                    );
                }
                self.model_manager.queue.submit(Some(encoder.finish()));
                frame.present();
            }
            Err(e) => {
                log_error!("SurfaceError: {}", e);
                if let wgpu::SurfaceError::Outdated = e {
                    self.resize(&self.window.inner_size());
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
        self.light.upload(&self.model_manager.queue);
        self.camera.upload(&self.model_manager.queue);
        self.wgpu_renderer
            .instance_buffers
            .upload(&self.model_manager.queue, &self.model_manager.device);
    }

    pub fn update(&mut self) {
        self.time.update();
        self.camera.update(&mut self.controller);
        self.light
            .orbit(self.time.elapsed * std::f32::consts::TAU / 15.0);
        self.wgpu_renderer.instance_buffers.update_batches(
            &self.world,
            &self.camera,
            &mut self.model_manager,
        );
        if self.last_shape_time.elapsed().as_millis() > 1500 {
            self.last_shape_time = std::time::Instant::now();
            let lines = self.buffer_lines();

            self.text_buffer.set_lines(lines);
            self.text_buffer
                .shape(&mut self.glyphon_renderer.font_system);
            self.glyphon_renderer.prepare(
                &self.model_manager.device,
                &self.model_manager.queue,
                &mut self.text_buffer,
                &self.surface_config,
            );
        }
    }
}
