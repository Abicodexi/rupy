#[derive(Copy, Clone, Debug)]
pub struct Frustum {
    planes: [Plane; 6],
}

#[derive(Copy, Clone, Debug)]
pub struct Plane {
    normal: cgmath::Vector3<f32>,
    d: f32,
}

impl Plane {
    pub fn from_components(a: f32, b: f32, c: f32, d: f32) -> Self {
        let normal = cgmath::Vector3::new(a, b, c);
        let length = cgmath::InnerSpace::magnitude(normal);
        Self {
            normal: normal / length,
            d: d / length,
        }
    }

    pub fn distance(&self, point: cgmath::Point3<f32>) -> f32 {
        cgmath::InnerSpace::dot(self.normal, cgmath::Vector3::new(point.x, point.y, point.z))
            + self.d
    }
}

impl Frustum {
    pub fn new() -> Self {
        Frustum::from_matrix(<cgmath::Matrix4<f32> as cgmath::SquareMatrix>::identity())
    }
    pub fn from_matrix(m: cgmath::Matrix4<f32>) -> Self {
        let m = cgmath::Matrix::transpose(&m);

        Self {
            planes: [
                Plane::from_components(m.w.x + m.x.x, m.w.y + m.x.y, m.w.z + m.x.z, m.w.w + m.x.w), // left
                Plane::from_components(m.w.x - m.x.x, m.w.y - m.x.y, m.w.z - m.x.z, m.w.w - m.x.w), // right
                Plane::from_components(m.w.x + m.y.x, m.w.y + m.y.y, m.w.z + m.y.z, m.w.w + m.y.w), // bottom
                Plane::from_components(m.w.x - m.y.x, m.w.y - m.y.y, m.w.z - m.y.z, m.w.w - m.y.w), // top
                Plane::from_components(m.w.x + m.z.x, m.w.y + m.z.y, m.w.z + m.z.z, m.w.w + m.z.w), // near
                Plane::from_components(m.w.x - m.z.x, m.w.y - m.z.y, m.w.z - m.z.z, m.w.w - m.z.w), // far
            ],
        }
    }

    pub fn contains_point(&self, point: cgmath::Point3<f32>) -> bool {
        self.planes.iter().all(|plane| plane.distance(point) >= 0.0)
    }

    pub fn contains_sphere(&self, center: cgmath::Point3<f32>, radius: f32) -> bool {
        self.planes
            .iter()
            .all(|plane| plane.distance(center) >= -radius)
    }
}
