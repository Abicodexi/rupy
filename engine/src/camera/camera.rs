use wgpu::BindingType;
use winit::{
    event::{ElementState, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{
    camera::{CameraControls, CameraModel, CameraTransform, CameraUniform, Frustum, Projection}, gfx::{bind_group::{camera_group, camera_layout, global_uniform_layout, material_storage_layout, normal_texture_layout, skybox_cubemap_layout}, buffer::WgpuBuffer}, log_info, AssetService, CacheKey, Entity, Position, Renderable, Rotation, Scale, TextRegion, Velocity
};

use glam::{FloatExt, Mat4, Quat, Vec3};

pub struct CameraUpdates {
    pub entity: Entity,
    pub velocity: Option<Velocity>,
    pub rotation: Option<Rotation>,
}

#[derive(Debug)]
pub struct Camera {
    transform: CameraTransform,
    model: CameraModel,
    free_look: bool,
    freeze_movement: bool,
    controls: CameraControls,
    uniform_buffer: WgpuBuffer,
    bind_group: wgpu::BindGroup,
    frustum: Frustum,
    uniform: CameraUniform,
}

impl Camera {
    pub const BINDING: BindingType = wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<CameraUniform>() as u64),
    };

    pub fn new(
        device: &wgpu::Device,
        screen_w: f32,
        screen_h: f32,
        speed: f32,
        sensitivity: f32,
    ) -> Self {
        let aspect = screen_w / screen_h;
        let cam_unif = crate::camera::CameraUniform::new();
        let uniform_buffer = WgpuBuffer::from_data(
            device,
            &[cam_unif],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            Some("camera uniform buffer"),
        );
        let bind_group = camera_group(device, &camera_layout(device), &uniform_buffer);

        let model = CameraModel::new("goblin.obj", "normal.vert.wgsl", "normal.frag.wgsl");
        let controls = CameraControls::new(speed, sensitivity);
        let transform = CameraTransform::new(aspect, 89.0_f32.to_radians(), 0.1, 100.0);
        Self {
            transform,
            model,
            free_look: false,
            freeze_movement: false,
            controls,
            uniform_buffer,
            bind_group,
            frustum: Frustum::new(),
            uniform: CameraUniform::new(),
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.transform.set_aspect(width / height);
    }

    pub fn process(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { event, .. } => match event.physical_key {
                PhysicalKey::Code(KeyCode::KeyL) => {
                    if event.state == ElementState::Pressed && !event.repeat {
                        log_info!("Changing free look: {}", self.free_look);
                        self.free_look = !self.free_look;
                    }
                }

                _ => {}
            },
            _ => {}
        }
        self.controls.process_event(event)
    }
    pub fn mark_dirty(&mut self) {
        self.transform.mark_dirty(true);
    }
    pub fn forward(&self) -> Vec3 {
        self.transform.eye
    }
    pub fn eye(&self) -> Vec3 {
        self.transform.eye
    }

    pub fn zfar(&self) -> f32 {
        self.transform.zfar
    }

    pub fn set_zfar(&mut self, zfar: f32) {
        self.transform.set_zfar(zfar);
    }

    pub fn znear(&self) -> f32 {
        self.transform.znear
    }

    pub fn set_znear(&mut self, znear: f32) {
        self.transform.set_znear(znear);
    }

    pub fn look_at(&mut self, pos: Vec3) {
        self.transform.set_target(pos);
    }
    pub fn target(&self) -> Vec3 {
        self.transform.target
    }

    pub fn up(&self) -> Vec3 {
        self.transform.up
    }

    pub fn fovy(&self) -> f32 {
        self.transform.fovy
    }
    pub fn aspect(&self) -> f32 {
        self.transform.aspect
    }
    pub fn is_frozen(&self) -> bool {
        self.freeze_movement
    }
    pub fn freeze(&mut self) {
        self.freeze_movement = true;
    }
    pub fn unfreeze(&mut self) {
        self.freeze_movement = false;
    }
    pub fn set_free_look(&mut self, val: bool) {
        self.free_look = val;
    }
    pub fn free_look(&self) -> bool {
        self.free_look
    }
    pub fn entity(&self) -> Option<Entity> {
        self.model.entity()
    }
    pub fn buffer(&self) -> &WgpuBuffer {
        &self.uniform_buffer
    }
    pub fn controller(&self) -> &CameraControls {
        &self.controls
    }
    pub fn set_projection_far_near(&mut self, projection: &Projection) {
        if matches!(
            *projection,
            Projection::FirstPerson | Projection::ThirdPerson
        ) {
            self.set_zfar(100.0);
            self.set_znear(0.1);
        } else {
            self.set_zfar(1.0);
            self.set_znear(-1.0);
        }
    }

    pub fn upload(&mut self, queue: &wgpu::Queue, device: &wgpu::Device) {
        if self.transform.is_dirty() {
            let data = [self.uniform()];
            self.uniform_buffer.write_data(queue, device, &data, None);
            self.transform.mark_dirty(false);
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
    pub fn load_model(&mut self, service: &AssetService, cfg: &wgpu::SurfaceConfiguration) {
        let device = service.device();
        self.model.load_model(
            service,
            vec![
               global_uniform_layout(device),
               skybox_cubemap_layout(device),
               material_storage_layout(device),
               normal_texture_layout(device),
            ],
            cfg.format,
        );
    }
    pub fn spawn(&mut self, entity: Entity) -> Option<(Renderable, Scale, Position)> {
        self.model.set_entity(entity);
        let renderable: Renderable = CacheKey::from(self.model.model()).into();
        let scale: Scale = Scale::one();
        let position: Position = Position::origin();
        Some((renderable, scale, position))
    }

    pub fn update(
        &mut self,
        projection: &Projection,
        position: Vec3,
        previous_velocity: Vec3,
        screen_w: f32,
        screen_h: f32,
    ) -> Option<CameraUpdates> {
        let entity = self.model.entity()?;

        projection.apply_to_transform(
            &mut self.transform,
            position,
            self.controls.yaw().to_radians(),
            self.controls.pitch().to_radians(),
            &self.model,
        );

        self.transform.update_matrices(
            self.fovy(),
            self.aspect(),
            self.znear(),
            self.zfar(),
            screen_w,
            screen_h,
            projection,
        );
        let forward_vec = (self.transform.target - self.transform.eye).normalize_or_zero();
        let up = Vec3::Y;
        let right = forward_vec.cross(up).normalize_or_zero();

        let rotation = {
            Some(Rotation::from(Quat::from_mat4(
                &Mat4::look_at_lh(Vec3::ZERO, forward_vec, up).inverse(),
            )))
        };

        if self.freeze_movement {
            return Some(CameraUpdates {
                entity,
                velocity: None,
                rotation,
            });
        }

        let mut displacement = Vec3::ZERO;
        let inputs = self.controls.input_flags();

        let mut effective_forward = forward_vec;
        if !self.free_look {
            effective_forward.y = 0.0;
            effective_forward = effective_forward.normalize_or_zero();
        } else {
            if inputs.len() > crate::camera::Q && inputs[crate::camera::Q] {
                displacement -= up;
            }
            if inputs.len() > crate::camera::E && inputs[crate::camera::E] {
                displacement += up;
            }
        }

        if inputs[crate::camera::W] {
            displacement += effective_forward;
        }
        if inputs[crate::camera::S] {
            displacement -= effective_forward;
        }
        if inputs[crate::camera::A] {
            displacement -= right;
        }
        if inputs[crate::camera::D] {
            displacement += right;
        }

        let velocity = if self.free_look {
            if displacement.length_squared() > 0.0 {
                Some(Velocity(displacement.normalize() * self.controls.speed()))
            } else {
                Some(Velocity(Vec3::ZERO))
            }
        } else {
            let mut new_velocity = previous_velocity;
            if displacement.length_squared() > 0.0 {
                let mv = displacement.normalize() * self.controls.speed();
                new_velocity.x = FloatExt::lerp(previous_velocity.x, mv.x, 0.2);
                new_velocity.z = FloatExt::lerp(previous_velocity.z, mv.z, 0.2);
            }

            if inputs.len() > crate::camera::JUMP
                && inputs[crate::camera::JUMP]
                && previous_velocity.y.abs() < 0.01
            {
                new_velocity.y = 5.0;
            }

            Some(Velocity(new_velocity))
        };

        let (vp, inv_proj, inv_view) = self.view_projection_matrix(projection, screen_h, screen_w);
        self.update_uniform(vp, inv_proj, inv_view, position);
        self.update_frustum(vp);

        Some(CameraUpdates {
            entity,
            velocity,
            rotation,
        })
    }

    fn view_projection_matrix(
        &self,
        projection: &Projection,
        screen_h: f32,
        screen_w: f32,
    ) -> (Mat4, Mat4, Mat4) {
        let view = self.transform.view();
        let proj = projection.matrix(
            self.fovy(),
            self.aspect(),
            self.znear(),
            self.zfar(),
            screen_w,
            screen_h,
        );
        let inv_view = self.transform.inv_view();
        let inv_proj = self.transform.inv_proj();
        (proj * view, inv_proj, inv_view)
    }

    fn update_frustum(&mut self, vp: Mat4) {
        self.frustum = Frustum::from_matrix(vp);
    }
    pub fn frustum(&self) -> Frustum {
        self.frustum
    }
    fn update_uniform(&mut self, vp: Mat4, inv_proj: Mat4, inv_view: Mat4, position: Vec3) {
        self.uniform.update(vp, inv_proj, inv_view, position);
    }
    pub fn uniform(&self) -> CameraUniform {
        self.uniform
    }

    pub fn text_region(&mut self, position: [f32; 2]) -> (TextRegion, TextRegion) {
        let camera_info = format!(
            "Eye:  x={:.2} y={:.2} z={:.2}\nTarget:  x={:.2} y={:.2} z={:.2}",
            self.transform.eye.x,
            self.transform.eye.y,
            self.transform.eye.z,
            self.transform.target.x,
            self.transform.target.y,
            self.transform.target.z
        );
        let tr_camera = TextRegion::new(camera_info, position, glyphon::Color::rgb(1, 1, 1));
        let tr_controls = self.controls.text_region(position);
        (tr_camera, tr_controls)
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
    Quat::from_mat4(&Mat4::look_at_lh(Vec3::ZERO, f, u).inverse())
}

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
