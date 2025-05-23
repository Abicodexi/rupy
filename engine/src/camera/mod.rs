pub mod controller;

pub use controller::*;

pub mod frustum;
pub use frustum::*;

pub mod uniform;
pub use uniform::*;

pub mod model;
pub use model::*;

pub mod projection;
pub use projection::*;

use crate::{
    log_debug, log_warning, Entity, ModelManager, Position, RenderBindGroupLayouts, Renderable,
    Rotation, Scale, TextRegion, Velocity, Vertex, VertexInstance, WgpuBuffer, World, GROUND_Y,
};

use glam::{FloatExt, Mat4, Quat, Vec3};

#[derive(Debug)]
pub struct Camera {
    eye: Vec3,
    target: Vec3,
    up: Vec3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
    forward: Vec3,
    reach_distance: f32,
    model: CameraModel,
    bind_group: wgpu::BindGroup,
    uniform_buffer: WgpuBuffer,
    free_look: bool,
}

impl Camera {
    pub const UNIFORM_BUFFER_BINDING: crate::BindGroupBindingType = crate::BindGroupBindingType {
        binding: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<
                crate::camera::uniform::CameraUniform,
            >() as u64),
        },
    };
    pub fn new(device: &wgpu::Device, aspect: f32) -> Self {
        let model = CameraModel::new("goblin.obj", "v_normal.wgsl");
        let uniform_buffer = WgpuBuffer::from_data(
            device,
            &[CameraUniform::default()],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            Some("camera uniform buffer"),
        );
        let bind_group = crate::BindGroup::camera(device, &uniform_buffer);
        let z = Vec3::ZERO;
        let eye = z;
        let target = z;
        let forward = z;
        let up = Vec3::Y;
        let fovy = 89.0_f32.to_radians();
        let zfar = 100.0;
        let znear = 0.1;
        let reach_distance = 2.0;
        let free_look = false;

        Camera {
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar,
            reach_distance,
            forward,
            model,
            bind_group,
            uniform_buffer,
            free_look,
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.aspect = width / height;
    }
    pub fn forward(&self) -> Vec3 {
        self.forward
    }
    pub fn eye(&self) -> &Vec3 {
        &self.eye
    }
    pub fn zfar(&self) -> f32 {
        self.zfar
    }
    pub fn znear(&self) -> f32 {
        self.znear
    }
    pub fn look_at(&mut self, pos: Vec3) {
        self.target = pos;
    }
    pub fn target(&self) -> &Vec3 {
        &self.target
    }
    pub fn up(&self) -> &Vec3 {
        &self.up
    }
    pub fn fovy(&self) -> f32 {
        self.fovy
    }
    pub fn set_free_look(&mut self, val: bool) {
        self.free_look = val;
        log_debug!("Free look: {:?}", self.free_look);
    }
    pub fn free_look(&self) -> bool {
        self.free_look
    }
    pub fn entity(&self) -> Option<Entity> {
        self.model.entity()
    }
    pub fn buffer(&self) -> &crate::WgpuBuffer {
        &self.uniform_buffer
    }
    pub fn upload(&mut self, queue: &wgpu::Queue, device: &wgpu::Device) {
        self.uniform_buffer
            .write_data(queue, device, &[self.uniform()], None);
    }
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    fn load_model(
        &mut self,
        model_manager: &mut ModelManager,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        surface_configuration: &wgpu::SurfaceConfiguration,
    ) {
        self.model.load_model(
            model_manager,
            buffers,
            bind_group_layouts,
            surface_configuration,
        );
    }

    pub fn world_spawn(
        &mut self,
        world: &mut World,
        model_manager: &mut ModelManager,
        surface_configuration: &wgpu::SurfaceConfiguration,
    ) {
        if self.model.model_key().is_none() && !self.model.model().is_empty() {
            self.load_model(
                model_manager,
                &[Vertex::LAYOUT, VertexInstance::LAYOUT],
                vec![
                    RenderBindGroupLayouts::uniform().clone(),
                    RenderBindGroupLayouts::equirect_dst().clone(),
                    RenderBindGroupLayouts::material_storage().clone(),
                    RenderBindGroupLayouts::normal().clone(),
                ],
                surface_configuration,
            );
        }
        if let Some(model_key) = self.model.model_key() {
            let renderable: Renderable = model_key.into();

            let entity = if let Some(ent) = self.model.entity() {
                ent
            } else {
                let new_ent = world.spawn();
                self.model.set_entity(new_ent);
                new_ent
            };

            world.insert_scale(entity, Scale::one());
            world.insert_position(entity, Position::new(0.0, GROUND_Y + 1.0, 0.0));
            world.insert_renderable(entity, renderable);
            log_debug!("Spawned camera model: {}", self.model.model());
        } else {
            log_warning!("No camera model available");
            return;
        }
    }
    pub fn update(
        &mut self,
        world: &mut World,
        cam: &mut CameraControls,
        projection: &Projection,
        bossman: &Entity,
    ) {
        let Some(model_entity) = self.model.entity() else {
            return;
        };

        let player_pos = world
            .physics
            .positions
            .get(model_entity.0)
            .and_then(|p| *p)
            .unwrap_or(Position::origin())
            .0;

        let prev_vel = world
            .physics
            .velocities
            .get(model_entity.0)
            .and_then(|v| *v)
            .unwrap_or(Velocity(Vec3::ZERO))
            .0;

        match projection {
            Projection::FirstPerson => {
                let cam_rot =
                    Rotation::from_euler(cam.yaw().to_radians(), cam.pitch().to_radians(), 0.0)
                        .quat();

                let forward = cam_rot * -Vec3::Z;
                self.eye = player_pos + Vec3::Y * 1.6;
                self.target = self.eye + forward;
                self.up = Vec3::Y;
                world.insert_rotation(
                    model_entity,
                    Rotation::from(glam::Quat::from_rotation_arc(
                        Vec3::Z,
                        cam_rot * -Vec3::Z.normalize(),
                    )),
                );
            }
            Projection::ThirdPerson => {
                let cam_rot = Rotation::from_euler(cam.yaw().to_radians(), 0.0, 0.0).quat();

                let cam_distance = self.model.distance();
                let cam_height = self.model.height();

                let behind = cam_rot * Vec3::Z * cam_distance;
                let above = Vec3::Y * cam_height;
                self.eye = player_pos + behind + above;
                self.target = player_pos + Vec3::Y * 1.0;
                self.up = Vec3::Y;

                world.insert_rotation(
                    model_entity,
                    Rotation::from(glam::Quat::from_rotation_arc(
                        Vec3::Z,
                        cam_rot * -Vec3::Z.normalize(),
                    )),
                );
            }
        }

        let mut forward = self.target - self.eye;
        forward.y = 0.0;

        forward = forward.normalize_or_zero();

        let right = forward.cross(Vec3::Y).normalize_or_zero();

        let mut displacement = Vec3::ZERO;
        let inputs = cam.inputs();

        if inputs[W] {
            displacement += forward;
        }
        if inputs[S] {
            displacement -= forward;
        }
        if inputs[A] {
            displacement -= right;
        }
        if inputs[D] {
            displacement += right;
        }

        let mut velocity = prev_vel;

        if displacement.length_squared() > 0.0 {
            let move_vec = displacement.normalize() * cam.speed();
            let blend = 0.2;
            velocity.x = FloatExt::lerp(prev_vel.x, move_vec.x, blend);
            velocity.z = FloatExt::lerp(prev_vel.z, move_vec.z, blend);
        }

        if inputs.len() > 4 && inputs[4] && prev_vel.y.abs() < 0.01 {
            velocity.y = 5.0;
        }

        world.insert_velocity(model_entity, Velocity(velocity));
    }

    pub fn view_projection_matrix(&self) -> (Mat4, Mat4, Mat4) {
        let view = Mat4::look_at_lh(self.eye, self.target, self.up);
        let proj = Mat4::perspective_lh(self.fovy, self.aspect, self.znear, self.zfar);
        let inv_view = view.inverse();
        let inv_proj = proj.inverse();
        (proj * view, inv_proj, inv_view)
    }
    pub fn frustum(&self) -> Frustum {
        let vp = self.view_projection_matrix();
        Frustum::from_matrix(vp.0)
    }
    pub fn uniform(&self) -> crate::camera::CameraUniform {
        let mut uniform = crate::camera::CameraUniform::new();
        uniform.update(self.view_projection_matrix(), self.eye);
        uniform
    }
    pub fn text_region(&mut self, position: [f32; 2]) -> TextRegion {
        let text_area = TextRegion::new(
            format!(
                "Eye: x: {:.2} y: {:.2} z: {:.2} Target: x: {:.2} y: {:.2} z: {:.2}",
                self.eye().x,
                self.eye().y,
                self.eye().z,
                self.target().x,
                self.target().y,
                self.target().z
            ),
            position,
            glyphon::Color::rgb(1, 1, 1),
        );
        text_area
    }
    pub fn reach_distance(&self) -> f32 {
        self.reach_distance
    }
}

