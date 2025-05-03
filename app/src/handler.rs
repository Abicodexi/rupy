use crate::state::{AppInnerState, ApplicationState};
use core::{ApplicationEvent, WgpuRenderer};
use winit::{event::WindowEvent, event_loop::ActiveEventLoop};

impl<'a> winit::application::ApplicationHandler<ApplicationEvent> for ApplicationState<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let AppInnerState::Stopped(..) = self.inner {
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

        if let AppInnerState::Running(app) = &mut self.inner {
            app.controller.process_events(&event);

            if let WindowEvent::Resized(new_size) = &event {
                app.resize(new_size);
            }

            if let WindowEvent::RedrawRequested = &event {
                app.update();
                app.draw();
            }
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: ApplicationEvent) {
        if let AppInnerState::Running(app) = &mut self.inner {
            match event {
                ApplicationEvent::ShaderReload(rel_path) => {
                    app.managers.shader_manager.reload_shader(&rel_path);

                    if let Ok(renderer_reload) = WgpuRenderer::new(
                        &app.resources.gpu,
                        &app.resources.asset_loader,
                        &mut app.managers.shader_manager,
                        &mut app.managers.pipeline_manager,
                        &app.surface_config,
                        &app.bind_group_layouts,
                    ) {
                        app.wgpu_renderer = renderer_reload;
                    }
                }
            }
        }
    }
}
