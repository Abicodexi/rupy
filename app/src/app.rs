use engine::{
    camera::{Camera, CameraControls, Projection},
    debug_scene, log_debug, log_error, log_info, BindGroup, DebugMode, DebugUniform, EngineError,
    Entity, FrameBuffer, Light, Medium, RenderPass, RenderTargetKind, RenderTargetManager,
    RenderText, Renderer3d, Rotation, ScreenCorner, SurfaceExt, TextRegion, Texture, Time,
    Velocity, WgpuBuffer, World,
};
use glam::Vec3;
use std::sync::Arc;
use wgpu::BufferUsages;
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
    controls: CameraControls,
    last_shape_time: std::time::Instant,
    uniform_bind_group: wgpu::BindGroup,
    model_manager: engine::ModelManager,
    bossman: Entity,
    debug_mode: DebugMode,
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
            let gpu = binding
                .read()
                .map_err(|e| crate::EngineError::GpuError(format!("{}", e.to_string())))?;

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

        let projection = Projection::ThirdPerson;
        let mut camera = Camera::new(&device, width as f32 / height as f32);
        let light = Light::new(&device)?;
        let controls = CameraControls::new(5.0, 0.1);

        let mut world = World::new(queue, device, &surface_config, Some(depth_stencil.clone()))?;

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

        let debug_mode = DebugMode::new(
            device,
            &mut model_manager.materials.shaders,
            &camera,
            &light,
            &surface_config,
        )?;

        let bossman = debug_scene(
            &mut model_manager,
            &mut world,
            &surface_config,
            depth_stencil.clone(),
        );
        camera.world_spawn(&mut world, &mut model_manager, &surface_config);
        let mediums = vec![Medium::Water, Medium::Water, Medium::Vacuum, Medium::Vacuum];
        world.generate_terrain(
            *camera.eye(),
            1,
            mediums,
            &surface_config,
            &depth_stencil,
            &mut model_manager,
        );
        model_manager.materials.build_storage(device);
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
            controls,
            render_targets,
            last_shape_time: std::time::Instant::now(),
            uniform_bind_group,
            model_manager,
            bossman,
            debug_mode,
        })
    }
    pub fn shutdown(&self, el: &ActiveEventLoop) {
        log_info!("Shutdown");
        World::stop();
        if !el.exiting() {
            el.exit();
        }
    }
    pub fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.controls.process_event(event)
    }
    pub fn window(&self) -> &Window {
        &self.window
    }
    pub fn cam(&self) -> &Camera {
        &self.camera
    }
    pub fn cam_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }
    pub fn next_projection(&mut self) {
        self.projection = if self.projection == Projection::FirstPerson {
            Projection::ThirdPerson
        } else {
            Projection::FirstPerson
        };
    }
    pub fn next_debug_mode(&mut self) {
        self.debug_mode
            .next_mode(&self.model_manager.device, &self.camera, &self.light);
        log_debug!("Debug mode: {:?}", self.debug_mode.mode());
    }
    pub fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        self.camera
            .resize(new_size.width as f32, new_size.height as f32);
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
                        &self.debug_mode,
                    );

                    self.rendertxt.render(
                        &mut self.model_manager,
                        &mut rpass,
                        &self.world,
                        &self.uniform_bind_group,
                        &self.debug_mode,
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
        let camera = self.camera.text_region(corner);
        let controller = self.controls.text_region(corner);
        let time = self.time.text_region(corner);
        let regions = vec![time, camera, controller];
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
        let dt = self.time.delta_time as f32;

        self.camera.update(
            &mut self.world,
            &mut self.controls,
            &self.projection,
            &self.bossman,
        );

        if let Some(entity) = self.camera.entity() {
            if let (Some(cam_pos), Some(boss_pos)) = (
                self.world.physics.positions[entity.0],
                self.world.physics.positions[self.bossman.0],
            ) {
                let direction = cam_pos.0 - boss_pos.0;
                let mut direction_normalized = direction.normalize_or_zero();
                let speed = self.controls.speed() - (self.controls.speed() / 2.0);
                let velocity = direction_normalized * speed;
                direction_normalized.y = 0.0;
                let rot_to_camera = glam::Quat::from_rotation_arc(Vec3::Z, direction_normalized);
                self.world
                    .insert_rotation(self.bossman, Rotation::from(rot_to_camera));
                self.world.insert_velocity(self.bossman, Velocity(velocity));
            }
        }

        self.world.terrain.update_streaming(*self.camera.eye(), 4);

        self.light.orbit(self.time.elapsed * 0.1);
        self.render3d
            .instances
            .update(&self.world, &self.camera, &mut self.model_manager);

        self.world.update(
            &self.model_manager.queue,
            &self.model_manager.device,
            &self.camera,
            dt,
        );

        if self.last_shape_time.elapsed().as_millis() > 1000 {
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
