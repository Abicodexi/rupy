use crate::{
    camera::OrthoUniform, container::UiContainer, menu_builder::MenuBuilder,
    menu_element::MenuElement, AssetService, BindGroup, CacheKey, EngineError, GlyphonTextRenderer,
    Renderer2d, UiElement, UiEvent, Vertex2d, WgpuBuffer,
};
use std::sync::Arc;
use winit::{
    event::WindowEvent,
    keyboard::{KeyCode, PhysicalKey},
};

pub struct Menu {
    buffer: WgpuBuffer,
    bind_group: Arc<wgpu::BindGroup>,
    texture_bind_group: Arc<wgpu::BindGroup>,
    pipeline: Arc<wgpu::RenderPipeline>,
    root: UiContainer,
    is_visible: bool,
}

impl Menu {
    pub fn new(
        service: &AssetService,
        surface_config: &wgpu::SurfaceConfiguration,
        texture: &str,
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
            Ok(BindGroup::ortho_uniform(service.device(), &buffer).into())
        })?;

        let pipeline_name = "sprite2d";
        let pipeline_key = CacheKey::from(pipeline_name);
        let pipeline_layout =
            service
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(&format!("{} layout", pipeline_name)),
                    bind_group_layouts: &[
                        &service.bind_group_layouts().ortho_uniform,
                        &service.bind_group_layouts().diffuse,
                    ],
                    push_constant_ranges: &[],
                });

        service.load_texture(texture);
        let texture_bind_group = if let Some(bg) =
            service.get_bind_group_for_texture(texture, &service.bind_group_layouts().diffuse)
        {
            bg
        } else {
            return Err(EngineError::AssetLoadError(format!(
                "Failed to create menu diffuse texture bind group: {}",
                texture
            )));
        };

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
                texture_bind_group,
                root,
                is_visible: false,
                pipeline,
            })
        } else {
            Err(EngineError::AssetLoadError(format!(
                "Failed to build pipeline {}",
                pipeline_name
            )))
        }
    }

    pub fn builder(
        service: Arc<AssetService>,
        surface_config: &wgpu::SurfaceConfiguration,
        screen_w: u32,
        screen_h: u32,
    ) -> MenuBuilder {
        MenuBuilder::new(service, surface_config, screen_w, screen_h)
    }
    pub fn with_elements(
        service: &Arc<AssetService>,
        surface_config: &wgpu::SurfaceConfiguration,
        texture: &str,
        screen_w: u32,
        screen_h: u32,
        elements: Vec<MenuElement>,
        container: UiContainer,
    ) -> Result<Self, EngineError> {
        let mut menu = Self::new(
            service,
            surface_config,
            texture,
            screen_w,
            screen_h,
            container,
        )?;
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
                PhysicalKey::Code(KeyCode::KeyQ) => {
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
        rpass: &mut wgpu::RenderPass<'a>,
        d2: &mut Renderer2d,
        txt: &mut GlyphonTextRenderer,
        queue: &wgpu::Queue,
    ) {
        if !self.is_visible {
            return;
        }

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, self.bind_group.as_ref(), &[]);
        rpass.set_bind_group(1, self.texture_bind_group.as_ref(), &[]);

        self.root.draw(d2, txt);
        d2.flush(queue, rpass);
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
