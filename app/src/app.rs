use crossbeam::channel::Sender;
use engine::{
    camera::{Camera, Projection},
    debug_scene, log_info, log_warning,
    menu::Menu,
    menu_button::MenuButton,
    menu_element::MenuElement,
    service::asset_service,
    ApplicationEvent, AssetRequest, AssetService, BindGroup, CacheKey, CacheStorage, DebugMode,
    EngineError, Entity, FrameBuffer, GlyphonTextRenderer, Light, Medium, RenderPass,
    RenderTargetKind, RenderTargetManager, Renderer2d, Renderer3d, ScreenCorner, SurfaceExt,
    Terrain, TextRegion, Texture, Time, World, WorldProjection, GPU,
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
    rendertxt: GlyphonTextRenderer,
    pub camera: Camera,
    pub projection: Projection,
    last_shape_time: Instant,
    uniform_bind_group: wgpu::BindGroup,
    pub bossman: Entity,
    pub debug_mode: DebugMode,
    pub menu: Menu,
    tx: Arc<Sender<ApplicationEvent>>,
    asset_tx: Arc<Sender<AssetRequest>>,
    service: Arc<engine::AssetService>,
    accumulator: f32,
    last_frame_time: Instant,
}

impl Rupy {
    pub fn new(
        event_loop: &ActiveEventLoop,
        service: &'static Arc<AssetService>,
        tx: Arc<Sender<ApplicationEvent>>,
        asset_tx: Arc<Sender<AssetRequest>>,
    ) -> Result<Rupy, EngineError> {
        let win_attrs = WindowAttributes::default().with_title("RupyEngine");
        let window = Arc::new(event_loop.create_window(win_attrs)?);
        let win_clone = Arc::clone(&window);
        let (width, height) = {
            let inner_size = window.inner_size();
            (inner_size.width, inner_size.height)
        };
        let binding = GPU::get();
        let (surface, surface_config) = {
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
            (surface, surface_config)
        };

        surface.configure(&service.device, &surface_config);

        let time = Time::new();

        let rendertxt =
            GlyphonTextRenderer::new(&service.device, &service.queue, surface_config.format);

        let projection = Projection::FirstPerson;
        let mut camera = Camera::new(&service.device, width as f32, height as f32, 5.0, 0.4);

        let mut render_targets = RenderTargetManager::new();
        render_targets.insert(
            FrameBuffer::new_with_depth(
                &service.device,
                (surface_config.width, surface_config.height).into(),
                surface_config.format,
                Texture::DEPTH_FORMAT,
                "scene buffer",
            ),
            RenderTargetKind::Scene,
        );
        render_targets.insert(
            FrameBuffer::new_color_only(
                &service.device,
                (surface_config.width, surface_config.height).into(),
                surface_config.format,
                "hdr buffer",
            ),
            RenderTargetKind::Hdr,
        );

        if let (
            Ok(mut models),
            Ok(mut materials),
            Ok(mut textures),
            Ok(mut shaders),
            Ok(mut pipelines),
        ) = (
            service.models.write(),
            service.materials.write(),
            service.textures.write(),
            service.shaders.write(),
            service.pipelines.write(),
        ) {
            let render3d = Renderer3d::new(
                &service.device,
                &mut shaders,
                &service.bind_group_layouts,
                &surface_config,
            )?;
            let render2d = Renderer2d::new(&service.device)?;
            let world_projection = WorldProjection::new(
                &service.queue,
                &service.device,
                &surface_config,
                "equirect_src.wgsl",
                "equirect_dst.wgsl",
                "pure-sky.hdr",
                Some(render3d.depth_stencil.as_ref().clone()),
            )?;
            let terrain = Terrain::new(Medium::Ground);
            let light = Light::new(&service.device)?;
            let mut world = World::new(world_projection, terrain, light)?;
            let uniform_bind_group =
                BindGroup::uniform(&service.device, camera.buffer(), world.light.buffer());
            let debug_mode = DebugMode::new(
                &service.device,
                &mut shaders,
                &camera,
                &world.light,
                &surface_config,
            )?;
            let bossman = debug_scene(
                &service.queue,
                &service.device,
                &mut models,
                &mut materials,
                &mut textures,
                &mut shaders,
                &mut pipelines,
                &mut world,
                surface_config.format,
                render3d.depth_stencil.as_ref().clone(),
            );
            let button_uv = [0.0, 0.0, 1.0, 1.0];
            let container_uv = [0.0, 0.0, 0.0, 0.0];
            let play_button = MenuElement::Button(MenuButton::new(
                "Play",
                (200.0, 120.0),
                (400.0, 80.0),
                [1.0, 1.0, 1.0, 0.75],
                button_uv,
                [0.8, 0.8, 0.8, 1.0],
                Box::new(|| println!("Play clicked!")),
            ));
            let quit_tx = tx.clone();

            let quit_button = MenuElement::Button(MenuButton::new(
                "Quit",
                (200.0, 200.0),
                (400.0, 80.0),
                [1.0, 1.0, 1.0, 0.75],
                button_uv,
                [0.8, 0.8, 0.8, 1.0],
                Box::new(move || {
                    quit_tx.send(ApplicationEvent::MenuCallback("Quit")).ok();
                }),
            ));

            let diffuse_texture = textures.get_or_load_texture(
                &service.queue,
                &service.device,
                "cube-diffuse.jpg",
                surface_config.format,
            )?;
            let texture_bind_group = BindGroup::texture(&service.device, &diffuse_texture.0);
            let mut menu = Menu::with_elements(
                &service.device,
                texture_bind_group,
                width,
                height,
                200.0,
                200.0,
                [1.0, 1.0, 1.0, 0.5],
                container_uv,
                20.0,
                vec![play_button, quit_button],
            );
            menu.build_pipeline(
                &service.device,
                &surface_config,
                &mut pipelines.render,
                &mut shaders,
                &service.bind_group_layouts,
            )?;

            camera.load_model(
                &service.queue,
                &service.device,
                &mut models,
                &mut materials,
                &mut textures,
                &mut shaders,
                &mut pipelines,
                surface_config.format,
            );
            camera.spawn(&mut world);

            if let Some(material) = materials.get(&CacheKey::from("Material.001")) {
                world.generate_terrain(
                    &service.queue,
                    &service.device,
                    *camera.eye(),
                    1,
                    Medium::Ground,
                    material,
                )?;
            }

            return Ok(Rupy {
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
                bossman,
                debug_mode,
                menu,
                tx,
                asset_tx,
                service: service.clone(),
                accumulator: 0.0,
                last_frame_time: Instant::now(),
            });
        } else {
            panic!("Failed to create Rupy struc");
        };
    }

