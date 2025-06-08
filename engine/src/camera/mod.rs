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
use wgpu::BindingType;
use winit::event::WindowEvent;

use crate::{
    log_warning, BindGroup, Entity, MaterialManager, ModelManager, PipelineManager, Position,
    RenderBindGroupLayouts, Renderable, Rotation, Scale, ShaderManager, TextRegion, TextureManager,
    Velocity, Vertex, VertexInstance, WgpuBuffer, World, GROUND_Y,
};

use glam::{FloatExt, Mat4, Quat, Vec3};
//
// --------------
//  CAMERA
// --------------
//
#[derive(Debug)]
pub struct Camera {
    // --- world‐space camera parameters
    eye: Vec3,
    target: Vec3,
    up: Vec3,

    // --- projection parameters
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,

    // --- helper fields for movement/controls
    forward: Vec3,
    model: CameraModel,
    free_look: bool,
    freeze_movement: bool,
    controls: CameraControls,

    // --- GPU side: uniform buffer + bind group
    uniform_buffer: WgpuBuffer,
    bind_group: wgpu::BindGroup,
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
        let cam_unif = CameraUniform::new();
        let uniform_buffer = WgpuBuffer::from_data(
            device,
            &[cam_unif],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            Some("camera uniform buffer"),
        );
        let bind_group = BindGroup::camera(device, &uniform_buffer);

        let model = CameraModel::new("goblin.obj", "normal.vert.wgsl", "normal.frag.wgsl");
        let eye = Vec3::ZERO;
        let target = Vec3::ZERO;
        let up = Vec3::Y;
        let forward = Vec3::ZERO;
        let fovy = 89.0_f32.to_radians();
        let znear = 0.1;
        let zfar = 100.0;
        let free_look = false;
        let freeze_movement = false;
        let controls = CameraControls::new(speed, sensitivity);

        Camera {
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar,
            forward,
            model,
            free_look,
            freeze_movement,
            controls,
            uniform_buffer,
            bind_group,
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.aspect = width / height;
    }

    pub fn process(&mut self, event: &WindowEvent) -> bool {
        CameraControls::process_event(self, event)
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
    pub fn set_zfar(&mut self, zfar: f32) {
        self.zfar = zfar;
    }
    pub fn znear(&self) -> f32 {
        self.znear
    }
    pub fn set_znear(&mut self, znear: f32) {
        self.znear = znear;
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
            return;
        } else {
            self.set_zfar(1.0);
            self.set_znear(-1.0);
        }
    }

