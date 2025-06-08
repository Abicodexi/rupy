use crate::state::{AppInnerState, ApplicationState};
use engine::{camera::Projection, log_error, ApplicationEvent, World};
use winit::{
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::Fullscreen,
};

impl winit::application::ApplicationHandler<ApplicationEvent> for ApplicationState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let AppInnerState::Stopped(..) = &self.inner {
            if let Err(e) = pollster::block_on(ApplicationState::run(self, event_loop)) {
                log_error!("{}", e.to_string());
                World::stop();
                event_loop.exit();
            } else {
                event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
            }
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

            app.menu.process(&event);

            if app.menu.is_visible() && !app.camera.is_frozen() {
                app.camera.freeze();
                app.set_projection(Projection::Orthographic);
            } else if !app.menu.is_visible() && app.camera.is_frozen() {
                app.camera.unfreeze();
                app.set_projection(Projection::FirstPerson);
            }

            app.camera.process(&event);

            match &event {
                WindowEvent::Resized(size) => app.resize(&size),
                WindowEvent::KeyboardInput { event, .. } => {
                    if event.state.is_pressed() && event.repeat == false {
                        match event.physical_key {
                            PhysicalKey::Code(KeyCode::Tab) => {
                                if !app.window.is_resizable() {
                                    return;
                                }
                                if app.window.fullscreen().is_some() {
                                    app.window.set_cursor_visible(true);
                                    app.window.set_fullscreen(None);
                                } else {
                                    app.window.set_fullscreen(Some(Fullscreen::Borderless(
                                        app.window.current_monitor(),
                                    )))
                                }
                            }
                            PhysicalKey::Code(KeyCode::Numpad1) => {
                                let new_speed = (app.world.light().speed() + 0.1).clamp(0.1, 1.5);
                                app.world.light.set_speed(new_speed);
                            }
                            PhysicalKey::Code(KeyCode::Numpad2) => {
                                let new_speed = (app.world.light().speed() - 0.1).clamp(0.1, 1.5);
                                app.world.light.set_speed(new_speed);
                            }
                            PhysicalKey::Code(KeyCode::KeyM) => app.next_projection(),
                            PhysicalKey::Code(KeyCode::KeyP) => app.next_debug_mode(),

                            PhysicalKey::Code(KeyCode::Escape) => app.shutdown(event_loop),
                            _ => {}
                        }
                    }
                }
                WindowEvent::RedrawRequested => app.redraw(),

                _ => {}
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ApplicationEvent) {
        if let AppInnerState::Running(app) = &mut self.inner {
            match event {
                ApplicationEvent::Shutdown => app.shutdown(event_loop),
                ApplicationEvent::Projection => app.next_projection(),
                ApplicationEvent::MenuCallback(callback) => {
                    if callback == "Quit" {
                        app.menu.hide();
                    }
                }
            }
        }
    }
}
