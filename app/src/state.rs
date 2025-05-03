use crate::app::{PreRupy, Rupy};
use core::EngineError;
use winit::event_loop::ActiveEventLoop;

#[allow(dead_code)]
pub enum AppInnerState<'a> {
    Cold(PreRupy),
    Warm(Rupy<'a>),
}

pub struct ApplicationState<'a> {
    pub inner: AppInnerState<'a>,
}

impl<'a> ApplicationState<'a> {
    /// Creates a new application state in the "cold" (uninitialized) phase.
    pub fn new(pre: PreRupy) -> Self {
        Self {
            inner: AppInnerState::Cold(pre),
        }
    }

    /// One-time async initialization, called from `resumed()`.
    pub async fn init(&mut self, event_loop: &ActiveEventLoop) -> Result<(), EngineError> {
        match &self.inner {
            AppInnerState::Cold(inner) => {
                self.inner = AppInnerState::Warm(Rupy::new(event_loop, inner).await?);
            }
            _ => {}
        }

        Ok(())
    }
}
