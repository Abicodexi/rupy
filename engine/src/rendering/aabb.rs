use super::Vertex;
use glam::Vec3;

#[derive(Copy, Clone, Debug)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl Default for AABB {
    fn default() -> Self {
        Self {
            min: Vec3::ZERO,
            max: Vec3::ZERO,
        }
    }
}

impl AABB {
    pub fn from_vertices(vertices: &[Vertex]) -> AABB {
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);

        for v in vertices {
            let pos = Vec3::from_slice(&v.position);
            min = min.min(pos);
            max = max.max(pos);
        }

        Self { min, max }
    }

    pub fn get_normal_positive_vertex(&self, normal: Vec3) -> Vec3 {
        Vec3::new(
            if normal.x >= 0.0 {
                self.max.x
            } else {
                self.min.x
            },
            if normal.y >= 0.0 {
                self.max.y
            } else {
                self.min.y
            },
            if normal.z >= 0.0 {
                self.max.z
            } else {
                self.min.z
            },
        )
    }
}
