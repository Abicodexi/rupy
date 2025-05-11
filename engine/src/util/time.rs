use std::time::Instant;

#[derive(Debug)]
pub struct Time {
    last_update: Instant,
    pub delta_time: f32, // in seconds
    pub fps: f32,
    pub elapsed: f32, // total time since start, in seconds
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

        self.delta_time = duration.as_secs_f32();
        self.elapsed += self.delta_time;
        self.frame_count += 1;

        if self.delta_time > 0.0 {
            self.fps = 1.0 / self.delta_time;
        }
    }
}
