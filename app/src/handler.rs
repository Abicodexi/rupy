use crate::state::{AppInnerState, ApplicationState};
use core::{ApplicationEvent, World};
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
            event_loop.exit();
        }

        if let AppInnerState::Running(app) = &mut self.inner {
            app.controller.process_events(&event);

            if let WindowEvent::Resized(new_size) = &event {
                app.resize(new_size);
            }

            if let WindowEvent::RedrawRequested = &event {
                app.update();
                app.window.request_redraw();
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ApplicationEvent) {
        if let AppInnerState::Running(app) = &mut self.inner {
            match event {
                ApplicationEvent::ShaderLoad(rel_path) => {}
                ApplicationEvent::WorldRequestRedraw => app.draw(),
                ApplicationEvent::Shutdown => {
                    World::stop();
                    event_loop.exit();
                }
            }
        }
    }
}