    pub fn shutdown(&self, el: &ActiveEventLoop) {
        log_info!("Shutdown");
        World::stop();
        if !el.exiting() {
            el.exit();
        }
    }

    pub fn next_projection(&mut self) {
        self.projection = self.projection.next();
        self.camera.set_projection_far_near(&self.projection);
    }
    pub fn set_projection(&mut self, projection: Projection) {
        self.camera.set_projection_far_near(&projection);
        self.projection = projection;
    }
    pub fn next_debug_mode(&mut self) {
        self.debug_mode
            .next_mode(&asset_service().device, &self.camera, self.world.light());
    }
    pub fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        let width = new_size.width.max(1) as f32;
        let height = new_size.height.max(1) as f32;
        self.surface.resize(
            &self.service.device,
            &mut self.surface_config,
            width,
            height,
        );
        self.camera.resize(width, height);
        self.menu
            .resize(&self.service.queue, &self.service.device, width, height);
        self.rendertxt.resize(&self.service.queue, width, height);
        self.render_targets
            .resize(&self.service.device, width, height);
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
            self.menu.update(
                self.camera.controller().last_mouse_pos(),
                self.camera.controller().mouse_just_pressed(),
            );
        }
        while self.time.consume_accumulator(Time::TIME_STEP) {
            self.camera.update(&mut self.world, &self.projection);
            self.world.update(
                &self.service.queue,
                &self.service.device,
                &self.time,
                &mut self.camera,
                &self.projection,
                self.bossman,
            );
            if let Ok(mut models) = self.service.models.write() {
                let screen_size = self.window.inner_size();
                self.render3d.instances.update(
                    &self.service.device,
                    &self.world,
                    &self.camera,
                    (screen_size.width as f32, screen_size.height as f32),
                    &self.projection,
                    &mut models,
                );
            }

            self.upload();
        }

        self.render();
        self.window.request_redraw();
    }

    pub fn upload(&mut self) {
        let screen_size = self.window.inner_size();

        self.camera.upload(
            &self.service.queue,
            &self.service.device,
            self.projection,
            screen_size.height as f32,
            screen_size.width as f32,
        );
        self.world.upload(&self.service.queue, &self.service.device);
        self.render3d
            .instances
            .upload(&self.service.queue, &self.service.device);
    }

    pub fn render(&mut self) {
        let frame = match self.surface.texture() {
            Ok(f) => f,
            Err(e) => {
                match e {
                    wgpu::SurfaceError::Outdated => {
                        return self.resize(&self.window.inner_size());
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
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Scene Encoder"),
                });

        if let Some(framebuffer) = self.render_targets.get(&RenderTargetKind::Scene) {
            if let (Ok(mut models), Ok(materials)) =
                (self.service.models.write(), self.service.materials.read())
            {
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Scene Pass"),
                    color_attachments: &[Some(framebuffer.color_attachment())],
                    depth_stencil_attachment: framebuffer.depth_attachment(),
                    ..Default::default()
                });
                self.render3d.render(
                    &mut models,
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
                &self.service.device,
                &mut encoder,
                &scene_fb.color(),
                hdr_fb,
            );
        }

        if let Some(hdr_fb) = self.render_targets.get(&RenderTargetKind::Hdr) {
            self.render3d.final_blit_to_surface(
                &self.service.device,
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
        if let Ok(pipelines) = self.service.pipelines.read() {
            self.menu.render(
                &mut rpass2d,
                &mut self.render2d,
                &mut self.rendertxt,
                &self.service.queue,
                &pipelines,
            );
        }
        self.rendertxt.draw(
            &self.service.device,
            &self.service.queue,
            &self.surface_config,
            &mut rpass2d,
        );
        drop(rpass2d);
        self.service.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
