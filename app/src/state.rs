use core::EngineError;

use crate::app::Rupy;
use winit::event_loop::ActiveEventLoop;

#[allow(dead_code)]
pub enum AppInnerState<'a> {
    Cold,
    Warm(Rupy<'a>),
}

pub struct ApplicationState<'a> {
    pub inner: AppInnerState<'a>,
}
impl<'a> ApplicationState<'a> {
    pub fn new() -> Self {
        Self {
            inner: AppInnerState::Cold,
        }
    }
    pub async fn init(&mut self, event_loop: &ActiveEventLoop) -> Result<(), EngineError> {
        self.inner = AppInnerState::Warm(Rupy::new(event_loop).await?);
        Ok(())
    }
}
