use crossbeam::channel::Sender;
use engine::{
    camera::{Camera, Projection},
    debug_scene, log_info,
    menu::Menu,
    menu_item::MenuAction,
    ApplicationEvent, BindGroup, DebugMode, EngineError, Entity, FrameBuffer, Medium, RenderPass,
    RenderTargetKind, RenderTargetManager, RenderText, Renderer2d, Renderer3d, ScreenCorner,
    SurfaceExt, TextRegion, Texture, Time, World,
};
use std::{sync::Arc, time::Instant};
use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
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
    rendertxt: RenderText,
    pub camera: Camera,
    pub projection: Projection,
    last_shape_time: Instant,
    uniform_bind_group: wgpu::BindGroup,
    pub model_manager: engine::ModelManager,
    pub bossman: Entity,
    pub debug_mode: DebugMode,
    pub menu: Menu,
    tx: Arc<Sender<ApplicationEvent>>,

    accumulator: f32,
    last_frame_time: Instant,
}

impl Rupy {
    pub fn new(
        event_loop: &ActiveEventLoop,
        tx: Arc<Sender<ApplicationEvent>>,
    ) -> Result<Rupy, EngineError> {
        let win_attrs = WindowAttributes::default().with_title("RupyEngine");
        let window = Arc::new(event_loop.create_window(win_attrs)?);
        let win_clone = Arc::clone(&window);
        let (width, height) = {
            let inner_size = window.inner_size();
            (inner_size.width, inner_size.height)
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
        let mut render3d = Renderer3d::new();
        let mut render2d = Renderer2d::new(width, height, &mut model_manager)?;
        render3d.build_pipelines(
            device,
            &surface_config,
            &mut model_manager.materials.pipelines.render,
        )?;
        render2d.build_pipelines(
            device,
            &surface_config,
            &mut model_manager.materials.pipelines.render,
        )?;
        let depth_stencil = wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };
        let rendertxt = RenderText::new(&device, &queue, surface_config.format);

        let projection = Projection::FirstPerson;
        let mut camera = Camera::new(&device, width as f32 / height as f32, 5.0, 0.4);

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

        let uniform_bind_group = BindGroup::uniform(&device, camera.buffer(), world.light.buffer());

        let debug_mode = DebugMode::new(
            device,
            &mut model_manager.materials.shaders,
            &camera,
            &world.light,
            &surface_config,
        )?;

        let bossman = debug_scene(
            &mut model_manager,
            &mut world,
            &surface_config,
            depth_stencil.clone(),
        );
        camera.world_spawn(&mut world, &mut model_manager, &surface_config);
        world.generate_terrain(
            *camera.eye(),
            1,
            Medium::Ground,
            &surface_config,
            &depth_stencil,
            &mut model_manager,
        );

        model_manager.materials.build_storage(device);

        let menu = Menu::new(
            vec![
                (
                    "Play",
                    MenuAction::Play,
                    Box::new({
                        let tx = tx.clone();
                        move || {
                            tx.send(ApplicationEvent::MenuCallback("Play")).ok();
                        }
                    }),
                ),
                (
                    "Options",
                    MenuAction::Options,
                    Box::new({
                        let tx = tx.clone();
                        move || {
                            tx.send(ApplicationEvent::MenuCallback("Options")).ok();
                        }
                    }),
                ),
                (
                    "Quit",
                    MenuAction::Quit,
                    Box::new({
                        let tx = tx.clone();
                        move || {
                            tx.send(ApplicationEvent::MenuCallback("Quit")).ok();
                        }
                    }),
                ),
            ],
            200.0, // x
            200.0, // y
            420.0,
            100.0,
            5.0,
        );

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
            last_shape_time: Instant::now(),
            uniform_bind_group,
            model_manager,
            bossman,
            debug_mode,
            menu,
            tx,
            accumulator: 0.0,
            last_frame_time: Instant::now(),
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
        self.camera.process_event(event)
    }

