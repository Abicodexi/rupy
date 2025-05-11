mod app;
mod handler;
mod state;
use crossbeam::channel::{self, Receiver, Sender};
use engine::{
    event_bus::{EventBusProxy, EventProxy, EventProxyTrait},
    logger::LogFactory,
    ApplicationEvent, BindGroupLayouts, EngineError, World, WorldTick, GPU,
};
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

    let (_tx, rx): (Sender<ApplicationEvent>, Receiver<ApplicationEvent>) = channel::unbounded();

    let arc_rx = Arc::new(rx);

    let event_loop = EventLoop::<ApplicationEvent>::with_user_event().build()?;
    let proxy: Arc<dyn EventProxyTrait<ApplicationEvent> + Send + Sync> =
        Arc::new(EventProxy::new(Arc::new(event_loop.create_proxy())));

    GPU::init();
    World::init();

    WorldTick::run_tokio();
    EventBusProxy::new(&arc_rx, proxy).run_tokio();

    let _ = BindGroupLayouts::get();

    Ok(event_loop.run_app(&mut ApplicationState::new())?)
}