    pub fn upload(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        projection: Projection,
        screen_h: f32,
        screen_w: f32,
    ) {
        let (view, proj) = {
            let view = Mat4::look_at_rh(self.eye, self.target, self.up);
            let proj = projection.matrix(self, screen_w, screen_h);
            (view, proj)
        };

        let mut unif = CameraUniform::new();
        unif.update(view, proj, self.eye);

        self.uniform_buffer.write_data(queue, device, &[unif], None);
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
    pub fn load_model(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        model_manager: &mut ModelManager,
        material_manager: &mut MaterialManager,
        texture_manager: &mut TextureManager,
        shader_manager: &mut ShaderManager,
        pipeline_manager: &mut PipelineManager,
        surface_config: &wgpu::SurfaceConfiguration,
    ) {
        if self.model.model_key().is_none() && !self.model.model().is_empty() {
            let bind_group_layouts = &vec![
                RenderBindGroupLayouts::uniform(),
                RenderBindGroupLayouts::equirect_dst(),
                RenderBindGroupLayouts::material_storage(),
                RenderBindGroupLayouts::normal(),
            ];
            self.model.load_model(
                queue,
                device,
                model_manager,
                material_manager,
                texture_manager,
                shader_manager,
                pipeline_manager,
                &[Vertex::LAYOUT, VertexInstance::LAYOUT],
                bind_group_layouts,
                surface_config,
            );
        }
    }
    pub fn spawn(&mut self, world: &mut World) {
        if let Some(mk) = self.model.model_key() {
            let renderable: Renderable = mk.into();
            let entity = self.model.entity().unwrap_or_else(|| {
                let new_ent = world.spawn();
                self.model.set_entity(new_ent);
                new_ent
            });

            world.insert_scale(entity, Scale::one());
            world.insert_position(entity, Position::new(0.0, GROUND_Y + 1.0, 0.0));
            world.insert_renderable(entity, renderable);
        } else {
            log_warning!("No camera model available");
        }
    }

    pub fn update(&mut self, world: &mut World, projection: &Projection) {
        let model_entity = match self.model.entity() {
            Some(e) => e,
            None => return,
        };

        let player_pos: Vec3 = world
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

        if !self.freeze_movement {
            match projection {
                Projection::FirstPerson => {
                    let cam_rot = Rotation::from_euler(
                        self.controls.yaw().to_radians(),
                        self.controls.pitch().to_radians(),
                        0.0,
                    )
                    .quat();
                    let forward = cam_rot * -Vec3::Z;
                    self.eye = player_pos + Vec3::Y * 1.6;
                    self.target = self.eye + forward;
                    self.up = Vec3::Y;

                    world.insert_rotation(
                        model_entity,
                        Rotation::from(Quat::from_rotation_arc(
                            Vec3::Z,
                            (cam_rot * -Vec3::Z).normalize(),
                        )),
                    );
                }

                Projection::Orthographic => {
                    let ortho_height = self.model.distance().max(10.0);
                    self.eye = player_pos + Vec3::Y * ortho_height;
                    self.target = player_pos;
                    self.up = Vec3::Z;

                    world.insert_rotation(model_entity, Rotation::zero());
                }

                Projection::ThirdPerson => {
                    let cam_rot =
                        Rotation::from_euler(self.controls.yaw().to_radians(), 0.0, 0.0).quat();
                    let cam_distance = self.model.distance();
                    let cam_height = self.model.height();

                    let behind = cam_rot * Vec3::Z * cam_distance;
                    let above = Vec3::Y * cam_height;
                    self.eye = player_pos + behind + above;
                    self.target = player_pos + Vec3::Y * 1.0;
                    self.up = Vec3::Y;

                    world.insert_rotation(
                        model_entity,
                        Rotation::from(Quat::from_rotation_arc(
                            Vec3::Z,
                            (cam_rot * -Vec3::Z).normalize(),
                        )),
                    );
                }
            }

            let mut forward_vec = (self.target - self.eye).normalize_or_zero();
            if !self.free_look {
                forward_vec.y = 0.0;
            }
            forward_vec = forward_vec.normalize_or_zero();

            let right = forward_vec.cross(Vec3::Y).normalize_or_zero();

            let mut displacement = Vec3::ZERO;
            let inputs = self.controls.inputs();
            if inputs[W] {
                displacement += forward_vec;
            }
            if inputs[S] {
                displacement -= forward_vec;
            }
            if inputs[A] {
                displacement -= right;
            }
            if inputs[D] {
                displacement += right;
            }

            if self.free_look {
                if inputs.len() > 4 && inputs[4] {
                    displacement += Vec3::Y;
                }
                if displacement.length_squared() > 0.0 {
                    let mv = displacement.normalize() * self.controls.speed();
                    world.insert_velocity(model_entity, Velocity(mv));
                } else {
                    world.insert_velocity(model_entity, Velocity(Vec3::ZERO));
                }
                return;
            }

            let mut velocity = prev_vel;
            if displacement.length_squared() > 0.0 {
                let mv = displacement.normalize() * self.controls.speed();
                let blend = 0.2;
                velocity.x = FloatExt::lerp(prev_vel.x, mv.x, blend);
                velocity.z = FloatExt::lerp(prev_vel.z, mv.z, blend);
            }

            if inputs.len() > 4 && inputs[4] && prev_vel.y.abs() < 0.01 {
                velocity.y = 5.0;
            }

            world.insert_velocity(model_entity, Velocity(velocity));
        }
    }

    pub fn view_projection_matrix(
        &self,
        projection: &Projection,
        screen_h: f32,
        screen_w: f32,
    ) -> (Mat4, Mat4, Mat4) {
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = projection.matrix(self, screen_w, screen_h);
        let inv_view = view.inverse();
        let inv_proj = proj.inverse();
        (proj * view, inv_proj, inv_view)
    }

    pub fn frustum(&self, projection: &Projection, screen_h: f32, screen_w: f32) -> Frustum {
        let (vp, _ip, _iv) = self.view_projection_matrix(projection, screen_w, screen_h);
        Frustum::from_matrix(vp)
    }

    pub fn uniform(&self, projection: &Projection, screen_h: f32, screen_w: f32) -> CameraUniform {
        let (view, proj) = {
            let view = Mat4::look_at_rh(self.eye, self.target, self.up);
            let proj = projection.matrix(self, screen_w, screen_h);
            (view, proj)
        };
        let mut cu = CameraUniform::new();
        cu.update(view, proj, self.eye);
        cu
    }

    pub fn text_region(&mut self, position: [f32; 2]) -> (TextRegion, TextRegion) {
        let camera_info = format!(
            "Eye:  x={:.2} y={:.2} z={:.2}\nTarget:  x={:.2} y={:.2} z={:.2}",
            self.eye.x, self.eye.y, self.eye.z, self.target.x, self.target.y, self.target.z
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
    Quat::from_mat4(&Mat4::look_at_rh(Vec3::ZERO, f, u).inverse())
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