    pub fn next_projection(&mut self) {
        if self.projection == Projection::FirstPerson {
            self.projection = Projection::ThirdPerson;
            return;
        }
        if self.projection == Projection::ThirdPerson {
            self.projection = Projection::Orthographic;
            self.camera.set_zfar(1.0);
            self.camera.set_znear(-1.0);
            return;
        };
        if self.projection == Projection::Orthographic {
            self.projection = Projection::FirstPerson;
            self.camera.set_zfar(100.0);
            self.camera.set_znear(0.1);
            return;
        }
    }
    fn dispatch_menu_action(&self, action: MenuAction) {
        match action {
            MenuAction::Play => {
                self.tx.send(ApplicationEvent::MenuCallback("Play")).ok();
            }
            MenuAction::Options => {
                self.tx.send(ApplicationEvent::MenuCallback("Options")).ok();
            }
            MenuAction::Quit => {
                self.tx.send(ApplicationEvent::MenuCallback("Quit")).ok();
            }
        }
    }
    pub fn next_debug_mode(&mut self) {
        self.debug_mode
            .next_mode(&self.model_manager.device, &self.camera, self.world.light());
    }
    pub fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        self.camera
            .resize(new_size.width as f32, new_size.height as f32);
        self.surface.resize(
            &self.model_manager.device,
            &mut self.surface_config,
            *new_size,
        );
        self.render2d.resize(
            &self.model_manager.queue,
            &self.model_manager.device,
            new_size.width,
            new_size.height,
        );

        self.rendertxt.resize(&self.model_manager.queue, *new_size);
        self.render_targets
            .resize(&self.model_manager.device, *new_size);
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
        while self.time.consume_accumulator(Time::TIME_STEP) {
            self.world.update(
                &self.model_manager.queue,
                &self.model_manager.device,
                &self.time,
                &mut self.camera,
                &self.projection,
                self.bossman,
            );

            self.render3d.instances.update(
                &self.world,
                &self.camera,
                &self.projection,
                &mut self.model_manager,
            );
            self.upload();
        }

        if let Some((mouse_x, mouse_y)) = self.camera.controller().last_mouse_pos() {
            let (mouse_just_pressed_left, ..) = self.camera.controller().mouse_just_pressed();
            if let Some(action) = self
                .menu
                .update(*mouse_x, *mouse_y, mouse_just_pressed_left)
            {
                self.dispatch_menu_action(action);
            }
        }

        self.render();
        self.window.request_redraw();
    }

    pub fn upload(&mut self) {
        self.camera.upload(
            &self.model_manager.queue,
            &self.model_manager.device,
            self.projection,
        );
        self.world
            .upload(&self.model_manager.queue, &self.model_manager.device);
        self.render3d
            .instances
            .upload(&self.model_manager.queue, &self.model_manager.device);
    }

    pub fn render(&mut self) {
        let frame = match self.surface.texture() {
            Ok(f) => f,
            Err(e) => {
                match e {
                    wgpu::SurfaceError::Outdated => self.resize(&self.window.inner_size()),
                    wgpu::SurfaceError::Other
                    | wgpu::SurfaceError::OutOfMemory
                    | wgpu::SurfaceError::Timeout
                    | wgpu::SurfaceError::Lost => {
                        panic!("SurfaceError: {}", e);
                    }
                }
                return;
            }
        };

        let surface_view = frame.texture.create_view(&Default::default());

        let mut encoder =
            self.model_manager
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Scene Encoder"),
                });

        if let Some(framebuffer) = self.render_targets.get(&RenderTargetKind::Scene) {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Scene Pass"),
                color_attachments: &[Some(framebuffer.color_attachment())],
                depth_stencil_attachment: framebuffer.depth_attachment(),
                ..Default::default()
            });
            self.render3d.render(
                &mut self.model_manager,
                &mut rpass,
                &self.world,
                &self.uniform_bind_group,
                &self.debug_mode,
            );
        }

        if let (Some(scene_fb), Some(hdr_fb)) = (
            self.render_targets.get(&RenderTargetKind::Scene),
            self.render_targets.get(&RenderTargetKind::Hdr),
        ) {
            self.render3d
                .hdr(&mut encoder, &self.model_manager, &scene_fb.color(), hdr_fb);
        }

        if let Some(hdr_fb) = self.render_targets.get(&RenderTargetKind::Hdr) {
            self.render3d.final_blit_to_surface(
                &self.model_manager.device,
                &mut encoder,
                hdr_fb.color(),
                &surface_view,
                &self.model_manager,
            );
        }

        {
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

            if self.menu.visible() {
                self.menu.draw_ui(&mut self.render2d, &mut self.rendertxt);
            }
            for item in self.text_regions() {
                self.rendertxt.queue_text(
                    &item.text,
                    item.pos[0],
                    item.pos[1],
                    glyphon::Color::rgb(255, 255, 255),
                );
            }
            self.render2d.flush(&mut rpass2d, &self.model_manager);
            self.rendertxt.draw(
                &self.model_manager.device,
                &self.model_manager.queue,
                &self.surface_config,
                &mut rpass2d,
            );
            drop(rpass2d);
        }
        self.model_manager.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
