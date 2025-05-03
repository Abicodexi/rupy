mod app;
mod handler;
mod state;
use app::Resources;
use core::{
    asset_dir,
    event_bus::{EventBusProxy, EventProxy, EventProxyTrait},
    log_error,
    logger::LogFactory,
    ApplicationEvent, AssetLoader, AssetWatcher, EngineError, GpuContext,
};
use crossbeam::channel::{self, Receiver, Sender};
use state::ApplicationState;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
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

    let event_loop = EventLoop::<ApplicationEvent>::with_user_event().build()?;
    let event_loop_proxy = Arc::new(event_loop.create_proxy());
    let event_proxy: Arc<dyn EventProxyTrait<ApplicationEvent> + Send + Sync> =
        Arc::new(EventProxy::new(event_loop_proxy));

    let event_bus_rx = rx.clone();
    let event_bus_proxy = event_proxy.clone();
    let event_bus = EventBusProxy::new(event_bus_rx, event_bus_proxy);

    let gpu = GpuContext::new().await?;

    let asset_loader = AssetLoader::new(gpu.device.clone())?;

    tokio::spawn(async move {
        event_bus.start().await;
    });
    let shader_dir = asset_dir()?.join("shaders");
    let reload_map = Arc::new(Mutex::new(HashMap::<String, Instant>::new()));
    let _shader_watcher = AssetWatcher::new(shader_dir.clone(), {
        let reload_map = Arc::clone(&reload_map);
        move |event| {
            for path in event.paths.iter() {
                let filename = path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or_default()
                    .to_string();

                let mut map = reload_map.lock().unwrap();
                let now = Instant::now();
                let last = map.entry(filename.clone()).or_insert(now);

                if now.duration_since(*last) > Duration::from_millis(2) {
                    *last = now;
                    if let Err(e) = arc_tx.send(ApplicationEvent::ShaderReload(filename.into())) {
                        log_error!("Error sending shader hot reload event: {}", e);
                    };
                }
            }
        }
    });

    let mut app = ApplicationState::new(Resources {
        gpu: gpu.into(),
        asset_loader: asset_loader.into(),
    });
    let _ = event_loop.run_app(&mut app);
    Ok(())
}
