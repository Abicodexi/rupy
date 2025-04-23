use cgmath::{Deg, Point3, Vector3};
use core::{
    camera::{controller::CameraController, uniform::CameraUniform, Camera},
    error::EngineError,
    gpu::global::get_global_gpu,
    renderer::wgpu_renderer::WgpuRenderer,
    texture::TextureManager,
    BindGroupLayouts, CacheKey,
};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::{
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

#[allow(dead_code)]
pub struct Rupy<'a> {
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'a>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub wgpu_renderer: WgpuRenderer,
    pub bind_group_layouts: BindGroupLayouts,
    pub texture_manager: TextureManager,
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
}

impl<'a> Rupy<'a> {
    pub async fn new(event_loop: &ActiveEventLoop) -> Result<Self, EngineError> {
        let gpu = get_global_gpu();
        let (adapter, instance) = { (gpu.adapter(), gpu.instance()) };

        let win_attrs = WindowAttributes::default().with_title("RupyEngine");
        let window = Arc::new(event_loop.create_window(win_attrs)?);
        let win_clone = Arc::clone(&window);

        let (width, height) = {
            let inenr_size = window.inner_size();
            (inenr_size.width, inenr_size.height)
        };

        let bind_group_layouts = BindGroupLayouts::new(gpu.device());

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

        let camera_buffer = gpu
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let mut texture_manager = TextureManager::new(gpu.device.clone(), gpu.queue.clone());
        texture_manager
            .load(
                CacheKey::new("cube_diffuse"),
                "C:\\Users\\abism\\Desktop\\rupy\\cube-diffuse.jpg",
            )
            .await?;

        let surface = instance.create_surface(win_clone)?;
        let surface_config = surface.get_default_config(adapter, width, height).ok_or(
            EngineError::SurfaceConfigError("surface isn't supported by this adapter".into()),
        )?;
        surface.configure(&gpu.device, &surface_config);

        let wgpu_renderer = WgpuRenderer::new(&gpu, &surface_config, &bind_group_layouts)?;

        Ok(Rupy {
            window,
            surface,
            surface_config,
            wgpu_renderer,
            bind_group_layouts,
            texture_manager,
            camera,
            camera_controller,
            camera_uniform,
            camera_buffer,
        })
    }
}
