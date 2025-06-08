use crossbeam::channel::Sender;
use engine::{
    camera::{Camera, Projection},
    debug_scene, log_info,
    menu::Menu,
    menu_button::MenuButton,
    menu_element::MenuElement,
    ApplicationEvent, BindGroup, CacheKey, CacheStorage, DebugMode, EngineError, Entity,
    FrameBuffer, GlyphonTextRenderer, MaterialManager, Medium, ModelManager, PipelineManager,
    RenderPass, RenderTargetKind, RenderTargetManager, Renderer2d, Renderer3d, ScreenCorner,
    ShaderManager, SurfaceExt, TextRegion, Texture, TextureManager, Time, World,
};
use std::{sync::Arc, time::Instant};
use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

#[allow(dead_code)]
pub struct Rupy {
    pub queue: Arc<wgpu::Queue>,
    pub device: Arc<wgpu::Device>,
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
    pub models: ModelManager,
    pub materials: MaterialManager,
    pub textures: TextureManager,
    pub shaders: ShaderManager,
    pub pipelines: PipelineManager,
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
        let mut materials = MaterialManager::new();
        let mut models = ModelManager::new();
        let mut textures = TextureManager::new();
        let mut shaders = ShaderManager::new();
        let mut pipelines = PipelineManager::new();

        surface.configure(&device, &surface_config);

        let time = Time::new();
        let render3d = Renderer3d::new(&device, &surface_config)?;
        let render2d = Renderer2d::new(&device)?;

        let depth_stencil = wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };
        let rendertxt = GlyphonTextRenderer::new(&device, &queue, surface_config.format);

        let projection = Projection::FirstPerson;
        let mut camera = Camera::new(&device, width as f32, height as f32, 5.0, 0.4);

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

        let debug_mode =
            DebugMode::new(device, &mut shaders, &camera, &world.light, &surface_config)?;

        let bossman = debug_scene(
            &queue,
            &device,
            &mut models,
            &mut materials,
            &mut textures,
            &mut shaders,
            &mut pipelines,
            &mut world,
            &surface_config,
            depth_stencil.clone(),
        );

        camera.load_model(
            &queue,
            &device,
            &mut models,
            &mut materials,
            &mut textures,
            &mut shaders,
            &mut pipelines,
            &surface_config,
        );
        camera.spawn(&mut world);

        if let Some(material) = materials.get(&CacheKey::from("Material.001")) {
            world.generate_terrain(&queue, &device, *camera.eye(), 1, Medium::Ground, material)?;
        }

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
            queue,
            device,
            "cube-diffuse.jpg",
            surface_config.format,
        )?;
        let texture_bind_group = BindGroup::texture(device, &diffuse_texture.0);

        let mut menu = Menu::with_elements(
            &device,
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
        menu.build_pipeline(device, &surface_config, &mut pipelines.render)?;

        Ok(Rupy {
            queue: queue.clone(),
            device: device.clone(),
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
            materials,
            textures,
            shaders,
            pipelines,
            models,
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

    pub fn next_projection(&mut self) {
        self.projection = self.projection.next();
        self.camera.set_projection_far_near(&self.projection);
    }
    pub fn set_projection(&mut self, projection: Projection) {
        self.camera.set_projection_far_near(&projection);
        self.projection = projection;
    }
    fn dispatch(&self, event: ApplicationEvent) {
        self.tx.send(event).ok();
    }
    pub fn next_debug_mode(&mut self) {
        self.debug_mode
            .next_mode(&self.device, &self.camera, self.world.light());
    }
    pub fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        let width = new_size.width.max(1) as f32;
        let height = new_size.height.max(1) as f32;
        self.surface
            .resize(&self.device, &mut self.surface_config, width, height);
        self.camera.resize(width, height);
        self.menu.resize(&self.queue, &self.device, width, height);
        self.rendertxt.resize(&self.queue, width, height);
        self.render_targets.resize(&self.device, width, height);
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
                &self.queue,
                &self.device,
                &self.time,
                &mut self.camera,
                &self.projection,
                self.bossman,
            );
            let screen_size = self.window.inner_size();
            self.render3d.instances.update(
                &self.device,
                &self.world,
                &self.camera,
                (screen_size.width as f32, screen_size.height as f32),
                &self.projection,
                &mut self.models,
            );
            self.upload();
        }

        self.menu.update(
            self.camera.controller().last_mouse_pos(),
            self.camera.controller().mouse_just_pressed(),
        );

        self.render();
        self.window.request_redraw();
    }

    pub fn upload(&mut self) {
        let screen_size = self.window.inner_size();

        self.camera.upload(
            &self.queue,
            &self.device,
            self.projection,
            screen_size.height as f32,
            screen_size.width as f32,
        );
        self.world.upload(&self.queue, &self.device);
        self.render3d.instances.upload(&self.queue, &self.device);
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

        let mut encoder = self
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
                &mut self.models,
                &self.materials,
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
                .hdr(&self.device, &mut encoder, &scene_fb.color(), hdr_fb);
        }

        if let Some(hdr_fb) = self.render_targets.get(&RenderTargetKind::Hdr) {
            self.render3d.final_blit_to_surface(
                &self.device,
                &mut encoder,
                hdr_fb.color(),
                &surface_view,
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

            for item in self.text_regions() {
                self.rendertxt.queue_text(
                    &item.text,
                    item.pos[0],
                    item.pos[1],
                    glyphon::Color::rgb(255, 255, 255),
                );
            }
            self.menu.render(
                &mut rpass2d,
                &mut self.render2d,
                &mut self.rendertxt,
                &self.queue,
                &self.pipelines,
            );
            self.rendertxt.draw(
                &self.device,
                &self.queue,
                &self.surface_config,
                &mut rpass2d,
            );
            drop(rpass2d);
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
