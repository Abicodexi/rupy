use cgmath::{Deg, Point3, Vector3};
use core::{
    assets::loader::AssetLoader,
    buffer::BufferManager,
    camera::{controller::CameraController, uniform::CameraUniform, Camera},
    error::EngineError,
    log_error,
    pipeline::PipelineManager,
    renderer::{Mesh, VertexTexture},
    texture::TextureManager,
    BindGroupLayouts, CacheKey, GpuContext, Renderer, ShaderManager, SurfaceExt, WgpuBuffer,
    WgpuRenderer,
};
use std::sync::Arc;
use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

const VERTICES: [VertexTexture; 3] = [
    VertexTexture {
        position: [-0.5, -0.5, 0.0],
        color: [1.0, 0.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    VertexTexture {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
    VertexTexture {
        position: [0.0, 0.5, 0.0],
        color: [0.0, 0.0, 1.0],
        tex_coords: [0.0, 1.0],
    },
];

pub struct Resources {
    pub gpu: Arc<GpuContext>,
    pub asset_loader: Arc<AssetLoader>,
}

pub struct Managers {
    pub shader_manager: ShaderManager,
    pub pipeline_manager: PipelineManager,
    pub buffer_manager: BufferManager,
}

#[allow(dead_code)]
pub struct Rupy<'a> {
    pub resources: Arc<Resources>,
    pub managers: Managers,
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'a>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub wgpu_renderer: WgpuRenderer,
    pub bind_group_layouts: BindGroupLayouts,
    pub texture_manager: TextureManager,
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub camera_uniform: CameraUniform,
    pub camera_bind_group: wgpu::BindGroup,
    pub mesh: Mesh,
}

impl<'a> Rupy<'a> {
    pub async fn new(
        event_loop: &ActiveEventLoop,
        resources: Arc<Resources>,
    ) -> Result<Self, EngineError> {
        let win_attrs = WindowAttributes::default().with_title("RupyEngine");
        let window = Arc::new(event_loop.create_window(win_attrs)?);
        let win_clone = Arc::clone(&window);

        let mut shader_manager = ShaderManager::new(resources.asset_loader.clone());
        let mut pipeline_manager = PipelineManager::new();
        let mut buffer_manager = BufferManager::new();

        let (width, height) = {
            let inenr_size = window.inner_size();
            (inenr_size.width, inenr_size.height)
        };

        let bind_group_layouts = BindGroupLayouts::new(&resources.gpu.device);

        let camera = Camera {
            eye: Point3::new(0.0, 1.0, 2.0),
            target: Point3::new(0.0, 0.0, 0.0),
            up: Vector3::unit_y(),
            aspect: width as f32 / height as f32,
            fovy: Deg(45.0),
            znear: 0.1,
            zfar: 100.0,
        };
        let camera_controller = CameraController::new(1.0, 0.5);
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_bind_group =
            resources
                .gpu
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("camera_bg"),
                    layout: &bind_group_layouts.camera,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer_manager
                            .w_buffer
                            .get_or_create(&CacheKey::new("camera_uniform_buffer"), || {
                                WgpuBuffer::from_data(
                                    &resources.gpu.device,
                                    &[camera_uniform],
                                    wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                                )
                            })
                            .buffer
                            .as_entire_binding(),
                    }],
                });

        let surface = resources.gpu.instance.create_surface(win_clone)?;
        let surface_config = surface
            .get_default_config(&resources.gpu.adapter, width, height)
            .ok_or(EngineError::SurfaceConfigError(
                "surface isn't supported by this adapter".into(),
            ))?;
        surface.configure(&resources.gpu.device, &surface_config);

        let mut texture_manager =
            TextureManager::new(resources.gpu.device.clone(), resources.gpu.queue.clone());
        texture_manager
            .load(
                CacheKey::new("cube_diffuse"),
                &resources.asset_loader,
                "cube-diffuse.jpg",
            )
            .await?;

        if let Err(e) = texture_manager.prepare_equirect_projection_textures(
            &resources.asset_loader,
            &bind_group_layouts,
            "pure-sky.hdr",
            1080,
            wgpu::TextureFormat::Rgba32Float,
        ) {
            log_error!("Error preparing cupemap textures: {}", e);
        };

        let wgpu_renderer = WgpuRenderer::new(
            &resources.gpu,
            &resources.asset_loader,
            &mut shader_manager,
            &mut pipeline_manager,
            &surface_config,
            &bind_group_layouts,
        )?;
        let encoder = resources
            .gpu
            .device
            .create_command_encoder(&Default::default());

        if let Some(equirect_bind_group) = texture_manager
            .bind_group_for("equirect_projection_src", &bind_group_layouts.equirect_src)
        {
            wgpu_renderer.equirect_projection(
                &resources.gpu.queue,
                encoder,
                equirect_bind_group,
                1080,
                Some("equirect projection"),
            );
        }

        let mesh_buffer_key = CacheKey::new("mesh_vertex_buffer");
        buffer_manager.w_buffer.get_or_create(&mesh_buffer_key, || {
            WgpuBuffer::from_data(&resources.gpu.device, &VERTICES, wgpu::BufferUsages::VERTEX)
        });
        let mesh = Mesh::Shared {
            key: CacheKey::new("mesh_vertex_buffer"),
            count: VERTICES.len() as u32,
        };

        Ok(Rupy {
            resources,
            managers: Managers {
                shader_manager,
                pipeline_manager,
                buffer_manager,
            },
            window,
            surface,
            surface_config,
            wgpu_renderer,
            bind_group_layouts,
            texture_manager,
            camera,
            camera_controller,
            camera_uniform,
            camera_bind_group,
            mesh,
        })
    }

    pub fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        self.surface.resize(
            self.resources.gpu.device(),
            &mut self.surface_config,
            *new_size,
        );
        self.wgpu_renderer
            .resize(&self.surface_config, self.resources.gpu.device());
    }

    pub fn draw(&mut self) {
        match self.surface.texture() {
            Ok(frame) => {
                self.wgpu_renderer.render(
                    &self.resources.gpu,
                    frame,
                    &self.bind_group_layouts,
                    &mut self.texture_manager,
                    &mut self.managers.buffer_manager.w_buffer,
                    &self.camera_bind_group,
                    &self.mesh,
                );
                self.window.request_redraw();
            }
            Err(e) => {
                log_error!("SurfaceError: {}", e);
            }
        };
    }

    pub fn update(&mut self) {
        self.camera_controller
            .update_camera(&mut self.camera, 1.0 / 60.0);
        self.camera_uniform.update_view_proj(&self.camera);
        self.resources.gpu.queue.write_buffer(
            &self
                .managers
                .buffer_manager
                .w_buffer
                .get_or_create(&CacheKey::new("camera_uniform_buffer"), || {
                    WgpuBuffer::from_data(
                        self.resources.gpu.device(),
                        &[self.camera_uniform],
                        wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    )
                })
                .buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }
}
