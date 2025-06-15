use crate::{menu_element::MenuElement, Dispatch, GlyphonTextRenderer, Renderer2d};

pub mod container;
pub mod menu;
pub mod menu_builder;
pub mod menu_button;
pub mod menu_element;

#[derive(Debug, Clone)]
pub enum UiEvent {
    ButtonClicked(u32),
    ButtonHovered(u32),
}
pub enum UiElement {
    Menu(MenuElement),
}
pub enum UiPartial {
    Button {
        id: u32,
        label: String,
        action: Dispatch,
        size: (f32, f32),
        color: [f32; 4],
        uv: [f32; 4],
        highlight_color: [f32; 4],
        texture_path: String,
    },
}
pub trait UiElements {
    fn id(&self) -> u32;
    fn label(&self) -> &str;
    fn size(&self) -> (f32, f32);
    fn position(&self) -> (f32, f32);
    fn texture(&self) -> &Option<String>;
    fn set_position(&mut self, position: (f32, f32));
    fn get_element(self) -> UiElement;
    fn draw(&self, renderer: &mut Renderer2d, text_renderer: &mut GlyphonTextRenderer);
}

impl UiElements for UiElement {
    fn id(&self) -> u32 {
        match self {
            UiElement::Menu(menu_element) => menu_element.id(),
        }
    }

    fn label(&self) -> &str {
        match self {
            UiElement::Menu(menu_element) => menu_element.label(),
        }
    }

    fn size(&self) -> (f32, f32) {
        match self {
            UiElement::Menu(menu_element) => menu_element.size(),
        }
    }

    fn position(&self) -> (f32, f32) {
        match self {
            UiElement::Menu(menu_element) => menu_element.position(),
        }
    }

    fn set_position(&mut self, position: (f32, f32)) {
        match self {
            UiElement::Menu(menu_element) => menu_element.set_position(position.0, position.1),
        }
    }

    fn get_element(self) -> UiElement {
        match self {
            UiElement::Menu(menu_element) => menu_element.get_element(),
        }
    }

    fn draw(&self, renderer: &mut Renderer2d, text_renderer: &mut GlyphonTextRenderer) {
        match self {
            UiElement::Menu(menu_element) => menu_element.draw(renderer, text_renderer),
        }
    }

    fn texture(&self) -> &Option<String> {
        match self {
            UiElement::Menu(menu_element) => menu_element.texture(),
        }
    }
}
