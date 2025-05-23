use crate::state::{AppInnerState, ApplicationState};
use engine::{ApplicationEvent, World};
use pollster::FutureExt;
use winit::{
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
};

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
        _: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let AppInnerState::Running(app) = &mut self.inner {
            if matches!(event, WindowEvent::CloseRequested) {
                app.shutdown(event_loop)
            }
            app.input(&event);
            match &event {
                WindowEvent::Resized(size) => app.resize(&size),
                WindowEvent::CursorEntered { .. } => app.window().set_cursor_visible(false),
                WindowEvent::CursorLeft { .. } => app.window().set_cursor_visible(true),

                WindowEvent::KeyboardInput { event, .. } => {
                    if event.state.is_pressed() && event.repeat == false {
                        match event.physical_key {
                            PhysicalKey::Code(KeyCode::KeyM) => app.next_projection(),
                            PhysicalKey::Code(KeyCode::KeyP) => app.next_debug_mode(),
                            PhysicalKey::Code(KeyCode::KeyL) => {
                                let free_look = if app.cam().free_look() { false } else { true };
                                app.cam_mut().set_free_look(free_look)
                            }
                            PhysicalKey::Code(KeyCode::Escape) => app.shutdown(event_loop),
                            _ => {}
                        }
                    }
                }
                WindowEvent::RedrawRequested => {
                    app.update();
                    app.upload();
                    app.render();
                    app.window().request_redraw();
                }
                _ => {}
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
                ApplicationEvent::Projection => {
                    app.next_projection();
                }
            }
        }
    }
}
