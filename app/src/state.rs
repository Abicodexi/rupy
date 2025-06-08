use std::sync::Arc;

use crate::app::Rupy;
use crossbeam::channel::Sender;
use engine::{service::AssetRequest, ApplicationEvent, EngineError};
use winit::event_loop::ActiveEventLoop;

#[allow(dead_code)]

pub enum AppInnerState {
    Stopped(Arc<Sender<ApplicationEvent>>, Arc<Sender<AssetRequest>>),
    Running(Rupy),
}

pub struct ApplicationState {
    pub inner: AppInnerState,
}

impl ApplicationState {
    /// Creates a new application state in the "stopped" (uninitialized) phase.
    pub fn new(tx: Arc<Sender<ApplicationEvent>>, asset_tx: Arc<Sender<AssetRequest>>) -> Self {
        Self {
            inner: AppInnerState::Stopped(tx, asset_tx),
        }
    }

    /// One-time async initialization, called from `resumed()`.
    pub async fn init(
        state: &mut ApplicationState,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), EngineError> {
        if let AppInnerState::Stopped(tx, asset_tx) = &state.inner {
            let rupy = Rupy::new(event_loop, tx.clone(), asset_tx.clone())?;
            state.inner = AppInnerState::Running(rupy);
        }
        Ok(())
    }
}