pub fn compute_target_from_rotation(eye: Vec3, yaw: f32, pitch: f32, distance: f32) -> Vec3 {
    let yaw = yaw.to_radians();
    let pitch = pitch.to_radians();
    let look_dir = Vec3::new(
        yaw.sin() * pitch.cos(),
        pitch.sin(),
        -yaw.cos() * pitch.cos(),
    )
    .normalize();
    eye + look_dir * distance
}

pub fn compute_target_from_quat(eye: Vec3, rotation: Quat, distance: f32) -> Vec3 {
    let forward = rotation * -Vec3::Z;
    eye + forward.normalize() * distance
}

pub fn rotation_to_face(forward: Vec3, up: Vec3) -> Quat {
    let f = forward.normalize();
    let u = up.normalize();
    Quat::from_mat4(&glam::Mat4::look_at_rh(Vec3::ZERO, f, u).inverse())
}

/// Returns `Some(t)` for the nearest positive t, or `None` if no hit.
pub fn ray_intersects_ray_sphere(
    ray_origin: Vec3,
    ray_dir: Vec3,
    sphere_center: Vec3,
    sphere_radius: f32,
) -> Option<f32> {
    #[allow(non_snake_case)]
    let L = sphere_center - ray_origin;
    let tca = L.dot(ray_dir);
    let d2 = L.dot(L) - tca * tca;
    if d2 > sphere_radius * sphere_radius {
        return None;
    }
    let thc = (sphere_radius * sphere_radius - d2).sqrt();
    let t0 = tca - thc;
    let t1 = tca + thc;
    if t0 >= 0.0 {
        Some(t0)
    } else if t1 >= 0.0 {
        Some(t1)
    } else {
        None
    }
}
