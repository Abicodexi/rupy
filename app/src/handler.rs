use crate::{
    app::Rupy,
    state::{AppInnerState, ApplicationState},
};
use engine::{log_error, ApplicationEvent, Dispatch, World};
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::Fullscreen};

impl winit::application::ApplicationHandler<ApplicationEvent> for ApplicationState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let AppInnerState::Stopped(tx, asset_tx, asset_rx) = &self.inner {
            match Rupy::new(
                event_loop,
                tx.to_owned(),
                asset_tx.to_owned(),
                asset_rx.to_owned(),
            ) {
                Ok(rupy) => {
                    rupy.dispatch(Dispatch::Event(ApplicationEvent::Start));
                    ApplicationState::run(self, rupy);
                    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
                }
                Err(e) => {
                    log_error!("{}", e.to_string());
                    World::stop();
                    event_loop.exit();
                    return;
                }
            };
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let AppInnerState::Running(app) = &mut self.inner {
            match &event {
                WindowEvent::CloseRequested => return app.shutdown(event_loop),
                WindowEvent::Resized(size) => return app.resize(size),
                WindowEvent::KeyboardInput { event, .. } => {
                    if event.state.is_pressed() && !event.repeat {
                        app.handle_key(event.physical_key);
                    }
                }
                WindowEvent::RedrawRequested => {
                    app.update();
                    app.upload();
                    app.redraw();
                }
                _ => {}
            }

            if let Some(new_proj) = app.projection.process(&event) {
                app.set_projection(new_proj);
            }

            app.camera.process(&event);
            app.handle_menu_toggle(&event);
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ApplicationEvent) {
        if let AppInnerState::Running(app) = &mut self.inner {
            match event {
                ApplicationEvent::Run => {
                    if !World::running() {
                        app.world.start();
                    }
                    if app.menu.is_visible() {
                        app.menu.hide();
                    }
                }
                ApplicationEvent::Stop => {
                    if World::running() {
                        World::stop()
                    }
                    if !app.menu.is_visible() {
                        app.menu.show();
                    }
                }
                ApplicationEvent::Start => {
                    if !app.menu.is_visible() {
                        app.menu.show();
                    }
                }
                ApplicationEvent::Shutdown => app.shutdown(event_loop),
                ApplicationEvent::ToggleFullscreen => {
                    if !app.window.is_resizable() {
                        return;
                    }
                    if app.window.fullscreen().is_some() {
                        app.window.set_fullscreen(None);
                    } else {
                        let fs = Fullscreen::Borderless(app.window.current_monitor());
                        app.window.set_fullscreen(Some(fs));
                    }
                }
            }
        }
    }
}
