use crate::state::{AppInnerState, ApplicationState};
use engine::{ApplicationEvent, World};
use pollster::FutureExt;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop};

impl winit::application::ApplicationHandler<ApplicationEvent> for ApplicationState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let AppInnerState::Stopped = self.inner {
            ApplicationState::init(self, event_loop)
                .block_on()
                .expect("State init on resume failed");

            event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let WindowEvent::CloseRequested = event {
            World::stop();
            event_loop.exit()
        }

        if let AppInnerState::Running(app) = &mut self.inner {
            match app.controller(&event) {
                engine::camera::Action::Projection => app.next_projection(),
                engine::camera::Action::Movement(..) => (),
            };

            if let WindowEvent::Resized(new_size) = &event {
                app.resize(new_size)
            }

            if let WindowEvent::RedrawRequested = &event {
                app.update();
                app.upload();
                app.render();
                app.window().request_redraw()
            }
            match event {
                WindowEvent::CursorEntered { .. } => {
                    let _ = app.window().set_cursor_visible(false);
                }
                WindowEvent::CursorLeft { .. } => {
                    let _ = app.window().set_cursor_visible(true);
                }
                _ => {}
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ApplicationEvent) {
        if let AppInnerState::Running(app) = &mut self.inner {
            match event {
                ApplicationEvent::Shutdown => {
                    World::stop();
                    event_loop.exit()
                }
                ApplicationEvent::Projection(projection) => {
                    app.set_projection(projection);
                }
            }
        }
    }
}
