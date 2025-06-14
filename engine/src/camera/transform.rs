use glam::{Mat4, Vec3};

use crate::camera::Projection;

#[derive(Debug)]
pub struct CameraTransform {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fovy: f32,
    pub aspect: f32,
    pub znear: f32,
    pub zfar: f32,

    view: Mat4,
    proj: Mat4,
    view_proj: Mat4,
    dirty: bool,
}

impl CameraTransform {
    pub fn new(aspect: f32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            eye: Vec3::ZERO,
            target: Vec3::Z,
            up: Vec3::Y,
            fovy,
            aspect,
            znear,
            zfar,
            view: Mat4::IDENTITY,
            proj: Mat4::IDENTITY,
            view_proj: Mat4::IDENTITY,
            dirty: true,
        }
    }
    pub fn mark_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    pub fn update_matrices(
        &mut self,
        fov_y_radians: f32,
        aspect_ratio: f32,
        z_near: f32,
        z_far: f32,
        screen_w: f32,
        screen_h: f32,
        projection: &Projection,
    ) {
        self.view = Mat4::look_at_rh(self.eye, self.target, self.up);
        self.proj = Projection::matrix(
            projection,
            fov_y_radians,
            aspect_ratio,
            z_near,
            z_far,
            screen_w,
            screen_h,
        );
        self.view_proj = self.proj * self.view;
    }
    pub fn set_eye(&mut self, eye: Vec3) {
        if self.eye != eye {
            self.eye = eye;
            self.mark_dirty(true);
        }
    }

    pub fn set_target(&mut self, target: Vec3) {
        if self.target != target {
            self.target = target;
            self.mark_dirty(true);
        }
    }

    pub fn set_up(&mut self, up: Vec3) {
        if self.up != up {
            self.up = up;
            self.mark_dirty(true);
        }
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        if (self.aspect - aspect).abs() > f32::EPSILON {
            self.aspect = aspect;
            self.mark_dirty(true);
        }
    }

    pub fn set_fovy(&mut self, fovy: f32) {
        if (self.fovy - fovy).abs() > f32::EPSILON {
            self.fovy = fovy;
            self.mark_dirty(true);
        }
    }

    pub fn set_znear(&mut self, znear: f32) {
        if (self.znear - znear).abs() > f32::EPSILON {
            self.znear = znear;
            self.mark_dirty(true);
        }
    }

    pub fn set_zfar(&mut self, zfar: f32) {
        if (self.zfar - zfar).abs() > f32::EPSILON {
            self.zfar = zfar;
            self.mark_dirty(true);
        }
    }
    pub fn view_proj(&self) -> Mat4 {
        self.view_proj
    }
    pub fn view(&self) -> Mat4 {
        self.view
    }
    pub fn inv_view(&self) -> Mat4 {
        self.view.inverse()
    }

    pub fn inv_proj(&self) -> Mat4 {
        self.proj.inverse()
    }
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}
