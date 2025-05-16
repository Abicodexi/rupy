use std::time::Instant;

use crate::TextRegion;

#[derive(Debug)]
pub struct Time {
    last_update: Instant,
    pub delta_time: f64,
    pub fps: f64,
    pub elapsed: f64,
    frame_count: u32,
}

impl Time {
    pub fn new() -> Self {
        Self {
            last_update: Instant::now(),
            delta_time: 0.0,
            fps: 0.0,
            elapsed: 0.0,
            frame_count: 0,
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let duration = now - self.last_update;
        self.last_update = now;

        self.delta_time = duration.as_secs_f64();
        self.elapsed += self.delta_time;
        self.frame_count += 1;

        if self.delta_time > 0.0 {
            self.fps = 1.0 / self.delta_time;
        }
    }

    pub fn text_region(&self, position: [f32; 2]) -> TextRegion {
        let text_area = TextRegion::new(
            format!(
                "FPS: {:.1} Frame time: {:.3}ms",
                self.fps,
                self.delta_time * 1000.0,
            ),
            position,
            glyphon::Color::rgb(1, 1, 1),
        );
        text_area
    }
}
