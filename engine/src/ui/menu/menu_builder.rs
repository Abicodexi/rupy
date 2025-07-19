use crate::{
    container::UiContainer,
    menu::{menu_button::MenuButton, menu_element::MenuElement, Menu},
    AssetLoader, AssetService, BindGroup, Dispatch, EngineError, Texture, UiElements,
};

pub struct MenuBuilder {
    surface_config: wgpu::SurfaceConfiguration,
    screen_w: u32,
    screen_h: u32,
    texture: Option<String>,
    container: UiContainer,
    x: f32,
    y: f32,
    padding: f32,

    elements: Vec<MenuElement>,
}

impl MenuBuilder {
    pub fn new(surface_config: &wgpu::SurfaceConfiguration, screen_w: u32, screen_h: u32) -> Self {
        Self {
            surface_config: surface_config.clone(),
            screen_w,
            screen_h,
            container: UiContainer::default(),
            x: 0.0,
            y: 0.0,
            texture: None,
            padding: 0.0,
            elements: Vec::new(),
        }
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
    pub fn with_texture(mut self, texture: &str) -> Self {
        self.texture = Some(texture.to_string());
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
        texture: Option<&str>,
    ) -> Self {
        let id = self.elements.len();
        let btn = MenuButton::new(
            id as u32,
            label,
            action,
            (0.0, 0.0),
            size,
            texture,
            id as i32,
            color,
            uv,
            highlight_color,
        );
        self.elements.push(MenuElement::Button(btn));
        self
    }

    pub fn build(self, service: &AssetService) -> Result<Menu, EngineError> {
        let mut menu = Menu::new(
            service,
            &self.surface_config,
            self.screen_w,
            self.screen_h,
            self.container,
        )?;
        let mut images: Vec<image::DynamicImage> = Vec::new();
        for elem in self.elements {
            if let Some(tex) = elem.texture() {
                let path = AssetLoader::resolve("textures").join(tex);
                let img_rgba = AssetLoader::image(path)?;
                images.push(img_rgba);
            }
            menu.add_element(elem);
        }

        let array_tex = Texture::from_image_array(
            service.device(),
            service.queue(),
            &images,
            wgpu::TextureFormat::Bgra8UnormSrgb,
            "ui_texture_array",
        );
        let array_tex_bg =
            BindGroup::sprite_2d_array(service.device(), service.bind_group_layouts(), &array_tex);

        menu.set_texture_bind_group(array_tex_bg);
        Ok(menu)
    }
}
