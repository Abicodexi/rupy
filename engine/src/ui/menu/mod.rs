use winit::{
    event::WindowEvent,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{
    camera::OrthoUniform,
    container::UiContainer,
    gfx::{
        bind_group::{ortho_uniform_group, ortho_uniform_layout, sprite_2d_array_layout},
        buffer::WgpuBuffer,
    },
    menu::{menu_builder::MenuBuilder, menu_element::MenuElement},
    AssetService, CacheKey, EngineError, GlyphonTextRenderer, Renderer2d, UiElement, UiElements,
    UiEvent, Vertex2d,
};
use std::sync::Arc;
pub mod menu_builder;
pub mod menu_button;
pub mod menu_element;

pub struct Menu {
    buffer: WgpuBuffer,
    bind_group: Arc<wgpu::BindGroup>,
    texture_bind_group: Option<Arc<wgpu::BindGroup>>,
    pipeline: Arc<wgpu::RenderPipeline>,
    root: UiContainer,
    is_visible: bool,
}

impl Menu {
    pub fn new(
        service: &AssetService,
        surface_config: &wgpu::SurfaceConfiguration,
        screen_w: u32,
        screen_h: u32,
        container: UiContainer,
    ) -> Result<Self, EngineError> {
        let root = container;

        let ortho_uniform = OrthoUniform::new(screen_w as f32, screen_h as f32);
        let buffer = WgpuBuffer::from_data(
            service.device(),
            bytemuck::bytes_of(&ortho_uniform),
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            Some("Menu Ortho Buffer"),
        );

        let bind_group = service.get_or_create_bind_group("orthographic".into(), || {
            Ok(ortho_uniform_group(
                service.device(),
                &ortho_uniform_layout(service.device()),
                &buffer,
            )
            .into())
        })?;

        let pipeline_name = "sprite2d";
        let pipeline_key = CacheKey::from(pipeline_name);
        let pipeline_layout =
            service
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(&format!("{} layout", pipeline_name)),
                    bind_group_layouts: &[
                        &ortho_uniform_layout(service.device()),
                        &sprite_2d_array_layout(service.device()),
                    ],
                    push_constant_ranges: &[],
                });

        if let Some(pipeline) = service.get_or_load_render_pipeline(
            "sprite2d.frag.wgsl",
            "sprite2d.vert.wgsl",
            pipeline_layout,
            &[Vertex2d::LAYOUT],
            surface_config.format,
            None,
            pipeline_key,
            format!("{} pipeline", pipeline_name),
        ) {
            Ok(Menu {
                buffer,
                bind_group,
                root,
                is_visible: false,
                pipeline,
                texture_bind_group: None,
            })
        } else {
            Err(EngineError::AssetLoadError(format!(
                "Failed to build pipeline {}",
                pipeline_name
            )))
        }
    }

    pub fn builder(
        surface_config: &wgpu::SurfaceConfiguration,
        screen_w: u32,
        screen_h: u32,
    ) -> MenuBuilder {
        MenuBuilder::new(surface_config, screen_w, screen_h)
    }
    pub fn with_elements(
        service: &Arc<AssetService>,
        surface_config: &wgpu::SurfaceConfiguration,
        screen_w: u32,
        screen_h: u32,
        elements: Vec<MenuElement>,
        container: UiContainer,
    ) -> Result<Self, EngineError> {
        let mut menu = Self::new(service, surface_config, screen_w, screen_h, container)?;
        for element in elements {
            menu.add_element(element);
        }
        Ok(menu)
    }
    pub fn resize(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        screen_w: f32,
        screen_h: f32,
    ) {
        self.buffer.write_data(
            queue,
            device,
            &[OrthoUniform::new(screen_w, screen_h)],
            None,
        );
    }
    pub fn process(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { event, .. } => match event.physical_key {
                PhysicalKey::Code(KeyCode::F1) => {
                    if !event.repeat && event.state.is_pressed() {
                        match self.is_visible() {
                            true => {
                                self.hide();
                            }
                            false => {
                                self.show();
                            }
                        }
                    }
                    {}
                }

                _ => {}
            },
            _ => {}
        }
        self.is_visible
    }
    pub fn render<'a>(
        &'a mut self,
        service: &AssetService,
        rpass: &mut wgpu::RenderPass<'a>,
        d2: &mut Renderer2d,
        txt: &mut GlyphonTextRenderer,
    ) {
        if !self.is_visible {
            return;
        }

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, self.bind_group.as_ref(), &[]);

        if let Some(arr_tex_bg) = &self.texture_bind_group {
            rpass.set_bind_group(1, arr_tex_bg.as_ref(), &[]);
        }

        for elem in self.root.elements() {
            elem.draw(d2, txt);
        }

        d2.flush(service.queue(), rpass);
    }

    pub fn set_texture_bind_group(&mut self, bind_group: wgpu::BindGroup) {
        self.texture_bind_group = Some(bind_group.into());
    }
    pub fn texture_bind_group(&self) -> Option<&Arc<wgpu::BindGroup>> {
        self.texture_bind_group.as_ref()
    }
    pub fn add_element(&mut self, element: MenuElement) {
        self.root.push_element(UiElement::Menu(element));
    }

    pub fn update(
        &mut self,
        mouse_position: Option<(f32, f32)>,
        clicked: (bool, bool),
    ) -> Vec<UiEvent> {
        let mut events = Vec::new();
        for elem in self.root.elements_mut() {
            match elem {
                UiElement::Menu(menu_element) => {
                    if let Some(ev) = menu_element.update(mouse_position, clicked) {
                        events.push(ev);
                    }
                }
            }
        }

        events
    }
    pub fn get_element(&self, id: u32) -> Option<&UiElement> {
        self.root.get_element(id)
    }
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }
    pub fn show(&mut self) {
        self.is_visible = true;
    }
    pub fn hide(&mut self) {
        self.is_visible = false;
    }
}
