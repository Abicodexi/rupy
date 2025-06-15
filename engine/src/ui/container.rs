use crate::{GlyphonTextRenderer, Renderer2d, UiElement, UiElements};
pub struct UiContainer {
    elements: Vec<UiElement>,
    x: f32,
    y: f32,
    padding: f32,
    layout_width: f32,
    layout_height: f32,
}
impl Default for UiContainer {
    fn default() -> Self {
        UiContainer::new((0.0, 0.0), 0.0)
    }
}
impl UiContainer {
    pub fn new(position: (f32, f32), padding: f32) -> Self {
        Self {
            elements: Vec::new(),
            x: position.0,
            y: position.1,

            padding,
            layout_width: 0.0,
            layout_height: 0.0,
        }
    }

    pub fn push_element(&mut self, element: UiElement) {
        self.elements.push(element);
        self.layout();
    }

    pub fn layout(&mut self) {
        let mut cur_y = self.y + 10.0;
        let mut max_width = 0.0;

        for elem in &mut self.elements {
            let (elem_w, elem_h) = elem.size();

            elem.set_position((self.x + 10.0, cur_y));

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

    pub fn set_position(&mut self, position: (f32, f32)) {
        self.x = position.0;
        self.y = position.1;
    }

    pub fn elements(&self) -> &Vec<UiElement> {
        &self.elements
    }
    pub fn elements_mut(&mut self) -> &mut Vec<UiElement> {
        &mut self.elements
    }
    pub fn get_element(&self, id: u32) -> Option<&UiElement> {
        self.elements.get(id as usize)
    }
}
