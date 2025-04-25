use crate::app::Rupy;
use crate::handler::ApplicationEvent;
use core::EngineError;
use winit::{
    error::EventLoopError,
    event_loop::{ActiveEventLoop, EventLoop},
};

#[allow(dead_code)]
pub enum AppInnerState<'a> {
    Cold,
    Warm(Rupy<'a>),
}

pub struct ApplicationState<'a> {
    pub inner: AppInnerState<'a>,
}

impl<'a> ApplicationState<'a> {
    /// Creates a new application state in the "cold" (uninitialized) phase.
    pub fn new() -> Self {
        Self {
            inner: AppInnerState::Cold,
        }
    }

    /// One-time async initialization, called from `resumed()`.
    pub async fn init(&mut self, event_loop: &ActiveEventLoop) -> Result<(), EngineError> {
        self.inner = AppInnerState::Warm(Rupy::new(event_loop).await?);
        Ok(())
    }
}

pub fn run_app() -> Result<(), EventLoopError> {
    let event_loop = EventLoop::<ApplicationEvent>::with_user_event().build()?;
    let mut app = ApplicationState::new();
    event_loop.run_app(&mut app)
}
