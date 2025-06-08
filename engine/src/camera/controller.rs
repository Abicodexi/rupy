use crate::{camera::Camera, TextRegion};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseScrollDelta, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

pub const W: usize = 0;
pub const A: usize = 1;
pub const S: usize = 2;
pub const D: usize = 3;
pub const J: usize = 4;
pub const WASDJ: [usize; 5] = [W, A, S, D, J];

#[derive(Debug)]
struct MouseState {
    is_down: bool,
    just_pressed: bool,
}

impl MouseState {
    pub fn new() -> Self {
        Self {
            is_down: false,
            just_pressed: false,
        }
    }
    fn update(&mut self, is_down_now: bool) {
        self.just_pressed = is_down_now && !self.is_down;
        self.is_down = is_down_now;
    }
}

#[derive(Debug)]
pub enum Action {
    Projection,
    Movement(bool),
}

#[derive(Debug)]
pub struct CameraControls {
    speed: f32,
    sensitivity: f32,
    forward: bool,
    back: bool,
    left: bool,
    right: bool,
    jump: bool,
    pitch: f32,
    yaw: f32,
    zoom: f32,
    last_mouse: Option<(f32, f32)>,
    mouse_state_left: MouseState,
    mouse_state_right: MouseState,
}

impl CameraControls {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity,
            forward: false,
            back: false,
            left: false,
            right: false,
            jump: false,
            pitch: 0.0,
            yaw: 0.0,
            zoom: 0.0,
            last_mouse: None,
            mouse_state_left: MouseState::new(),
            mouse_state_right: MouseState::new(),
        }
    }
    pub fn set_zoom(&mut self, level: f32) {
        self.zoom = level
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
    pub fn sensitivity(&self) -> f32 {
        self.sensitivity
    }
    pub fn zoom(&self) -> f32 {
        self.zoom
    }
    pub fn last_mouse_pos(&self) -> Option<(f32, f32)> {
        self.last_mouse
    }
    pub fn mouse_state_is_down(&self) -> (bool, bool) {
        (
            self.mouse_state_left.is_down,
            self.mouse_state_right.is_down,
        )
    }
    pub fn mouse_just_pressed(&self) -> (bool, bool) {
        (
            self.mouse_state_left.just_pressed,
            self.mouse_state_right.just_pressed,
        )
    }
    pub fn process_event(camera: &mut Camera, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                let down = event.state == ElementState::Pressed;

                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => {
                        camera.controls.forward = down;
                        true
                    }
                    PhysicalKey::Code(KeyCode::KeyS) => {
                        camera.controls.back = down;
                        true
                    }
                    PhysicalKey::Code(KeyCode::KeyA) => {
                        camera.controls.left = down;
                        true
                    }
                    PhysicalKey::Code(KeyCode::KeyD) => {
                        camera.controls.right = down;
                        true
                    }
                    PhysicalKey::Code(KeyCode::Space) => {
                        camera.controls.jump = down;
                        true
                    }
                    PhysicalKey::Code(KeyCode::KeyL) => {
                        let free_look = if camera.free_look() { false } else { true };
                        camera.set_free_look(free_look);
                        true
                    }
                    _ => false,
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x as f32, position.y as f32);
                if let Some((lx, ly)) = camera.controls.last_mouse {
                    let dx = (x - lx) * camera.controls.sensitivity;
                    let dy = (y - ly) * camera.controls.sensitivity;
                    camera.controls.yaw += dx;
                    camera.controls.pitch = (camera.controls.pitch + dy).clamp(-89.9, 89.9);
                }
                camera.controls.last_mouse = Some((x, y));
                true
            }
            WindowEvent::MouseInput { state, button, .. } => match button {
                winit::event::MouseButton::Left => {
                    camera.controls.mouse_state_left.update(state.is_pressed());
                    true
                }
                winit::event::MouseButton::Right => {
                    camera.controls.mouse_state_right.update(state.is_pressed());
                    true
                }
                _ => false,
            },
            WindowEvent::MouseWheel { delta, .. } => {
                camera.controls.process_scroll(delta);
                true
            }
            _ => false,
        }
    }
    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.zoom += match delta {
            MouseScrollDelta::LineDelta(_, scroll) => {
                if *scroll > 0.0 {
                    0.1
                } else {
                    -0.1
                }
            }
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => {
                if *scroll > 0.0 {
                    0.1
                } else {
                    -0.1
                }
            }
        };
    }

    pub fn rotation(&self) -> (f32, f32) {
        (self.yaw, self.pitch)
    }

    pub fn inputs(&self) -> [bool; 5] {
        [self.forward, self.left, self.back, self.right, self.jump]
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
