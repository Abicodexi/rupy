use glam::{Mat4, Vec3};

use crate::AABB;

#[derive(Copy, Clone, Debug)]
pub struct Plane {
    pub normal: Vec3,
    d: f32,
}

impl Plane {
    pub fn from_components(a: f32, b: f32, c: f32, d: f32) -> Self {
        let normal = Vec3::new(a, b, c);
        let length = normal.length();
        Self {
            normal: normal / length,
            d: d / length,
        }
    }

    pub fn distance(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.d
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Frustum {
    pub planes: [Plane; 6],
}

impl Frustum {
    pub fn new() -> Self {
        Frustum::from_matrix(Mat4::IDENTITY)
    }

    pub fn from_matrix(m: Mat4) -> Self {
        let m = m.to_cols_array_2d();
        let row = |i| glam::Vec4::new(m[0][i], m[1][i], m[2][i], m[3][i]);
        let r0 = row(0);
        let r1 = row(1);
        let r2 = row(2);
        let r3 = row(3);

        Self {
            planes: [
                Plane::from_components(r3.x + r0.x, r3.y + r0.y, r3.z + r0.z, r3.w + r0.w), // left
                Plane::from_components(r3.x - r0.x, r3.y - r0.y, r3.z - r0.z, r3.w - r0.w), // right
                Plane::from_components(r3.x + r1.x, r3.y + r1.y, r3.z + r1.z, r3.w + r1.w), // bottom
                Plane::from_components(r3.x - r1.x, r3.y - r1.y, r3.z - r1.z, r3.w - r1.w), // top
                Plane::from_components(r3.x + r2.x, r3.y + r2.y, r3.z + r2.z, r3.w + r2.w), // near
                Plane::from_components(r3.x - r2.x, r3.y - r2.y, r3.z - r2.z, r3.w - r2.w), // far
            ],
        }
    }

    pub fn contains_point(&self, point: Vec3) -> bool {
        self.planes.iter().all(|plane| plane.distance(point) >= 0.0)
    }

    pub fn contains_sphere(&self, center: Vec3, radius: f32) -> bool {
        self.planes
            .iter()
            .all(|plane| plane.distance(center) >= -radius)
    }
    pub fn frustum_cull_aabb(&self, aabb: &AABB, model_matrix: &Mat4) -> bool {
        let corners = [
            Vec3::new(aabb.min.x, aabb.min.y, aabb.min.z),
            Vec3::new(aabb.min.x, aabb.min.y, aabb.max.z),
            Vec3::new(aabb.min.x, aabb.max.y, aabb.min.z),
            Vec3::new(aabb.min.x, aabb.max.y, aabb.max.z),
            Vec3::new(aabb.max.x, aabb.min.y, aabb.min.z),
            Vec3::new(aabb.max.x, aabb.min.y, aabb.max.z),
            Vec3::new(aabb.max.x, aabb.max.y, aabb.min.z),
            Vec3::new(aabb.max.x, aabb.max.y, aabb.max.z),
        ];

        for plane in &self.planes {
            if corners
                .iter()
                .all(|corner| plane.distance(model_matrix.transform_point3(*corner)) < 0.0)
            {
                return false;
            }
        }
        true
    }
}
