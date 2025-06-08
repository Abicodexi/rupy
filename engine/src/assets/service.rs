use crossbeam::channel::{Receiver, Sender};
use once_cell::sync::OnceCell;
use pollster::FutureExt;
use std::sync::Arc;
use std::sync::RwLock;

use crate::create_render_pipeline;
use crate::log_info;
use crate::AssetRequest;
use crate::CacheStorage;
use crate::RenderBindGroupLayouts;
use crate::Vertex;
use crate::Vertex2d;
use crate::VertexInstance;
use crate::{
    log_error, MaterialManager, ModelManager, PipelineManager, ShaderManager, TextureManager,
};

static GLOBAL_ASSET_SERVICE: OnceCell<Arc<AssetService>> = OnceCell::new();
static GLOBAL_ASSET_TX: OnceCell<Sender<AssetRequest>> = OnceCell::new();

pub struct AssetService {
    pub queue: Arc<wgpu::Queue>,
    pub device: Arc<wgpu::Device>,
    pub bind_group_layouts: Arc<RenderBindGroupLayouts>,
    pub materials: Arc<RwLock<MaterialManager>>,
    pub models: Arc<RwLock<ModelManager>>,
    pub textures: Arc<RwLock<TextureManager>>,
    pub shaders: Arc<RwLock<ShaderManager>>,
    pub pipelines: Arc<RwLock<PipelineManager>>,
}

impl AssetService {
    pub fn new(
        queue: Arc<wgpu::Queue>,
        device: Arc<wgpu::Device>,
        materials: MaterialManager,
        models: ModelManager,
        textures: TextureManager,
        shaders: ShaderManager,
        pipelines: PipelineManager,
    ) -> Self {
        RenderBindGroupLayouts::init();
        let bind_group_layouts = RenderBindGroupLayouts::get().clone();
        Self {
            queue,
            device,
            bind_group_layouts,
            materials: Arc::new(RwLock::new(materials)),
            models: Arc::new(RwLock::new(models)),
            textures: Arc::new(RwLock::new(textures)),
            shaders: Arc::new(RwLock::new(shaders)),
            pipelines: Arc::new(RwLock::new(pipelines)),
        }
    }
    pub fn materials(&self) -> Arc<RwLock<MaterialManager>> {
        Arc::clone(&self.materials)
    }
    pub fn models(&self) -> Arc<RwLock<ModelManager>> {
        Arc::clone(&self.models)
    }
    pub fn textures(&self) -> Arc<RwLock<TextureManager>> {
        Arc::clone(&self.textures)
    }
    pub fn shaders(&self) -> Arc<RwLock<ShaderManager>> {
        Arc::clone(&self.shaders)
    }
    pub fn pipelines(&self) -> Arc<RwLock<PipelineManager>> {
        Arc::clone(&self.pipelines)
    }
    pub fn spawn_thread(
        queue: Arc<wgpu::Queue>,
        device: Arc<wgpu::Device>,
        rx: Arc<Receiver<AssetRequest>>,
    ) {
        let materials = MaterialManager::new();
        let models = ModelManager::new();
        let textures = TextureManager::new();
        let shaders = ShaderManager::new();
        let pipelines = PipelineManager::new();
        spawn_asset_service_thread(
            queue, device, rx, materials, models, textures, shaders, pipelines,
        );
    }
}

fn spawn_asset_service_thread(
    queue: Arc<wgpu::Queue>,
    device: Arc<wgpu::Device>,
    rx: Arc<Receiver<AssetRequest>>,
    materials: MaterialManager,
    models: ModelManager,
    textures: TextureManager,
    shaders: ShaderManager,
    pipelines: PipelineManager,
) {
    let service = Arc::new(AssetService::new(
        queue, device, materials, models, textures, shaders, pipelines,
    ));
    GLOBAL_ASSET_SERVICE.set(service.clone()).ok().unwrap();

    std::thread::spawn(move || {
        asset_service_thread(service, rx.clone());
    });
}

