use crate::app::{Resources, Rupy};
use core::EngineError;
use std::sync::Arc;
use winit::event_loop::ActiveEventLoop;

#[allow(dead_code)]
pub enum AppInnerState<'a> {
    Stopped(Arc<Resources>),
    Running(Rupy<'a>),
}

pub struct ApplicationState<'a> {
    pub inner: AppInnerState<'a>,
}

impl<'a> ApplicationState<'a> {
    /// Creates a new application state in the "stopped" (uninitialized) phase.
    pub fn new(resources: Resources) -> Self {
        Self {
            inner: AppInnerState::Stopped(resources.into()),
        }
    }

    /// One-time async initialization, called from `resumed()`.
    pub async fn init(&mut self, event_loop: &ActiveEventLoop) -> Result<(), EngineError> {
        match &self.inner {
            AppInnerState::Stopped(inner) => {
                self.inner = AppInnerState::Running(Rupy::new(event_loop, inner.clone()).await?);
            }
            _ => {}
        }

        Ok(())
    }
}
