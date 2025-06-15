use crossbeam::channel::{Receiver, Sender};
use engine::{
    asset_service,
    camera::{Camera, Projection},
    debug_scene, log_error, log_info, log_warning,
    menu::Menu,
    menu_element::MenuElement,
    ApplicationEvent, AssetRequest, AssetService, BindGroup, CacheKey, DebugMode, Dispatch,
    EngineError, Entity, FrameBuffer, GlyphonTextRenderer, Light, Medium, Position, RenderPass,
    RenderTargetKind, RenderTargetManager, Renderer2d, Renderer3d, ScreenCorner, SurfaceExt,
    Terrain, TextRegion, Texture, Time, UiEvent, Velocity, World, WorldProjection, GPU,
};
use glam::Vec3;
use std::sync::Arc;
use winit::{
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes},
};

#[allow(dead_code)]
pub struct Rupy {
    pub time: Time,
    pub window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    pub world: World,
    render3d: Renderer3d,
    render2d: Renderer2d,
    render_targets: RenderTargetManager,
    rendertxt: GlyphonTextRenderer,
    pub camera: Camera,
    pub projection: Projection,
    uniform_bind_group: Arc<wgpu::BindGroup>,
    pub bossman: Entity,
    pub debug_mode: DebugMode,
    pub menu: Menu,
    tx: Arc<Sender<ApplicationEvent>>,
    asset_tx: Arc<Sender<AssetRequest>>,
    service: Arc<engine::AssetService>,
}

impl Rupy {
    pub fn new(
        event_loop: &ActiveEventLoop,
        tx: Arc<Sender<ApplicationEvent>>,
        asset_tx: Arc<Sender<AssetRequest>>,
        asset_rx: Arc<Receiver<AssetRequest>>,
    ) -> Result<Rupy, EngineError> {
        GPU::init();

        let win_attrs = WindowAttributes::default().with_title("RupyEngine");
        let window = Arc::new(event_loop.create_window(win_attrs)?);
        let win_clone = Arc::clone(&window);
        let (width, height) = {
            let inner_size = window.inner_size();
            (inner_size.width, inner_size.height)
        };
        let binding = GPU::get();
        let (surface, surface_config, gpu) = {
            let gpu = binding
                .read()
                .map_err(|e| EngineError::GpuError(format!("{}", e.to_string())))?;

            let surface = gpu.instance().create_surface(win_clone)?;
            let mut surface_config = surface
                .get_default_config(&gpu.adapter(), width, height)
                .ok_or(EngineError::SurfaceConfigError(
                    "surface isn't supported by this adapter".into(),
                ))?;
            surface_config.present_mode = wgpu::PresentMode::AutoVsync;
            (surface, surface_config, gpu)
        };

        AssetService::spawn_thread(gpu.queue().clone(), gpu.device().clone(), asset_rx);

        let service = asset_service();
        let device = service.device();
        surface.configure(device, &surface_config);

        let time = Time::new();

        let rendertxt = GlyphonTextRenderer::new(device, service.queue(), surface_config.format);

        let projection = Projection::FirstPerson;
        let mut camera = Camera::new(
            device,
            service.bind_group_layouts(),
            width as f32,
            height as f32,
            5.0,
            0.4,
        );

        let mut render_targets = RenderTargetManager::new();
        render_targets.insert(
            FrameBuffer::new_with_depth(
                device,
                (surface_config.width, surface_config.height).into(),
                surface_config.format,
                Texture::DEPTH_FORMAT,
                "scene buffer",
            ),
            RenderTargetKind::Scene,
        );
        render_targets.insert(
            FrameBuffer::new_color_only(
                device,
                (surface_config.width, surface_config.height).into(),
                surface_config.format,
                "hdr buffer",
            ),
            RenderTargetKind::Hdr,
        );

        let render3d = Renderer3d::new(service, &surface_config)?;
        let render2d = Renderer2d::new(device)?;

        let world_projection = WorldProjection::new(
            &service,
            &surface_config,
            "equirect_src.wgsl",
            "equirect_dst.wgsl",
            "pure-sky.hdr",
        )?;

        let terrain = Terrain::new(Medium::Ground);
        let light = Light::new(service.device(), service.bind_group_layouts())?;

        let uniform_bind_group = service.get_or_create_bind_group("uniform".into(), || {
            Ok(BindGroup::uniform(
                device,
                service.bind_group_layouts(),
                camera.buffer(),
                light.buffer(),
            )
            .into())
        })?;

        let mut world = World::new(world_projection, terrain, light)?;
        let debug_mode = DebugMode::new(service, &camera, &world.light, &surface_config)?;

        let bossman = debug_scene(
            &asset_tx,
            service.bind_group_layouts(),
            &mut world,
            surface_config.format,
        );

        let menu = Menu::builder(&surface_config, width, height)
            .with_position(200.0, 200.0)
            .with_padding(8.0)
            .with_button(
                "Play",
                Dispatch::Event(ApplicationEvent::Run),
                (420.0, 120.0),
                [1.0, 1.0, 1.0, 1.0],
                [0.0, 0.0, 0.5, 0.5],
                [0.5, 0.5, 1.0, 1.0],
                Some("cube-diffuse.jpg"),
            )
            .with_button(
                "Quit",
                Dispatch::Event(ApplicationEvent::Shutdown),
                (420.0, 120.0),
                [1.0, 1.0, 1.0, 1.0],
                [0.5, 0.0, 1.0, 0.5],
                [1.0, 0.5, 0.5, 1.0],
                Some("cube-normal.png"),
            )
            .build(&service)?;

        camera.load_model(service, &surface_config);

        let camera_entity = world.spawn();
        if let Some((renderable, scale, position)) = camera.spawn(camera_entity) {
            world.insert_renderable(camera_entity, renderable);
            world.insert_position(camera_entity, position);
            world.insert_scale(camera_entity, scale);
        };

        if let Some(material) = service.get_material(&CacheKey::from("Material.001")) {
            world.generate_terrain(
                service.queue(),
                service.device(),
                camera.eye(),
                1,
                Medium::Ground,
                &material,
            )?;
        }

        Ok(Rupy {
            time,
            window,
            surface,
            surface_config,
            world,
            render3d,
            render2d,
            rendertxt,
            camera,
            projection,
            render_targets,
            uniform_bind_group,
            bossman,
            debug_mode,
            menu,
            tx,
            asset_tx,
            service: service.clone(),
        })
    }
    pub fn shutdown(&self, el: &ActiveEventLoop) {
        World::stop();
        if let Err(e) = self.asset_tx.send(AssetRequest::Shutdown) {
            log_error!(
                "Failed to send asset service shutdown command: {}",
                e.to_string()
            );
        };
        if !el.exiting() {
            el.exit();
        }
        log_info!("Shutdown");
    }
    pub fn handle_key(&mut self, key: PhysicalKey) {
        match key {
            PhysicalKey::Code(KeyCode::Tab) => {
                self.dispatch(Dispatch::Event(ApplicationEvent::ToggleFullscreen))
            }
            PhysicalKey::Code(KeyCode::Numpad1) => {
                let new_speed = (self.world.light().speed() + 0.1).clamp(0.1, 1.5);
                self.world.light.set_speed(new_speed);
            }
            PhysicalKey::Code(KeyCode::Numpad2) => {
                let new_speed = (self.world.light().speed() - 0.1).clamp(0.1, 1.5);
                self.world.light.set_speed(new_speed);
            }
            PhysicalKey::Code(KeyCode::KeyP) => self.next_debug_mode(),
            PhysicalKey::Code(KeyCode::Escape) => {
                self.dispatch(Dispatch::Event(ApplicationEvent::Shutdown))
            }
            _ => {}
        }
    }

