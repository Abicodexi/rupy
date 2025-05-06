mod app;
mod handler;
mod state;
use core::{
    event_bus::{EventBusProxy, EventProxy, EventProxyTrait},
    logger::LogFactory,
    ApplicationEvent, EngineError, World, WorldTick, GPU,
};
use crossbeam::channel::{self, Receiver, Sender};
use state::ApplicationState;
use std::sync::Arc;
use winit::event_loop::EventLoop;

#[tokio::main]
async fn main() -> Result<(), EngineError> {
    #[cfg(feature = "logging")]
    {
        let logger = LogFactory::default();
        let _ = logger.init();
    }

    let (tx, rx): (Sender<ApplicationEvent>, Receiver<ApplicationEvent>) = channel::unbounded();

    let arc_tx = Arc::new(tx);
    let arc_rx = Arc::new(rx);

    let event_loop = EventLoop::<ApplicationEvent>::with_user_event().build()?;
    let proxy: Arc<dyn EventProxyTrait<ApplicationEvent> + Send + Sync> =
        Arc::new(EventProxy::new(Arc::new(event_loop.create_proxy())));

    GPU::init();
    World::init();
    WorldTick::run_tokio(&arc_tx);
    EventBusProxy::new(&arc_rx, proxy).run_tokio();
    let _ = core::ShaderHotReloader::watch(&arc_tx);
    let mut app = ApplicationState::new();
    Ok(event_loop.run_app(&mut app)?)
}
