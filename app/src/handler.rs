use crate::state::{AppInnerState, ApplicationState};
use engine::ApplicationEvent;
use pollster::FutureExt;
use winit::{
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::Fullscreen,
};

impl winit::application::ApplicationHandler<ApplicationEvent> for ApplicationState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let AppInnerState::Stopped(..) = &self.inner {
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
                WindowEvent::KeyboardInput { event, .. } => {
                    if event.state.is_pressed() && event.repeat == false {
                        match event.physical_key {
                            PhysicalKey::Code(KeyCode::Tab) => {
                                if !app.window.is_resizable() {
                                    return;
                                }

                                let is_fullscreen = app.window.fullscreen().is_some();
                                let monitor = app.window.current_monitor();
                                app.window.set_fullscreen(match is_fullscreen {
                                    true => None,
                                    false => Some(Fullscreen::Borderless(monitor)),
                                });
                                app.window.set_cursor_visible(is_fullscreen);
                            }
                            PhysicalKey::Code(KeyCode::Numpad1) => {
                                let new_speed = (app.world.light().speed() + 0.1).clamp(0.1, 1.5);
                                app.world.light.set_speed(new_speed);
                            }
                            PhysicalKey::Code(KeyCode::Numpad2) => {
                                let new_speed = (app.world.light().speed() - 0.1).clamp(0.1, 1.5);
                                app.world.light.set_speed(new_speed);
                            }
                            PhysicalKey::Code(KeyCode::KeyQ) => match app.menu.visible() {
                                true => app.menu.hide(),
                                false => app.menu.show(),
                            },
                            PhysicalKey::Code(KeyCode::KeyM) => app.next_projection(),
                            PhysicalKey::Code(KeyCode::KeyP) => app.next_debug_mode(),
                            PhysicalKey::Code(KeyCode::KeyL) => {
                                let free_look = if app.camera.free_look() { false } else { true };
                                app.camera.set_free_look(free_look)
                            }
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
                ApplicationEvent::Shutdown => {
                    app.shutdown(event_loop);
                }
                ApplicationEvent::Projection => {
                    app.next_projection();
                }
                ApplicationEvent::MenuCallback(callback) => {
                    if callback == "Quit" {
                        app.menu.hide();
                    }
                }
            }
        }
    }
}
