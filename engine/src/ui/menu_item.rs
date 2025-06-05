#[derive(Clone, Debug)]
pub enum MenuAction {
    Play,
    Options,
    Quit,
}
pub struct MenuItem {
    pub label: String,
    pub action: MenuAction,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub clicked: Option<std::time::Instant>,
    pub on_click: Box<dyn Fn()>,
}

impl MenuItem {
    pub fn on_click(&mut self) {
        self.set_clicked();
        (self.on_click)()
    }
    pub fn click_elapsed(&self) -> u128 {
        if let Some(click) = self.clicked {
            click.elapsed().as_millis()
        } else {
            0
        }
    }
    fn set_clicked(&mut self) {
        self.clicked = Some(std::time::Instant::now());
    }
}
