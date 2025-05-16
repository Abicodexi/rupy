pub struct TextRegion {
    pub text: String,
    pub pos: [f32; 2],
    pub color: glyphon::Color,
    pub bounds: Option<glyphon::TextBounds>,
}

impl TextRegion {
    pub fn new(text: impl Into<String>, pos: [f32; 2], color: glyphon::Color) -> Self {
        Self {
            text: text.into(),
            pos,
            color,
            bounds: None,
        }
    }
}
