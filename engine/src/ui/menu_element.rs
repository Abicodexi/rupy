use crate::{
    menu_button::MenuButton, GlyphonTextRenderer, Renderer2d, UiElement, UiElements, UiEvent,
};
#[derive(Debug, Clone)]
pub enum MenuElement {
    Button(MenuButton),
}
impl MenuElement {
    pub fn draw(&self, renderer: &mut Renderer2d, text_renderer: &mut GlyphonTextRenderer) {
        match self {
            MenuElement::Button(btn) => {
                btn.draw(renderer, text_renderer);
            }
        }
    }
    pub fn size(&self) -> (f32, f32) {
        match self {
            MenuElement::Button(menu_button) => (menu_button.width(), menu_button.height()),
        }
    }
    pub fn set_position(&mut self, pos_x: f32, pos_y: f32) {
        match self {
            MenuElement::Button(menu_button) => menu_button.set_position((pos_x, pos_y)),
        }
    }
    pub fn update(
        &mut self,
        mouse_position: Option<(f32, f32)>,
        clicked: (bool, bool),
    ) -> Option<UiEvent> {
        match self {
            MenuElement::Button(menu_button) => {
                if let Some(ev) = menu_button.update_event(mouse_position, clicked) {
                    return Some(ev);
                }
            }
        }
        None
    }
}

impl UiElements for MenuElement {
    fn id(&self) -> u32 {
        match self {
            MenuElement::Button(menu_button) => menu_button.id(),
        }
    }

    fn label(&self) -> &str {
        match self {
            MenuElement::Button(menu_button) => menu_button.label(),
        }
    }

    fn get_element(self) -> UiElement {
        UiElement::Menu(self)
    }

    fn size(&self) -> (f32, f32) {
        match self {
            MenuElement::Button(menu_button) => menu_button.size(),
        }
    }

    fn position(&self) -> (f32, f32) {
        match self {
            MenuElement::Button(menu_button) => menu_button.position(),
        }
    }

    fn set_position(&mut self, position: (f32, f32)) {
        match self {
            MenuElement::Button(menu_button) => menu_button.set_position(position),
        }
    }

    fn draw(&self, renderer: &mut Renderer2d, text_renderer: &mut GlyphonTextRenderer) {
        match self {
            MenuElement::Button(menu_button) => menu_button.draw(renderer, text_renderer),
        }
    }

    fn texture(&self) -> &Option<String> {
        match self {
            MenuElement::Button(menu_button) => menu_button.texture(),
        }
    }
}