fn asset_service_thread(service: Arc<AssetService>, rx: Arc<Receiver<AssetRequest>>) {
    while let Ok(request) = rx.recv() {
        match request {
            AssetRequest::LoadShader { shader } => {
                if let Ok(mut shaders) = service.shaders.write() {
                    if let Err(e) = shaders.load(&service.device, &shader) {
                        log_error!("{}", e.to_string());
                    }
                }
            }
            AssetRequest::LoadRenderPipeline {
                layout,
                f_shader,
                v_shader,
                format,
                key,
                label,
            } => {
                if let Ok(mut pipelines) = service.pipelines.write() {
                    if !pipelines.render.contains(&key) {
                        if let Ok(mut shaders) = service.shaders.write() {
                            if let Ok(pipeline) = create_render_pipeline(
                                &service.device,
                                &mut shaders,
                                &f_shader,
                                &v_shader,
                                layout,
                                &[Vertex2d::LAYOUT],
                                format,
                                label,
                            ) {
                                pipelines.render.insert(key, pipeline.into());
                            }
                        }
                    }
                }
            }
            AssetRequest::LoadTexture { texture } => {
                if let Ok(mut textures) = service.textures.write() {
                    if let Err(e) = textures.load(&service.queue, &service.device, &texture) {
                        log_error!("{}", e.to_string());
                    } else {
                        log_info!("Loaded texture: {}", texture);
                    }
                }
            }
            AssetRequest::LoadMaterial {
                bind_group_layouts,
                mat,
                v_shader,
                f_shader,
                primitive,
                color_target,
                depth_stencil,
            } => {
                let mat_name = mat.name.clone();

                if let (Ok(mut materials), Ok(mut textures), Ok(mut pipelines), Ok(mut shaders)) = (
                    service.materials.write(),
                    service.textures.write(),
                    service.pipelines.write(),
                    service.shaders.write(),
                ) {
                    let format = color_target.format.clone();
                    if let Err(e) = materials.load_tobj(
                        &service.queue,
                        &service.device,
                        &mut textures,
                        &mut shaders,
                        &mut pipelines,
                        bind_group_layouts,
                        mat,
                        &v_shader,
                        &f_shader,
                        primitive,
                        color_target,
                        format,
                        &[Vertex::LAYOUT, VertexInstance::LAYOUT],
                        depth_stencil,
                    ) {
                        log_error!("{}", e.to_string());
                    } else {
                        log_info!("Loaded material: {}", mat_name);
                    }
                }
            }
            AssetRequest::LoadMaterialAsset {
                bind_group_layouts,
                asset,
                format,
            } => {
                let asset_name = asset.name.clone();

                if let (Ok(mut materials), Ok(mut textures), Ok(mut pipelines), Ok(mut shaders)) = (
                    service.materials.write(),
                    service.textures.write(),
                    service.pipelines.write(),
                    service.shaders.write(),
                ) {
                    if let Err(e) = materials.load_asset(
                        service.device.as_ref(),
                        service.queue.as_ref(),
                        &mut textures,
                        &mut shaders,
                        &mut pipelines,
                        bind_group_layouts,
                        asset,
                        format,
                        &[Vertex::LAYOUT, VertexInstance::LAYOUT],
                    ) {
                        log_error!("{}", e.to_string());
                    } else {
                        log_info!("Loaded material asset: {}", asset_name);
                    }
                }
            }
            AssetRequest::LoadModel {
                file,
                v_shader,
                f_shader,
                bind_group_layouts,
                primitive,
                color_target,
                depth_stencil,
            } => {
                if let (
                    Ok(mut models),
                    Ok(mut materials),
                    Ok(mut textures),
                    Ok(mut pipelines),
                    Ok(mut shaders),
                ) = (
                    service.models.write(),
                    service.materials.write(),
                    service.textures.write(),
                    service.pipelines.write(),
                    service.shaders.write(),
                ) {
                    if let Err(e) = models
                        .load(
                            &service.queue,
                            &service.device,
                            &mut materials,
                            &mut textures,
                            &mut shaders,
                            &mut pipelines,
                            &file,
                            &v_shader,
                            &f_shader,
                            &[Vertex::LAYOUT, VertexInstance::LAYOUT],
                            bind_group_layouts,
                            color_target.format,
                            primitive,
                            color_target,
                            depth_stencil,
                        )
                        .block_on()
                    {
                        log_error!("{}", e.to_string());
                    } else {
                        log_info!("Loaded model: {}", file);
                    }
                }
            }
            AssetRequest::LoadModelAsset {
                bind_group_layouts,
                format,
                asset,
            } => {
                if let (
                    Ok(mut models),
                    Ok(mut materials),
                    Ok(mut textures),
                    Ok(mut pipelines),
                    Ok(mut shaders),
                ) = (
                    service.models.write(),
                    service.materials.write(),
                    service.textures.write(),
                    service.pipelines.write(),
                    service.shaders.write(),
                ) {
                    if let Err(e) = models.load_asset(
                        &service.queue,
                        &service.device,
                        &mut materials,
                        &mut textures,
                        &mut shaders,
                        &mut pipelines,
                        bind_group_layouts,
                        format,
                        &[Vertex::LAYOUT, VertexInstance::LAYOUT],
                        asset,
                    ) {
                        log_error!("{}", e.to_string());
                    }
                } else {
                    log_info!("Loaded model asset: {}", asset.name);
                }
            }
        }
    }
    log_info!("Asset service thread exiting");
}

pub fn asset_service() -> &'static Arc<AssetService> {
    GLOBAL_ASSET_SERVICE
        .get()
        .expect("AssetService not initialized!")
}
pub fn asset_request_tx() -> &'static Sender<AssetRequest> {
    GLOBAL_ASSET_TX
        .get()
        .expect("AssetService not initialized!")
}
