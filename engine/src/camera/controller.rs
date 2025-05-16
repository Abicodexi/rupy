use crate::{Entity, Rotation, TextRegion, Velocity, World};
use cgmath::{InnerSpace, Zero};

#[derive(Debug)]
pub enum MovementMode {
    Full3D,  // WASD follows look, including pitch (flight controls)
    FPSFlat, // WASD ignores pitch, only follows yaw (classic FPS)
}

#[derive(Debug)]
pub enum Action {
    Projection,
    Movement(bool),
}

#[derive(Debug)]
pub struct CameraController {
    speed: f32,
    sensitivity: f32,
    forward: bool,
    back: bool,
    left: bool,
    right: bool,
    pitch: f32,
    yaw: f32,
    last_mouse: Option<(f32, f32)>,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity,
            forward: false,
            back: false,
            left: false,
            right: false,
            pitch: 0.0,
            yaw: -90.0,
            last_mouse: None,
        }
    }
    pub fn yaw(&self) -> f32 {
        self.yaw
    }
    pub fn pitch(&self) -> f32 {
        self.pitch
    }
    pub fn speed(&self) -> f32 {
        self.speed
    }
    pub fn process(&mut self, event: &winit::event::WindowEvent) -> Action {
        match event {
            winit::event::WindowEvent::KeyboardInput { event, .. } => {
                let down = event.state == winit::event::ElementState::Pressed;

                if event.physical_key
                    == winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyM)
                    && down
                {
                    return Action::Projection;
                }
                match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyW) => {
                        self.forward = down;
                        Action::Movement(true)
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyS) => {
                        self.back = down;
                        Action::Movement(true)
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyA) => {
                        self.left = down;
                        Action::Movement(true)
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyD) => {
                        self.right = down;
                        Action::Movement(true)
                    }
                    _ => Action::Movement(false),
                }
            }
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x as f32, position.y as f32);
                if let Some((lx, ly)) = self.last_mouse {
                    let dx = (x - lx) * self.sensitivity as f32;
                    let dy = (y - ly) * self.sensitivity as f32;
                    self.yaw += dx;
                    self.pitch = (self.pitch + dy).clamp(-89.0, 89.0);
                }
                self.last_mouse = Some((x, y));
                Action::Movement(true)
            }
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                if let winit::event::MouseScrollDelta::LineDelta(_, _scroll) = delta {}
                Action::Movement(true)
            }
            _ => Action::Movement(false),
        }
    }

    pub fn update(&mut self, camera: &mut super::Camera, dt: f64) {
        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();

        let front = cgmath::Vector3 {
            x: yaw_rad.sin() * pitch_rad.cos(),
            y: pitch_rad.sin(),
            z: -yaw_rad.cos() * pitch_rad.cos(),
        }
        .normalize();

        let up = cgmath::Vector3::unit_y();
        let right = front.cross(up).normalize();

        let mut displacement = cgmath::Vector3::zero();
        if self.forward {
            displacement += front;
        }
        if self.back {
            displacement -= front;
        }
        if self.right {
            displacement += right;
        }
        if self.left {
            displacement -= right;
        }

        if displacement.magnitude2() > 0.0 {
            let disp = displacement.normalize() * self.speed * (dt as f32);
            camera.eye += disp;
        }

        camera.target = camera.eye + front;
    }
    pub fn rotation(&self) -> (f32, f32) {
        (self.yaw, self.pitch)
    }
    pub fn compute_movement_and_rotation(
        &self,
        movement_mode: &MovementMode,
    ) -> (cgmath::Vector3<f32>, (f32, f32)) {
        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();

        let front = cgmath::Vector3 {
            x: yaw_rad.sin() * pitch_rad.cos(),
            y: pitch_rad.sin(),
            z: -yaw_rad.cos() * pitch_rad.cos(),
        }
        .normalize();

        let flat_front = cgmath::Vector3 {
            x: yaw_rad.sin(),
            y: 0.0,
            z: -yaw_rad.cos(),
        }
        .normalize();

        let up = cgmath::Vector3::unit_y();
        let right = up.cross(front).normalize();

        let mut move_vec = cgmath::Vector3::zero();
        let use_front = match movement_mode {
            MovementMode::Full3D => front,
            MovementMode::FPSFlat => flat_front,
        };
        if self.forward {
            move_vec += right;
        }
        if self.back {
            move_vec -= right;
        }
        if self.right {
            move_vec += use_front;
        }
        if self.left {
            move_vec -= use_front;
        }
        (move_vec, (self.yaw, self.pitch))
    }

    pub fn apply(
        &self,
        world: &mut World,
        camera_entity: Entity,
        movement_mode: &MovementMode,
        speed: f32,
    ) {
        let (move_vec, (yaw, pitch)) = self.compute_movement_and_rotation(movement_mode);

        world.insert_rotation(camera_entity, Rotation::from_euler(yaw, pitch, 0.0));

        if let Some(rot) = world.rotations[camera_entity.0].as_ref() {
            let world_move_vec = rot.quat() * move_vec;
            world.insert_velocity(camera_entity, Velocity::from(world_move_vec * speed));
        }
    }
    pub fn text_region(&mut self, position: [f32; 2]) -> TextRegion {
        let text_area = TextRegion::new(
            format!("Yaw: {:.2} Pitch: {:.2}", self.yaw, self.pitch),
            position,
            glyphon::Color::rgb(1, 1, 1),
        );
        text_area
    }
}
