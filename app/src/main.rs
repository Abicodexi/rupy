mod app;
mod handler;
mod state;
use crossbeam::channel::{unbounded, Receiver, Sender};
use engine::{
    event_bus::{EventBusProxy, EventProxy, EventProxyTrait},
    logger::LogFactory,
    service::{AssetRequest, AssetService},
    ApplicationEvent, EngineError, MaterialManager, ModelManager, PipelineManager,
    RenderBindGroupLayouts, ShaderManager, TextureManager, GPU,
};
use state::ApplicationState;
use std::sync::Arc;
use winit::event_loop::EventLoop;

#[tokio::main]
async fn main() -> Result<(), EngineError> {
    #[cfg(feature = "logging")]
    {
        let _ = LogFactory::default().init();
    }

    let (tx, rx): (Sender<ApplicationEvent>, Receiver<ApplicationEvent>) = unbounded();
    let (asset_tx, asset_rx): (Sender<AssetRequest>, Receiver<AssetRequest>) = unbounded();

    let arc_rx = Arc::new(rx);
    let arc_tx = Arc::new(tx);

    let arc_asset_rx = Arc::new(asset_rx);
    let arc_asset_tx = Arc::new(asset_tx);

    let event_loop = EventLoop::<ApplicationEvent>::with_user_event().build()?;
    let proxy: Arc<dyn EventProxyTrait<ApplicationEvent> + Send + Sync> =
        Arc::new(EventProxy::new(Arc::new(event_loop.create_proxy())));

    GPU::init();
    let binding = GPU::get();

    let gpu = binding
        .read()
        .expect("GPU resources must exist at this point");

    let _ = RenderBindGroupLayouts::get();
    let materials = MaterialManager::new();
    let models = ModelManager::new();
    let textures = TextureManager::new();
    let shaders = ShaderManager::new();
    let pipelines = PipelineManager::new();

    EventBusProxy::new(&arc_rx, proxy).run_tokio();

    AssetService::spawn_thread(
        gpu.queue().clone(),
        gpu.device().clone(),
        arc_asset_rx.clone(),
        materials,
        models,
        textures,
        shaders,
        pipelines,
    );
    Ok(event_loop.run_app(&mut ApplicationState::new(arc_tx, arc_asset_tx))?)
}
