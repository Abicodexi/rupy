use crate::{GlyphonTextRenderer, Renderer2d};

pub struct MenuButton {
    pub label: String,
    x: f32,
    y: f32,
    color: [f32; 4],
    uv: [f32; 4],
    highlight_color: [f32; 4],
    width: f32,
    height: f32,
    callback: Box<dyn Fn()>,
    disabled: bool,
    highlight: bool,
}

impl MenuButton {
    pub fn new(
        label: &str,
        position: (f32, f32),
        size: (f32, f32),
        color: [f32; 4],
        uv: [f32; 4],
        highlight_color: [f32; 4],
        callback: Box<dyn Fn()>,
    ) -> Self {
        Self {
            label: label.to_string(),
            x: position.0,
            y: position.1,
            width: size.0,
            height: size.1,
            color,
            uv,
            highlight_color,
            callback,
            highlight: false,
            disabled: false,
        }
    }
    pub fn width(&self) -> f32 {
        self.width
    }
    pub fn height(&self) -> f32 {
        self.height
    }
    pub fn set_position(&mut self, position: (f32, f32)) {
        let (pos_x, pos_y) = position;
        self.x = pos_x;
        self.y = pos_y;
    }
    pub fn update(&mut self, mouse_position: Option<(f32, f32)>, clicked: (bool, bool)) {
        if let Some((mouse_x, mouse_y)) = mouse_position {
            let contains = self.contains(mouse_x, mouse_y);
            if contains {
                let (m1_clicked, _m2_clicked) = clicked;
                if m1_clicked {
                    self.on_click();
                }
            }
            self.highlight = contains;
        }
    }
    pub fn draw(&self, renderer: &mut Renderer2d, text_renderer: &mut GlyphonTextRenderer) {
        let color = if !self.highlight {
            self.color
        } else {
            self.highlight_color
        };

        renderer.draw_filled_rect(self.x, self.y, self.width, self.height, color, self.uv);
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
    pub fn set_callback(&mut self, callback: Box<dyn Fn()>) {
        self.callback = callback;
    }

    pub fn on_click(&self) {
        (self.callback)()
    }
}
