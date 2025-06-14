use glam::Vec3;

use crate::GRAVITY;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Medium {
    Air,
    Water,
    Ground,
    Vacuum,
}
#[derive(Debug, Clone, Copy)]
pub struct MediumProperties {
    pub gravity: Vec3,
    pub drag: f32,
}

impl Medium {
    pub fn properties(self) -> MediumProperties {
        match self {
            Medium::Air => MediumProperties {
                gravity: Vec3::new(0.0, GRAVITY, 0.0),
                drag: 0.1,
            },
            Medium::Water => MediumProperties {
                gravity: Vec3::new(0.0, GRAVITY + 7.81, 0.0),
                drag: 0.2,
            },
            Medium::Ground => MediumProperties {
                gravity: Vec3::new(0.0, GRAVITY, 0.0),
                drag: 0.01,
            },
            Medium::Vacuum => MediumProperties {
                gravity: Vec3::ZERO,
                drag: 0.9,
            },
        }
    }
    pub fn is_solid(self) -> bool {
        matches!(self, Medium::Ground)
    }

    pub fn is_fluid(self) -> bool {
        matches!(self, Medium::Air | Medium::Water)
    }
}
