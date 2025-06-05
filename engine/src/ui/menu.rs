use crate::{
    menu_item::{MenuAction, MenuItem},
    RenderText, Renderer2d,
};

pub struct Menu {
    pub items: Vec<MenuItem>,
    pub padding: f32,
    pub selected_idx: Option<usize>,
    pub rect_width: f32,
    pub rect_height: f32,
    is_visible: bool,
}

impl Menu {
    pub fn new(
        entries: Vec<(&str, MenuAction, Box<dyn Fn()>)>,
        x: f32,
        y: f32,
        rect_width: f32,
        rect_height: f32,
        padding: f32,
    ) -> Self {
        let mut items = Vec::with_capacity(entries.len());
        let mut cur_y = y;

        for (label, action, on_click) in entries {
            items.push(MenuItem {
                label: label.to_string(),
                action,
                x,
                y: cur_y,
                w: rect_width,
                h: rect_height,
                clicked: None,
                on_click,
            });
            cur_y += rect_height + padding;
        }

        Menu {
            items,
            padding,
            rect_width,
            rect_height,
            is_visible: false,
            selected_idx: None,
        }
    }

    pub fn set_rect_size(&mut self, width: f32, height: f32) {
        self.rect_width = width;
        self.rect_height = height;
        if let Some(first) = self.items.first() {
            let x = first.x;
            let mut y = first.y;
            for item in &mut self.items {
                item.x = x;
                item.y = y;
                item.w = width;
                item.h = height;
                y += height + self.padding;
            }
        }
    }

    pub fn update(
        &mut self,
        mouse_x: f32,
        mouse_y: f32,
        mouse_pressed: bool,
    ) -> Option<MenuAction> {
        self.selected_idx = None;
        for (i, item) in self.items.iter().enumerate() {
            let inside_x = mouse_x >= item.x && mouse_x <= item.x + item.w;
            let inside_y = mouse_y >= item.y && mouse_y <= item.y + item.h;
            if inside_x && inside_y {
                self.selected_idx = Some(i);
                if mouse_pressed {
                    return Some(item.action.clone());
                }
                break;
            }
        }
        None
    }
    pub fn visible(&self) -> bool {
        self.is_visible
    }
    pub fn show(&mut self) {
        self.is_visible = true;
    }
    pub fn hide(&mut self) {
        self.is_visible = false;
    }
    pub fn draw_ui(&self, renderer2d: &mut Renderer2d, render_text: &mut RenderText) {
        if let Some(first) = self.items.first() {
            let panel_x = first.x - 10.0;
            let panel_y = first.y - 10.0;
            let panel_w = self.rect_width + 20.0;
            let panel_h = self.items.len() as f32 * self.rect_height
                + (self.items.len() - 1) as f32 * self.padding
                + 20.0;
            renderer2d.draw_filled_rect(panel_x, panel_y, panel_w, panel_h, [0.0, 0.0, 0.0, 0.75]);
        }

        for (i, item) in self.items.iter().enumerate() {
            let color = if Some(i) == self.selected_idx {
                [0.2, 0.5, 0.9, 0.8]
            } else {
                [0.1, 0.1, 0.1, 0.8]
            };
            renderer2d.draw_filled_rect(item.x, item.y, item.w, item.h, color);
        }

        for item in &self.items {
            render_text.queue_text(
                &item.label,
                item.x,
                item.y,
                glyphon::Color::rgb(255, 255, 255),
            );
        }
    }
}
