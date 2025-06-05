use std::sync::Arc;

use crate::app::Rupy;
use crossbeam::channel::Sender;
use engine::{ApplicationEvent, EngineError};
use winit::event_loop::ActiveEventLoop;

#[allow(dead_code)]

pub enum AppInnerState {
    Stopped(Arc<Sender<ApplicationEvent>>),
    Running(Rupy),
}

pub struct ApplicationState {
    pub inner: AppInnerState,
}

impl ApplicationState {
    /// Creates a new application state in the "stopped" (uninitialized) phase.
    pub fn new(tx: Arc<Sender<ApplicationEvent>>) -> Self {
        Self {
            inner: AppInnerState::Stopped(tx),
        }
    }

    /// One-time async initialization, called from `resumed()`.
    pub async fn init(
        state: &mut ApplicationState,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), EngineError> {
        match &state.inner {
            AppInnerState::Stopped(tx) => {
                let run = Rupy::new(event_loop, tx.clone())?;
                state.inner = AppInnerState::Running(run);
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
