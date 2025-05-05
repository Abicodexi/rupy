use cgmath::{InnerSpace, Matrix, Matrix4, Point3, SquareMatrix, Vector3};

use crate::renderer::model::AABB;

#[derive(Copy, Clone, Debug)]
pub struct Frustum {
    pub planes: [Plane; 6],
}

#[derive(Copy, Clone, Debug)]
pub struct Plane {
    pub normal: Vector3<f32>,
    pub d: f32,
}

impl Plane {
    pub fn from_components(a: f32, b: f32, c: f32, d: f32) -> Self {
        let normal = Vector3::new(a, b, c);
        let length = normal.magnitude();
        Self {
            normal: normal / length,
            d: d / length,
        }
    }

    pub fn distance(&self, point: Point3<f32>) -> f32 {
        self.normal.dot(Vector3::new(point.x, point.y, point.z)) + self.d
    }
}

impl Frustum {
    pub fn new() -> Self {
        Frustum::from_matrix(Matrix4::identity())
    }
    pub fn from_matrix(m: Matrix4<f32>) -> Self {
        let m = m.transpose(); // cgmath uses column-major

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

    pub fn contains_point(&self, point: Point3<f32>) -> bool {
        self.planes.iter().all(|plane| plane.distance(point) >= 0.0)
    }

    pub fn contains_sphere(&self, center: Point3<f32>, radius: f32) -> bool {
        self.planes
            .iter()
            .all(|plane| plane.distance(center) >= -radius)
    }
}
