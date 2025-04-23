mod app;
mod handler;
mod state;
use handler::ApplicationEvent;
use state::ApplicationState;
use winit::error::EventLoopError;

#[tokio::main]
async fn main() -> Result<(), EventLoopError> {
    winit::event_loop::EventLoop::<ApplicationEvent>::with_user_event()
        .build()?
        .run_app(&mut ApplicationState::new())?;
    Ok(())
}
