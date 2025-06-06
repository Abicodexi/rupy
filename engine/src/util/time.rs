use std::time::Instant;

use crate::TextRegion;

#[derive(Debug)]
pub struct Time {
    pub last_update: Instant,
    pub delta_time: f64,
    pub fps: f64,
    pub elapsed: f64,
    pub frame_count: u32,

    pub accumulator: f64,
    pub last_frame_time: Instant,
}

impl Time {
    pub const TIME_STEP: f64 = 1.0 / 60.0;
    pub const MAX_FRAME_TIME: f64 = 0.25;
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            last_update: now,
            delta_time: 0.0,
            fps: 0.0,
            elapsed: 0.0,
            frame_count: 0,
            accumulator: 0.0,
            last_frame_time: now,
        }
    }

    pub fn update(&mut self, max_frame_time: f64) -> f64 {
        let now = Instant::now();
        let mut delta = (now - self.last_frame_time).as_secs_f64();
        if delta > max_frame_time {
            delta = max_frame_time;
        }
        self.last_frame_time = now;

        self.delta_time = delta;
        self.elapsed += self.delta_time;
        self.frame_count += 1;

        if self.delta_time > 0.0 {
            self.fps = 1.0 / self.delta_time;
        }

        self.accumulator += self.delta_time;

        self.delta_time
    }

    pub fn consume_accumulator(&mut self, fixed_time_step: f64) -> bool {
        if self.accumulator >= fixed_time_step {
            self.accumulator -= fixed_time_step;
            true
        } else {
            false
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
