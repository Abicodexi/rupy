mod app;
mod handler;
mod state;
use crossbeam::channel::{unbounded, Receiver, Sender};
use engine::{
    event_bus::{EventBusProxy, EventProxy, EventProxyTrait},
    log_error,
    logger::LogFactory,
    service::AssetService,
    ApplicationEvent, AssetRequest, EngineError, GPU,
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
    GPU::init();
    let binding = GPU::get();
    let gpu = match binding.read() {
        Ok(g) => g,
        Err(e) => panic!("{}", e.to_string()),
    };

    let (tx, rx): (Sender<ApplicationEvent>, Receiver<ApplicationEvent>) = unbounded();
    let (asset_tx, asset_rx): (Sender<AssetRequest>, Receiver<AssetRequest>) = unbounded();

    let arc_rx = Arc::new(rx);
    let arc_tx = Arc::new(tx);

    let arc_asset_rx = Arc::new(asset_rx);
    let arc_asset_tx = Arc::new(asset_tx);

    let event_loop = EventLoop::<ApplicationEvent>::with_user_event().build()?;
    let event_proxy: Arc<dyn EventProxyTrait<ApplicationEvent> + Send + Sync> =
        Arc::new(EventProxy::new(event_loop.create_proxy()));

    let event_bus = EventBusProxy::new(&arc_rx, event_proxy);

    let mut state = ApplicationState::new(arc_tx, arc_asset_tx);

    event_bus.run_tokio();

    AssetService::spawn_thread(
        gpu.queue().clone(),
        gpu.device().clone(),
        arc_asset_rx.clone(),
    );
    if let Err(e) = event_loop.run_app(&mut state) {
        log_error!("{}", e);
    }

    drop(state);
    Ok(())
}
