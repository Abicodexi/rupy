use crate::state::{AppInnerState, ApplicationState};
use core::{ApplicationEvent, WgpuRenderer};
use winit::{event::WindowEvent, event_loop::ActiveEventLoop};

impl<'a> winit::application::ApplicationHandler<ApplicationEvent> for ApplicationState<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let AppInnerState::Stopped(..) = self.inner {
            pollster::block_on(self.init(event_loop)).expect("Init failed");
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
            event_loop.exit();
        }

        if let AppInnerState::Running(app) = &mut self.inner {
            app.controller.process_events(&event);

            if let WindowEvent::Resized(new_size) = &event {
                app.resize(new_size);
            }

            if let WindowEvent::RedrawRequested = &event {
                app.update();
                app.draw();
                app.window.request_redraw();
            }
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: ApplicationEvent) {
        if let AppInnerState::Running(app) = &mut self.inner {
            match event {
                ApplicationEvent::ShaderReload(rel_path) => {
                    app.world
                        .managers_mut()
                        .shader_manager
                        .reload_shader(&rel_path);
                    app.wgpu_renderer = WgpuRenderer::new();
                }
            }
        }
    }
}
