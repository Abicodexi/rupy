use engine::{
    camera::{Action, Camera, CameraController, MovementMode, Projection},
    log_error, BindGroup, BindGroupLayouts, EngineError, FrameBuffer, Light, RenderPass,
    RenderTargetKind, RenderTargetManager, RenderText, Renderer3d, Scale, ScreenCorner, SurfaceExt,
    TextRegion, Texture, Time, Vertex, VertexInstance, World,
};
use std::sync::Arc;
use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

#[allow(dead_code)]
pub struct Rupy {
    time: Time,
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    world: World,
    render3d: Renderer3d,
    render_targets: RenderTargetManager,
    rendertxt: RenderText,
    camera: Camera,
    projection: Projection,
    light: Light,
    controller: CameraController,
    last_shape_time: std::time::Instant,
    uniform_bind_group: wgpu::BindGroup,
    model_manager: engine::ModelManager,
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
        let render3d = Renderer3d::new(&device, &surface_config)?;
        let depth_stencil = wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };
        let rendertxt = RenderText::new(
            &device,
            &queue,
            surface_config.format,
            &Some(depth_stencil.clone()),
        );
        let mut camera = Camera::new(&device, MovementMode::Full3D, width as f32 / height as f32);
        let light = Light::new(&device)?;
        let controller = CameraController::new(0.1, 0.005);

        let mut world = World::new(queue, device, &surface_config, Some(depth_stencil.clone()))?;

        let size = 10;
        let wall_height = 15;
        let wall_y_offset = 0.0;

        let cube_obj = "goblin.obj";

        camera.add_model(
            &mut model_manager,
            &[Vertex::LAYOUT, VertexInstance::LAYOUT],
            vec![
                BindGroupLayouts::uniform().clone(),
                BindGroupLayouts::equirect_dst().clone(),
                BindGroupLayouts::material_storage().clone(),
                BindGroupLayouts::normal().clone(),
            ],
            &surface_config,
        );

        camera.spawn(&mut world, &mut model_manager, &surface_config);

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

                    world.insert_scale(entity, Scale::new(0.5, 0.5, 0.5));
                    world.insert_position(entity, ((14.0 - x as f32), 0.0, z as f32).into());
                    world.insert_renderable(entity, model_key.into());
                }
            }

            //  Ceiling at y = wall_height – 1
            for x in 0..size {
                for z in 0..size {
                    let entity = world.spawn();

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
                    world.insert_scale(e1, Scale::new(0.5, 0.5, 0.5));
                    world.insert_position(e1, (x as f32, y as f32 + wall_y_offset, 0.0).into());
                    world.insert_renderable(e1, model_key.into());
                }
            }

            //  Left & Right walls (vary z & y, fix x)
            for z in 0..size {
                for y in 0..wall_height {
                    // left wall at x = 0
                    let e1 = world.spawn();
                    world.insert_scale(e1, Scale::new(0.5, 0.5, 0.5));
                    world.insert_position(e1, (0.0, y as f32 + wall_y_offset, z as f32).into());
                    world.insert_renderable(e1, model_key.into());

                    // right wall at x = size - 1
                    let e2 = world.spawn();
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
        let projection = Projection::FirstPerson;

        Ok(Rupy {
            time,
            window,
            surface,
            surface_config,
            world,
            render3d,
            rendertxt,
            camera,
            projection,
            light,
            controller,
            render_targets,
            last_shape_time: std::time::Instant::now(),
            uniform_bind_group,
            model_manager,
        })
    }

    pub fn controller(&mut self, event: &winit::event::WindowEvent) -> Action {
        self.controller.process(event)
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
    pub fn projection(&self) -> &Projection {
        &self.projection
    }
    pub fn set_projection(&mut self, projection: Projection) {
        self.projection = projection;
    }
    pub fn next_projection(&mut self) {
        self.projection = match self.projection {
            Projection::FirstPerson => Projection::ThirdPerson,
            Projection::ThirdPerson => Projection::FirstPerson,
        };
    }
    pub fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        self.surface.resize(
            &self.model_manager.device,
            &mut self.surface_config,
            *new_size,
        );
        self.rendertxt.resize(&self.model_manager.queue, *new_size);
        self.render_targets
            .resize(&self.model_manager.device, *new_size);
    }

    pub fn render(&mut self) {
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

                    self.render3d.render(
                        &mut self.model_manager,
                        &mut rpass,
                        &self.world,
                        &self.uniform_bind_group,
                    );

                    self.rendertxt.render(
                        &mut self.model_manager,
                        &mut rpass,
                        &self.world,
                        &self.uniform_bind_group,
                    );
                }

                // === 2. Postprocess Scene -> HDR ===
                if let Some(scene_fb) = self.render_targets.get(&RenderTargetKind::Scene) {
                    if let Some(hdr_fb) = self.render_targets.get(&RenderTargetKind::Hdr) {
                        self.render3d.hdr(
                            &mut encoder,
                            &self.model_manager,
                            &scene_fb.color(),
                            hdr_fb,
                        );
                    }
                }

                // === 3. Final HDR -> swapchain ===
                if let Some(hdr_fb) = self.render_targets.get(&RenderTargetKind::Hdr) {
                    self.render3d.final_blit_to_surface(
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
    fn text_regions(&mut self) -> Vec<TextRegion> {
        let corner =
            ScreenCorner::TopLeft.pos(self.surface_config.width, self.surface_config.height, 5.0);
        // let camera = self.camera.text_region(corner);
        // let controller = self.controller.text_region(corner);
        let time = self.time.text_region(corner);
        let regions = vec![time];
        regions
    }

    pub fn upload(&mut self) {
        let queue = &self.model_manager.queue;
        let device = &self.model_manager.device;
        self.light.upload(queue, device);
        self.camera.upload(queue, device);
        self.render3d.instances.upload(queue, device);
    }

    pub fn update(&mut self) {
        self.time.update();
        let dt = self.time.delta_time;
        if let Some(entity) = self.camera.entity() {
            self.controller.apply(
                &mut self.world,
                entity,
                self.camera.movement(),
                self.controller.speed(),
            );
        };

        self.world.update(dt);
        self.camera.update(&self.world, self.projection);

        self.light.orbit(self.time.elapsed);
        self.render3d
            .instances
            .update(&self.world, &self.camera, &mut self.model_manager);

        if self.last_shape_time.elapsed().as_millis() > 5000 {
            self.last_shape_time = std::time::Instant::now();
            let regions = self.text_regions();
            self.rendertxt.prepare_regions(
                &self.model_manager.device,
                &self.model_manager.queue,
                &regions,
                &self.surface_config,
            );
        }
    }
}
