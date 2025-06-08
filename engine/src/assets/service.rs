use crossbeam::channel::{unbounded, Receiver, Sender};
use once_cell::sync::OnceCell;
use std::sync::Arc;
use std::sync::RwLock;

use crate::{
    log_error, MaterialManager, ModelManager, PipelineManager, ShaderManager, TextureManager,
};

static GLOBAL_ASSET_SERVICE: OnceCell<Arc<AssetService>> = OnceCell::new();
static GLOBAL_ASSET_TX: OnceCell<Sender<AssetRequest>> = OnceCell::new();

pub enum AssetRequest {
    LoadTexture {
        texture: String,
        format: wgpu::TextureFormat,
    },
    LoadModel {/* all params */},
    LoadMaterial {/* all params */},
}

pub struct AssetService {
    pub queue: Arc<wgpu::Queue>,
    pub device: Arc<wgpu::Device>,
    pub materials: Arc<RwLock<MaterialManager>>,
    pub models: Arc<RwLock<ModelManager>>,
    pub textures: Arc<RwLock<TextureManager>>,
    pub shaders: Arc<RwLock<ShaderManager>>,
    pub pipelines: Arc<RwLock<PipelineManager>>,
}

impl AssetService {
    pub fn new(queue: Arc<wgpu::Queue>, device: Arc<wgpu::Device>) -> Self {
        Self {
            queue,
            device,
            materials: Arc::new(RwLock::new(MaterialManager::new())),
            models: Arc::new(RwLock::new(ModelManager::new())),
            textures: Arc::new(RwLock::new(TextureManager::new())),
            shaders: Arc::new(RwLock::new(ShaderManager::new())),
            pipelines: Arc::new(RwLock::new(PipelineManager::new())),
        }
    }
    // Immutable borrowors
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
}

pub fn spawn_asset_service_thread(
    queue: Arc<wgpu::Queue>,
    device: Arc<wgpu::Device>,
) -> Sender<AssetRequest> {
    let (tx, rx): (Sender<AssetRequest>, Receiver<AssetRequest>) = unbounded();
    let service = Arc::new(AssetService::new(queue, device));
    GLOBAL_ASSET_SERVICE.set(service.clone()).ok().unwrap();
    GLOBAL_ASSET_TX.set(tx.clone()).ok().unwrap();

    std::thread::spawn(move || {
        asset_service_thread(service, rx);
    });

    tx
}

fn asset_service_thread(service: Arc<AssetService>, rx: Receiver<AssetRequest>) {
    while let Ok(request) = rx.recv() {
        match request {
            AssetRequest::LoadTexture { texture, format } => {
                let mut textures = service.textures.write().unwrap();
                if let Err(e) =
                    textures.get_or_load_texture(&service.queue, &service.device, &texture, format)
                {
                    log_error!("{}", e.to_string());
                }
            }
            _ => {}
        }
    }
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
