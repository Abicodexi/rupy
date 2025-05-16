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
    log_debug, log_warning, BindGroupLayouts, Entity, ModelManager, Position, Renderable, Rotation,
    Scale, TextRegion, Vertex, VertexInstance, WgpuBuffer, World,
};
use cgmath::{EuclideanSpace, InnerSpace, Vector3, Zero};

#[derive(Debug)]
pub struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: cgmath::Deg<f32>,
    znear: f32,
    zfar: f32,
    forward: cgmath::Vector3<f32>,
    pick_distance: f32,
    movement: MovementMode,
    model: CameraModel,
    bind_group: wgpu::BindGroup,
    uniform_buffer: WgpuBuffer,
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
    pub fn new(device: &wgpu::Device, movement: MovementMode, aspect: f32) -> Self {
        let model = CameraModel::new("goblin.obj", "v_normal.wgsl");
        let uniform_buffer = WgpuBuffer::from_data(
            device,
            &[CameraUniform::default()],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            Some("camera uniform buffer"),
        );
        let bind_group = crate::BindGroup::camera(device, &uniform_buffer);
        Camera {
            eye: cgmath::Point3::new(0.0, 0.0, 0.0),
            target: cgmath::Point3::new(0.0, 0.0, 0.0),
            up: cgmath::Vector3::unit_y(),
            aspect,
            fovy: cgmath::Deg(45.0),
            znear: 0.1,
            zfar: 100.0,
            pick_distance: 10.0,
            forward: cgmath::Vector3::zero(),
            movement,
            model,
            bind_group,
            uniform_buffer,
        }
    }
    pub fn forward(&self) -> Vector3<f32> {
        self.forward
    }
    pub fn movement(&self) -> &MovementMode {
        &self.movement
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
    pub fn eye(&self) -> &cgmath::Point3<f32> {
        &self.eye
    }
    pub fn target(&self) -> &cgmath::Point3<f32> {
        &self.target
    }
    pub fn up(&self) -> &cgmath::Vector3<f32> {
        &self.up
    }
    pub fn fovy(&self) -> &cgmath::Deg<f32> {
        &self.fovy
    }

    pub fn add_model(
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
    pub fn spawn(
        &mut self,
        world: &mut World,
        model_manager: &mut ModelManager,
        surface_configuration: &wgpu::SurfaceConfiguration,
    ) {
        if self.model.model_key().is_none() && !self.model.model().is_empty() {
            self.model.load_model(
                model_manager,
                &[Vertex::LAYOUT, VertexInstance::LAYOUT],
                vec![
                    BindGroupLayouts::uniform().clone(),
                    BindGroupLayouts::equirect_dst().clone(),
                    BindGroupLayouts::material_storage().clone(),
                    BindGroupLayouts::normal().clone(),
                ],
                surface_configuration,
            );
        }
        if let Some(model_key) = self.model.model_key() {
            let renderable: Renderable = model_key.into();
            let view_pos: [f32; 3] = self.uniform().view_pos();
            let position = Position::new(view_pos[0], view_pos[1], view_pos[2]);
            let rotation = Rotation::from(cgmath::Quaternion::new(0.0, 0.0, 0.0, 0.0));
            let scale = Scale::from(cgmath::Vector3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            });

            let entity = if let Some(ent) = self.model.entity() {
                ent
            } else {
                let new_ent = world.spawn();
                self.model.set_entity(new_ent);
                new_ent
            };

            world.insert_position(entity, position);
            world.insert_scale(entity, scale);
            world.insert_rotation(entity, rotation);
            world.insert_renderable(entity, renderable);
            log_debug!("Spawned camera model: {}", self.model.model());
        } else {
            log_warning!("No camera model available");
            return;
        }
    }
    pub fn update(&mut self, world: &World, projection: Projection) {
        if let Some(camera_entity) = self.model.entity() {
            let (eye, target, up) = self.calculate_transform(world, camera_entity, projection);
            self.eye = eye;
            self.target = target;
            self.up = up;
            self.forward = (self.target - self.eye).normalize();
        }
    }

    pub fn calculate_transform(
        &self,
        world: &World,
        camera_entity: Entity,
        projection: Projection,
    ) -> (
        cgmath::Point3<f32>,
        cgmath::Point3<f32>,
        cgmath::Vector3<f32>,
    ) {
        match projection {
            Projection::FirstPerson => {
                let pos = world.positions[camera_entity.0]
                    .as_ref()
                    .expect("Camera entity must have position")
                    .to_point3();
                let rot = world.rotations[camera_entity.0]
                    .as_ref()
                    .expect("Camera entity must have rotation")
                    .quat();
                let target = compute_target_from_quat(pos, rot, 1.0);
                let up = rot * cgmath::Vector3::unit_y();
                (pos, target, up)
            }
            Projection::ThirdPerson => {
                let pos = world.positions[camera_entity.0]
                    .as_ref()
                    .expect("Camera entity must have position")
                    .to_point3();
                let rot = world.rotations[camera_entity.0]
                    .as_ref()
                    .expect("Camera entity must have rotation")
                    .quat();

                let backward = rot * -cgmath::Vector3::unit_z();
                let right = rot * cgmath::Vector3::unit_x();

                let eye = pos
                    + backward.normalize() * self.model.distance()
                    + cgmath::Vector3::unit_y() * self.model.height()
                    + right * self.model.shoulder_offset();
                let target = pos + cgmath::Vector3::unit_y() * self.model.target_height();
                let up = cgmath::Vector3::unit_y();
                (eye, target, up)
            }
        }
    }
    pub fn view_projection_matrix(
        &self,
    ) -> (
        cgmath::Matrix4<f32>,
        cgmath::Matrix4<f32>,
        cgmath::Matrix4<f32>,
    ) {
        use cgmath::perspective;
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = perspective(self.fovy, self.aspect, self.znear, self.zfar);
        let inv_view = cgmath::SquareMatrix::invert(&view).unwrap();
        let inv_proj = cgmath::SquareMatrix::invert(&proj).unwrap();
        (proj * view, inv_proj, inv_view)
    }
    pub fn frustum(&self) -> Frustum {
        let vp = self.view_projection_matrix();
        Frustum::from_matrix(vp.0)
    }
    pub fn uniform(&self) -> crate::camera::CameraUniform {
        let mut uniform = crate::camera::CameraUniform::new();
        uniform.update(self.view_projection_matrix(), self.eye.to_vec());
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
    /// Returns `Some(t)` for the nearest positive t, or `None` if no hit.
    fn intersect_ray_sphere(
        &self,
        ray_origin: cgmath::Point3<f32>,
        ray_dir: cgmath::Vector3<f32>,
        sphere_center: cgmath::Point3<f32>,
        sphere_radius: f32,
    ) -> Option<f32> {
        let L = sphere_center - ray_origin;
        let tca = cgmath::InnerSpace::dot(L, ray_dir);
        let d2 = cgmath::InnerSpace::dot(L, L) - tca * tca;
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
    pub fn pick_distance(&self) -> f32 {
        self.pick_distance
    }
    pub fn is_looking_at(&self, center: cgmath::Point3<f32>, radius: f32) -> Option<f32> {
        self.intersect_ray_sphere(*self.eye(), self.forward, center, radius)
    }
}

pub fn compute_target_from_rotation(
    eye: cgmath::Point3<f32>,
    yaw: f32,
    pitch: f32,
    distance: f32,
) -> cgmath::Point3<f32> {
    let yaw_rad = yaw.to_radians();
    let pitch_rad = pitch.to_radians();

    let direction = cgmath::Vector3 {
        x: yaw_rad.cos() * pitch_rad.cos(),
        y: pitch_rad.sin(),
        z: yaw_rad.sin() * pitch_rad.cos(),
    }
    .normalize();

    eye + direction * distance
}

pub fn compute_target_from_quat(
    eye: cgmath::Point3<f32>,
    rotation: cgmath::Quaternion<f32>,
    distance: f32,
) -> cgmath::Point3<f32> {
    let forward = rotation * cgmath::Vector3::unit_z();
    eye + forward.normalize() * distance
}
