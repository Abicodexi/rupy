use crate::{
    camera::{Camera, OrthoUniform},
    create_sprite2d_pipeline,
    menu_container::MenuContainer,
    menu_element::MenuElement,
    CacheKey, CacheStorage, EngineError, GlyphonTextRenderer, ModelManager, PipelineManager,
    RenderBindGroupLayouts, RenderPipelineManager, Renderer2d, WgpuBuffer,
};
use winit::{
    event::WindowEvent,
    keyboard::{KeyCode, PhysicalKey},
};

pub struct Menu {
    pub ortho_buffer: WgpuBuffer,
    pub ortho_bind_group: wgpu::BindGroup,
    pub texture_bind_group: wgpu::BindGroup,
    pub pipeline_key: CacheKey,

    root: MenuContainer,
    is_visible: bool,
}

impl Menu {
    pub fn new(
        device: &wgpu::Device,
        texture_bind_group: wgpu::BindGroup,
        screen_w: u32,
        screen_h: u32,
        x: f32,
        y: f32,
        container_color: [f32; 4],
        container_uv: [f32; 4],
        padding: f32,
    ) -> Self {
        let ortho_uniform = OrthoUniform::new(screen_w as f32, screen_h as f32);
        let ortho_buffer = WgpuBuffer::from_data(
            device,
            bytemuck::bytes_of(&ortho_uniform),
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            Some("Menu Ortho Buffer"),
        );
        let ortho_bind_group_layout = RenderBindGroupLayouts::ortho_uniform();
        let ortho_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Menu OrthoBindGroup"),
            layout: &ortho_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: ortho_buffer.get().as_entire_binding(),
            }],
        });

        let root = MenuContainer::new((x, y), container_color, container_uv, padding);
        let pipeline_key = CacheKey::from("sprite2d");

        Menu {
            ortho_buffer,
            ortho_bind_group,
            texture_bind_group,
            root,
            is_visible: false,
            pipeline_key,
        }
    }
    pub fn with_elements(
        device: &wgpu::Device,
        texture_bind_group: wgpu::BindGroup,
        screen_w: u32,
        screen_h: u32,
        x: f32,
        y: f32,
        color: [f32; 4],
        uv: [f32; 4],
        padding: f32,
        elements: Vec<MenuElement>,
    ) -> Self {
        let mut menu = Self::new(
            device,
            texture_bind_group,
            screen_w,
            screen_h,
            x,
            y,
            color,
            uv,
            padding,
        );
        for element in elements {
            menu.add_element(element);
        }
        menu.root.layout();
        menu
    }
    pub fn build_pipeline(
        &mut self,
        device: &wgpu::Device,
        cfg: &wgpu::SurfaceConfiguration,
        pipeline_manager: &mut RenderPipelineManager,
    ) -> Result<(), EngineError> {
        if !pipeline_manager.contains(&self.pipeline_key) {
            let sprite2d_pipeline = create_sprite2d_pipeline(device, cfg.format)?;
            pipeline_manager.insert(self.pipeline_key, sprite2d_pipeline.into());
        }
        Ok(())
    }
    pub fn resize(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        screen_w: f32,
        screen_h: f32,
    ) {
        self.ortho_buffer.write_data(
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
                    self.is_visible
                }

                _ => false,
            },
            _ => false,
        }
    }
    pub fn render<'a>(
        &'a mut self,
        rpass: &mut wgpu::RenderPass<'a>,
        d2: &mut Renderer2d,
        txt: &mut GlyphonTextRenderer,
        queue: &wgpu::Queue,
        pipelines: &PipelineManager,
    ) {
        if !self.is_visible {
            return;
        }
        if let Some(pipeline) = pipelines.render.get(&self.pipeline_key) {
            rpass.set_pipeline(pipeline);
            rpass.set_bind_group(0, &self.ortho_bind_group, &[]);
            rpass.set_bind_group(1, &self.texture_bind_group, &[]);
        }

        self.root.draw(d2, txt);
        d2.flush(queue, rpass);
    }
    pub fn add_element(&mut self, element: MenuElement) {
        self.root.push_element(element);
    }
    pub fn update(&mut self, mouse_position: Option<(f32, f32)>, clicked: (bool, bool)) {
        for elem in self.root.elements_mut() {
            match elem {
                MenuElement::Button(menu_button) => {
                    menu_button.update(mouse_position, clicked);
                }
                _ => (),
            }
        }
    }
    pub fn on_click(&mut self, mouse_position: Option<(f32, f32)>) {
        let Some((mouse_x, mouse_y)) = mouse_position else {
            return;
        };
        for elem in self.root.elements_mut() {
            match elem {
                MenuElement::Button(menu_button) => {
                    if menu_button.contains(mouse_x, mouse_y) {
                        menu_button.on_click();
                        break;
                    }
                }
                _ => {}
            }
        }
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
