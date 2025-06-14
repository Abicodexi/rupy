use std::sync::Arc;

use crate::{
    container::UiContainer, menu::Menu, menu_button::MenuButton, menu_element::MenuElement,
    AssetService, Dispatch, EngineError,
};

pub struct MenuBuilder {
    service: Arc<AssetService>,
    surface_config: wgpu::SurfaceConfiguration,
    screen_w: u32,
    screen_h: u32,
    container: UiContainer,

    texture: Option<String>,

    x: f32,
    y: f32,
    padding: f32,

    elements: Vec<MenuElement>,
}

impl MenuBuilder {
    pub fn new(
        service: Arc<AssetService>,
        surface_config: &wgpu::SurfaceConfiguration,
        screen_w: u32,
        screen_h: u32,
    ) -> Self {
        Self {
            service,
            surface_config: surface_config.clone(),
            screen_w,
            screen_h,
            container: UiContainer::default(),
            texture: None,
            x: 0.0,
            y: 0.0,
            padding: 0.0,
            elements: Vec::new(),
        }
    }

    pub fn with_texture(mut self, path: &str) -> Self {
        self.texture = Some(path.to_string());
        self
    }

    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self.container.set_position((self.x, self.y));
        self
    }

    pub fn with_padding(mut self, p: f32) -> Self {
        self.padding = p;
        self
    }
    pub fn with_container_style(mut self, color: [f32; 4], uv: [f32; 4]) -> Self {
        self.container.set_color(color);
        self.container.set_uv(uv);
        self
    }
    pub fn with_button(
        mut self,
        label: &str,
        action: Dispatch,
        size: (f32, f32),
        color: [f32; 4],
        uv: [f32; 4],
        highlight_color: [f32; 4],
    ) -> Self {
        let id = self
            .elements
            .iter()
            .filter(|el| match el {
                MenuElement::Button(..) => true,
            })
            .count();
        let btn = MenuButton::new(
            id as u32,
            label,
            action,
            (0.0, 0.0),
            size,
            color,
            uv,
            highlight_color,
        );
        self.elements.push(MenuElement::Button(btn));
        self
    }

    pub fn build(self) -> Result<Menu, EngineError> {
        let tex = self
            .texture
            .as_deref()
            .ok_or_else(|| EngineError::AssetLoadError("no texture set".into()))?;

        let mut menu = Menu::new(
            &self.service,
            &self.surface_config,
            tex,
            self.screen_w,
            self.screen_h,
            self.container,
        )?;
        for elem in self.elements {
            menu.add_element(elem);
        }
        Ok(menu)
    }
}
