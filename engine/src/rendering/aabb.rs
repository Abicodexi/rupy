use super::Vertex;

#[derive(Copy, Clone, Debug)]
pub struct AABB {
    pub min: cgmath::Point3<f32>,
    pub max: cgmath::Point3<f32>,
}
impl Default for AABB {
    fn default() -> Self {
        Self {
            min: cgmath::Point3::new(0.0, 0.0, 0.0),
            max: cgmath::Point3::new(0.0, 0.0, 0.0),
        }
    }
}
impl AABB {
    pub fn from_vertices(vertices: &[Vertex]) -> AABB {
        let mut min = cgmath::Vector3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = cgmath::Vector3::new(f32::MIN, f32::MIN, f32::MIN);
        for v in vertices {
            let pos = cgmath::Vector3::new(v.position[0], v.position[1], v.position[2]);
            min = min.zip(pos, f32::min);
            max = max.zip(pos, f32::max);
        }
        Self {
            min: <cgmath::Point3<f32> as cgmath::EuclideanSpace>::from_vec(min),
            max: <cgmath::Point3<f32> as cgmath::EuclideanSpace>::from_vec(max),
        }
    }
    pub fn get_normal_positive_vertex(&self, normal: cgmath::Vector3<f32>) -> cgmath::Point3<f32> {
        cgmath::Point3::new(
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