    pub fn set_projection(&mut self, projection: Projection) {
        self.projection = projection;
        self.camera.set_projection_far_near(&self.projection);
    }

    pub fn handle_menu_toggle(&mut self, event: &WindowEvent) {
        self.menu.process(event);
        let visible = self.menu.is_visible();
        let camera_frozen = self.camera.is_frozen();
        if visible && !camera_frozen {
            self.camera.freeze();
        }
        if !visible && camera_frozen {
            self.camera.unfreeze();
        }
    }
    pub fn next_debug_mode(&mut self) {
        self.debug_mode.next_mode(
            self.service.device(),
            self.service.bind_group_layouts(),
            &self.camera,
            self.world.light(),
        );
    }
    pub fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        let width = new_size.width.max(1) as f32;
        let height = new_size.height.max(1) as f32;
        self.surface.resize(
            self.service.device(),
            &mut self.surface_config,
            width,
            height,
        );
        self.camera.resize(width, height);
        self.menu
            .resize(&self.service.queue(), self.service.device(), width, height);
        self.rendertxt.resize(self.service.queue(), width, height);
        self.render_targets
            .resize(self.service.device(), width, height);
    }

    fn text_regions(&mut self) -> Vec<TextRegion> {
        let regions = vec![self.time.text_region(ScreenCorner::TopLeft.pos(
            self.surface_config.width,
            self.surface_config.height,
            5.0,
        ))];
        regions
    }

    pub fn redraw(&mut self) {
        self.time.update(Time::MAX_FRAME_TIME);

        if self.menu.is_visible() {
            let clicked = self.camera.controller().mouse_pressed();
            let mouse_pos = if let Some(pos) = self.camera.controller().last_mouse_pos() {
                Some((pos.x, pos.y))
            } else {
                None
            };
            let ui_events = self.menu.update(mouse_pos, clicked);
            for ev in ui_events {
                match ev {
                    UiEvent::ButtonClicked(id) => {
                        if let Some(el) = self.menu.get_element(id) {
                            match el {
                                engine::UiElement::Menu(menu_element) => match menu_element {
                                    MenuElement::Button(btn) => self.dispatch(btn.action()),
                                },
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        while self.time.consume_accumulator(Time::TIME_STEP) {
            if World::running() {
                self.world.update(
                    self.service.queue(),
                    self.service.device(),
                    &self.time,
                    &self.camera,
                    self.bossman,
                );
                if let Some(cam_ent) = self.camera.entity() {
                    let player_pos: Vec3 = self
                        .world
                        .physics
                        .positions
                        .get(cam_ent.0)
                        .and_then(|p| *p)
                        .unwrap_or(Position::origin())
                        .0;

                    let prev_velocity = self
                        .world
                        .physics
                        .velocities
                        .get(cam_ent.0)
                        .and_then(|v| *v)
                        .unwrap_or(Velocity(Vec3::ZERO))
                        .0;
                    if let Some(cam_update) = self.camera.update(
                        &self.projection,
                        player_pos,
                        prev_velocity,
                        self.surface_config.width as f32,
                        self.surface_config.height as f32,
                    ) {
                        if let Some(vel) = cam_update.velocity {
                            self.world.insert_velocity(cam_update.entity, vel);
                        }
                        if let Some(rot) = cam_update.rotation {
                            self.world.insert_rotation(cam_update.entity, rot);
                        }
                    }
                }
                if let Ok(models) = self.service.models() {
                    self.render3d.instances.update(
                        self.service.device(),
                        self.service.queue(),
                        &self.world,
                        &self.camera,
                        &models,
                    );
                }
            }

            self.upload();
        }

        self.render();
        self.window.request_redraw();
    }

    pub fn upload(&mut self) {
        self.camera
            .upload(self.service.queue(), self.service.device());
        self.world
            .upload(self.service.queue(), self.service.device());
        self.render3d
            .instances
            .upload(self.service.queue(), self.service.device());
    }

    pub fn render(&mut self) {
        let frame = match self.surface.texture() {
            Ok(f) => f,
            Err(e) => {
                match e {
                    wgpu::SurfaceError::Outdated => {
                        self.resize(&self.window.inner_size());
                        return;
                    }
                    wgpu::SurfaceError::Other
                    | wgpu::SurfaceError::Timeout
                    | wgpu::SurfaceError::Lost => {
                        log_warning!("{}", e);
                        return;
                    }
                    wgpu::SurfaceError::OutOfMemory => {
                        panic!("{}", e);
                    }
                };
            }
        };

        let surface_view = frame.texture.create_view(&Default::default());
        let mut encoder =
            self.service
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Scene Encoder"),
                });

        if let Some(framebuffer) = self.render_targets.get(&RenderTargetKind::Scene) {
            if let (Ok(models), Ok(materials)) = (self.service.models(), self.service.materials()) {
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Scene Pass"),
                    color_attachments: &[Some(framebuffer.color_attachment())],
                    depth_stencil_attachment: framebuffer.depth_attachment(),
                    ..Default::default()
                });
                self.render3d.render(
                    &models,
                    &materials,
                    &mut rpass,
                    &self.world,
                    &self.uniform_bind_group,
                    &self.debug_mode,
                );
                drop(rpass);
            }
        }

        if let (Some(scene_fb), Some(hdr_fb)) = (
            self.render_targets.get(&RenderTargetKind::Scene),
            self.render_targets.get(&RenderTargetKind::Hdr),
        ) {
            self.render3d.hdr(
                self.service.device(),
                self.service.bind_group_layouts(),
                &mut encoder,
                &scene_fb.color(),
                hdr_fb,
            );
        }

        if let Some(hdr_fb) = self.render_targets.get(&RenderTargetKind::Hdr) {
            self.render3d.final_blit_to_surface(
                self.service.device(),
                self.service.bind_group_layouts(),
                &mut encoder,
                hdr_fb.color(),
                &surface_view,
            );
        }

        let mut rpass2d = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("2D Overlay on Swapchain"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });

        self.render2d.begin_batch();

        for item in self.text_regions() {
            self.rendertxt.queue_text(
                &item.text,
                item.pos[0],
                item.pos[1],
                glyphon::Color::rgb(255, 255, 255),
            );
        }
        self.menu.render(
            &self.service,
            &mut rpass2d,
            &mut self.render2d,
            &mut self.rendertxt,
        );
        self.rendertxt.draw(
            self.service.device(),
            self.service.queue(),
            &self.surface_config,
            &mut rpass2d,
        );
        drop(rpass2d);
        self.service.queue().submit(Some(encoder.finish()));
        frame.present();
    }
    pub fn dispatch(&self, event: Dispatch) {
        match event {
            Dispatch::Asset(asset_request) => {
                if let Err(e) = self.asset_tx.send(asset_request) {
                    log_error!("Failed to dispatch asset request: {}", e.to_string());
                }
            }
            Dispatch::Event(application_event) => {
                if let Err(e) = self.tx.send(application_event) {
                    log_error!("Failed to dispatch application event: {}", e.to_string());
                }
            }
        }
    }
}
