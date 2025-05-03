use cgmath::{Deg, Point3, Vector3};
use core::{
    assets::{loader::AssetLoader, Assets},
    camera::{controller::CameraController, uniform::CameraUniform, Camera},
    error::EngineError,
    renderer::{Mesh, VertexTexture},
    texture::TextureManager,
    BindGroupLayouts, CacheKey, GlyphonBufferCache, GpuContext, Renderer, SurfaceExt, WgpuBuffer,
    WgpuBufferCache, WgpuRenderer,
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

#[allow(dead_code)]
pub struct Rupy<'a> {
    pub gpu: GpuContext,
    pub asset_loader: AssetLoader,
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'a>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub wgpu_renderer: WgpuRenderer,
    pub wgpu_buffer_cache: WgpuBufferCache,
    pub glyphon_buffer_cache: GlyphonBufferCache,
    pub bind_group_layouts: BindGroupLayouts,
    pub texture_manager: TextureManager,
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub camera_uniform: CameraUniform,
    pub camera_bind_group: wgpu::BindGroup,
    pub mesh: Mesh,
}

impl<'a> Rupy<'a> {
    pub async fn new(event_loop: &ActiveEventLoop) -> Result<Self, EngineError> {
        let gpu = GpuContext::new().await?;
        let asset_loader = AssetLoader::new(gpu.device.clone());
        let win_attrs = WindowAttributes::default().with_title("RupyEngine");
        let window = Arc::new(event_loop.create_window(win_attrs)?);
        let win_clone = Arc::clone(&window);

        let (width, height) = {
            let inenr_size = window.inner_size();
            (inenr_size.width, inenr_size.height)
        };

        let bind_group_layouts = BindGroupLayouts::new(&gpu.device);
        let mut wgpu_buffer_cache = WgpuBufferCache::new();
        let glyphon_buffer_cache = GlyphonBufferCache::new();

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

        let camera_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bg"),
            layout: &bind_group_layouts.camera,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu_buffer_cache
                    .get_or_create_buffer(&CacheKey::new("camera_uniform_buffer"), || {
                        WgpuBuffer::from_data(
                            &gpu.device,
                            &[camera_uniform],
                            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        )
                    })
                    .buffer
                    .as_entire_binding(),
            }],
        });

        let surface = gpu.instance.create_surface(win_clone)?;
        let surface_config = surface
            .get_default_config(&gpu.adapter, width, height)
            .ok_or(EngineError::SurfaceConfigError(
                "surface isn't supported by this adapter".into(),
            ))?;
        surface.configure(&gpu.device, &surface_config);

        let mut texture_manager = TextureManager::new(gpu.device.clone(), gpu.queue.clone());
        texture_manager
            .load(
                CacheKey::new("cube_diffuse"),
                &asset_loader,
                "cube-diffuse.jpg",
            )
            .await?;

        if let Err(e) = texture_manager.prepare_equirect_projection_textures(
            &asset_loader,
            &bind_group_layouts,
            "pure-sky.hdr",
            1080,
            wgpu::TextureFormat::Rgba32Float,
        ) {
            eprintln!("Error preparing cupemap textures: {}", e);
        };

        let wgpu_renderer =
            WgpuRenderer::new(&gpu, &asset_loader, &surface_config, &bind_group_layouts)?;
        let encoder = gpu.device.create_command_encoder(&Default::default());

        if let Some(equirect_bind_group) = texture_manager
            .bind_group_for("equirect_projection_src", &bind_group_layouts.equirect_src)
        {
            wgpu_renderer.equirect_projection(
                &gpu.queue,
                encoder,
                equirect_bind_group,
                1080,
                Some("equirect projection"),
            );
        }

        let mesh_buffer_key = CacheKey::new("mesh_vertex_buffer");
        wgpu_buffer_cache.get_or_create_buffer(&mesh_buffer_key, || {
            WgpuBuffer::from_data(&gpu.device, &VERTICES, wgpu::BufferUsages::VERTEX)
        });
        let mesh = Mesh::Shared {
            key: CacheKey::new("mesh_vertex_buffer"),
            count: VERTICES.len() as u32,
        };

        Ok(Rupy {
            gpu,
            asset_loader,
            window,
            surface,
            surface_config,
            wgpu_renderer,
            wgpu_buffer_cache,
            glyphon_buffer_cache,
            bind_group_layouts,
            texture_manager,
            camera,
            camera_controller,
            camera_uniform,
            camera_bind_group,
            mesh,
        })
    }
    pub fn assets(&self) -> Assets {
        Assets::new(&self.asset_loader)
    }
    pub fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        self.surface
            .resize(self.gpu.device(), &mut self.surface_config, *new_size);
        self.wgpu_renderer
            .resize(&self.surface_config, self.gpu.device());
    }

    pub fn draw(&mut self) {
        match self.surface.texture() {
            Ok(frame) => {
                self.wgpu_renderer.render(
                    &self.gpu,
                    frame,
                    &self.bind_group_layouts,
                    &mut self.texture_manager,
                    &mut self.wgpu_buffer_cache,
                    &self.camera_bind_group,
                    &self.mesh,
                );
                self.window.request_redraw();
            }
            Err(e) => {
                eprintln!("SurfaceError: {}", e);
            }
        };
    }

    pub fn update(&mut self) {
        self.camera_controller
            .update_camera(&mut self.camera, 1.0 / 60.0);
        self.camera_uniform.update_view_proj(&self.camera);
        self.gpu.queue.write_buffer(
            &self
                .wgpu_buffer_cache
                .get_or_create_buffer(&CacheKey::new("camera_uniform_buffer"), || {
                    WgpuBuffer::from_data(
                        self.gpu.device(),
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
