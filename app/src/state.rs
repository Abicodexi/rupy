use std::sync::Arc;

use crate::app::Rupy;
use crossbeam::channel::{Receiver, Sender};
use engine::{ApplicationEvent, AssetRequest};

#[allow(dead_code)]

pub enum AppInnerState {
    Stopped(
        Arc<Sender<ApplicationEvent>>,
        Arc<Sender<AssetRequest>>,
        Arc<Receiver<AssetRequest>>,
    ),
    Running(Rupy),
}

pub struct ApplicationState {
    pub inner: AppInnerState,
}

impl ApplicationState {
    pub fn new(
        tx: Arc<Sender<ApplicationEvent>>,
        asset_tx: Arc<Sender<AssetRequest>>,
        asset_rx: Arc<Receiver<AssetRequest>>,
    ) -> Self {
        Self {
            inner: AppInnerState::Stopped(tx, asset_tx, asset_rx),
        }
    }

    pub fn run(state: &mut ApplicationState, rupy: Rupy) {
        if let AppInnerState::Stopped(..) = &state.inner {
            state.inner = AppInnerState::Running(rupy);
        }
    }
}
