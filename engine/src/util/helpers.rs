pub enum ScreenCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

impl ScreenCorner {
    pub fn pos(&self, surface_width: u32, surface_height: u32, margin: f32) -> [f32; 2] {
        match self {
            ScreenCorner::TopLeft => [margin, margin],
            ScreenCorner::TopRight => [surface_width as f32 - margin, margin],
            ScreenCorner::BottomLeft => [margin, surface_height as f32 - margin],
            ScreenCorner::BottomRight => [
                surface_width as f32 - margin,
                surface_height as f32 - margin,
            ],
            ScreenCorner::Center => [surface_width as f32 * 0.5, surface_height as f32 * 0.5],
        }
    }
}
