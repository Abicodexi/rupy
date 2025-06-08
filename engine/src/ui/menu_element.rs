use crate::{menu_button::MenuButton, GlyphonTextRenderer, Renderer2d};

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
}
