use crate::{Dispatch, GlyphonTextRenderer, Renderer2d, UiElements, UiEvent};
#[derive(Debug, Clone)]
pub struct MenuButton {
    id: u32,
    label: String,
    action: Dispatch,
    x: f32,
    y: f32,
    texture: Option<String>,
    texture_index: i32,
    color: [f32; 4],
    uv: [f32; 4],
    highlight_color: [f32; 4],
    width: f32,
    height: f32,
    disabled: bool,
    highlight: bool,
}
impl UiElements for MenuButton {
    fn id(&self) -> u32 {
        self.id
    }

    fn label(&self) -> &str {
        &self.label
    }

    fn get_element(self) -> super::UiElement {
        super::UiElement::Menu(super::menu_element::MenuElement::Button(self))
    }

    fn size(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    fn position(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    fn set_position(&mut self, position: (f32, f32)) {
        self.x = position.0;
        self.y = position.1;
    }

    fn draw(&self, renderer: &mut Renderer2d, text_renderer: &mut GlyphonTextRenderer) {
        self.draw(renderer, text_renderer);
    }

    fn texture(&self) -> &Option<String> {
        &self.texture
    }
}
impl MenuButton {
    pub fn new(
        id: u32,
        label: &str,
        action: Dispatch,
        position: (f32, f32),
        size: (f32, f32),
        texture: Option<&str>,
        texture_index: i32,
        color: [f32; 4],
        uv: [f32; 4],
        highlight_color: [f32; 4],
    ) -> Self {
        Self {
            id: id,
            label: label.to_string(),
            action,
            x: position.0,
            y: position.1,
            width: size.0,
            height: size.1,
            color,
            texture: if let Some(tex) = texture {
                Some(tex.to_string())
            } else {
                None
            },
            texture_index,
            uv,
            highlight_color,
            disabled: false,
            highlight: false,
        }
    }

    pub fn update_event(
        &mut self,
        mouse_position: Option<(f32, f32)>,
        clicked: (bool, bool),
    ) -> Option<UiEvent> {
        if self.disabled {
            self.highlight = false;
            return None;
        }

        if let Some((mx, my)) = mouse_position {
            let over = self.contains(mx, my);
            self.highlight = over;

            if over && clicked.0 {
                return Some(UiEvent::ButtonClicked(self.id));
            }
        }

        None
    }
}

impl MenuButton {
    pub fn width(&self) -> f32 {
        self.width
    }
    pub fn height(&self) -> f32 {
        self.height
    }
    pub fn texture(&self) -> &Option<String> {
        &self.texture
    }
    pub fn set_position(&mut self, position: (f32, f32)) {
        let (pos_x, pos_y) = position;
        self.x = pos_x;
        self.y = pos_y;
    }

    pub fn draw(&self, renderer: &mut Renderer2d, text_renderer: &mut GlyphonTextRenderer) {
        let color = if !self.highlight {
            self.color
        } else {
            self.highlight_color
        };

        renderer.draw_filled_rect(
            self.x,
            self.y,
            self.width,
            self.height,
            color,
            self.uv,
            self.texture_index,
        );
        let center_x = self.x + (self.width - (self.label.len() as f32 * 10.0)) * 0.5;
        let center_y = self.y + (self.height) * 0.5;

        text_renderer.queue_text(
            &self.label,
            center_x,
            center_y,
            glyphon::Color::rgb(255, 255, 255),
        );
    }
    pub fn contains(&self, mouse_x: f32, mouse_y: f32) -> bool {
        mouse_x >= self.x
            && mouse_x <= self.x + self.width
            && mouse_y >= self.y
            && mouse_y <= self.y + self.height
    }
    pub fn set_hightlight(&mut self, hightlight: bool) {
        self.highlight = hightlight;
    }
    pub fn disable(&mut self) {
        self.disabled = true;
    }
    pub fn enable(&mut self) {
        self.disabled = false;
    }
    pub fn action(&self) -> Dispatch {
        self.action.clone()
    }
}
