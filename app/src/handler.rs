use crate::state::{AppInnerState, ApplicationState};
use winit::{event::WindowEvent, event_loop::ActiveEventLoop};
pub enum ApplicationEvent {/* custom events */}

impl<'a> winit::application::ApplicationHandler<ApplicationEvent> for ApplicationState<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let AppInnerState::Cold = self.inner {
            pollster::block_on(self.init(event_loop)).expect("Init failed");
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let WindowEvent::CloseRequested = event {
            event_loop.exit();
        }

        if let AppInnerState::Warm(app) = &mut self.inner {
            app.camera_controller.process_events(&event);

            if let WindowEvent::Resized(new_size) = &event {
                app.resize(new_size);
            }

            if let WindowEvent::RedrawRequested = &event {
                app.update();
                app.draw();
            }
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: ApplicationEvent) {
        if let AppInnerState::Warm(..) = &mut self.inner {}
    }
}
