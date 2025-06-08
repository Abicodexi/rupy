use crate::{menu_element::MenuElement, GlyphonTextRenderer, Renderer2d};
pub struct MenuContainer {
    elements: Vec<MenuElement>,
    x: f32,
    y: f32,
    color: [f32; 4],
    uv: [f32; 4],
    padding: f32,
    layout_width: f32,
    layout_height: f32,
}

impl MenuContainer {
    pub fn new(position: (f32, f32), color: [f32; 4], uv: [f32; 4], padding: f32) -> Self {
        Self {
            elements: Vec::new(),
            x: position.0,
            y: position.1,
            color,
            uv,
            padding,
            layout_width: 0.0,
            layout_height: 0.0,
        }
    }

    pub fn push_element(&mut self, element: MenuElement) {
        self.elements.push(element);
    }

    pub fn layout(&mut self) {
        let mut cur_y = self.y + 10.0;
        let mut max_width = 0.0;

        for elem in &mut self.elements {
            let (elem_w, elem_h) = elem.size();

            elem.set_position(self.x + 10.0, cur_y);

            cur_y += elem_h + self.padding;
            if elem_w > max_width {
                max_width = elem_w;
            }
        }

        let total_height = self.elements.iter().map(|e| e.size().1).sum::<f32>()
            + (self.elements.len().saturating_sub(1) as f32) * self.padding
            + 20.0; // outer padding
        self.layout_height = total_height;

        self.layout_width = max_width + 20.0;
    }

    pub fn draw(&self, renderer: &mut Renderer2d, text_renderer: &mut GlyphonTextRenderer) {
        renderer.draw_filled_rect(
            self.x,
            self.y,
            self.layout_width,
            self.layout_height,
            self.color,
            self.uv,
        );
        for elem in &self.elements {
            elem.draw(renderer, text_renderer);
        }
    }

    pub fn elements(&self) -> &Vec<MenuElement> {
        &self.elements
    }
    pub fn elements_mut(&mut self) -> &mut Vec<MenuElement> {
        &mut self.elements
    }
}
