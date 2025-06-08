use std::sync::Arc;

use crate::app::Rupy;
use crossbeam::channel::Sender;
use engine::{asset_service, ApplicationEvent, AssetRequest, EngineError};
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
    pub fn new(tx: Arc<Sender<ApplicationEvent>>, asset_tx: Arc<Sender<AssetRequest>>) -> Self {
        Self {
            inner: AppInnerState::Stopped(tx, asset_tx),
        }
    }

    pub async fn run(
        state: &mut ApplicationState,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), EngineError> {
        if let AppInnerState::Stopped(tx, asset_tx) = &state.inner {
            let rupy = Rupy::new(event_loop, asset_service(), tx.clone(), asset_tx.clone())?;
            state.inner = AppInnerState::Running(rupy);
        }
        Ok(())
    }
}
