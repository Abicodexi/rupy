#[repr(C)]
#[derive(Copy, Clone)]
pub struct DebugUniform {
    pub mode: u32,
}

impl DebugUniform {
    pub fn next(&mut self) {
        self.mode = match self.mode {
            0 => 1,
            1 => 2,
            2 => 3,
            3 => 4,
            4 => 5,
            5 => 6,
            _ => 0,
        };
    }
}
