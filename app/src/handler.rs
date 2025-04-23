use core::{gpu::global::get_global_gpu, renderer::traits::Renderer, SurfaceExt};

use crate::state::{AppInnerState, ApplicationState};
use winit::{event::WindowEvent, event_loop::ActiveEventLoop};

pub enum ApplicationEvent {/* custom events */}

impl<'a> winit::application::ApplicationHandler<ApplicationEvent> for ApplicationState<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let AppInnerState::Cold = self.inner {
            if let Err(e) = pollster::block_on(self.init(event_loop)) {
                panic!("Error on resume: {e}");
            };
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let WindowEvent::CloseRequested = event {
            event_loop.exit();
        }

        if let AppInnerState::Warm(app) = &mut self.inner {
            app.camera_controller.process_events(&event);
            let gpu = get_global_gpu();
            if let WindowEvent::Resized(new_size) = &event {
                app.surface
                    .resize(gpu.device(), &mut app.surface_config, *new_size);
                app.wgpu_renderer.resize(&app.surface_config);
            }
            if let WindowEvent::RedrawRequested = &event {
                match app.surface.texture() {
                    Ok(frame) => {
                        app.wgpu_renderer.render(
                            gpu,
                            frame,
                            &app.bind_group_layouts,
                            &app.texture_manager,
                            &app.camera_buffer,
                        );
                        app.window.request_redraw();
                    }
                    Err(e) => {
                        eprintln!("SurfaceError: {}", e);
                    }
                };
                app.camera_controller
                    .update_camera(&mut app.camera, 1.0 / 60.0);
                app.camera_uniform.update_view_proj(&app.camera);
                gpu.queue.write_buffer(
                    &app.camera_buffer,
                    0,
                    bytemuck::cast_slice(&[app.camera_uniform]),
                );
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ApplicationEvent) {
        if let AppInnerState::Warm(..) = &mut self.inner {}
    }
}
