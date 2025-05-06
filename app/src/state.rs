use crate::app::Rupy;
use core::EngineError;
use winit::event_loop::ActiveEventLoop;

#[allow(dead_code)]

pub enum AppInnerState {
    Stopped,
    Running(Rupy),
}

pub struct ApplicationState {
    pub inner: AppInnerState,
}

impl ApplicationState {
    /// Creates a new application state in the "stopped" (uninitialized) phase.
    pub fn new() -> Self {
        Self {
            inner: AppInnerState::Stopped,
        }
    }

    /// One-time async initialization, called from `resumed()`.
    pub async fn init(
        state: &mut ApplicationState,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), EngineError> {
        match state.inner {
            AppInnerState::Stopped => {
                let run = Rupy::new(event_loop)?;
                state.inner = AppInnerState::Running(run);
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
