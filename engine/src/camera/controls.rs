use crate::TextRegion;
use cgmath::num_traits::Float;
use glam::Vec2;
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
    pub speed: f32,
    pub sensitivity: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub zoom: f32,

    forward: bool,
    back: bool,
    left: bool,
    right: bool,
    jump: bool,

    last_mouse: Option<Vec2>,
    mouse_left: MouseState,
    mouse_right: MouseState,
}

impl CameraControls {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity,
            pitch: 0.0,
            yaw: 0.0,
            zoom: 0.0,
            forward: false,
            back: false,
            left: false,
            right: false,
            jump: false,
            last_mouse: None,
            mouse_left: MouseState::new(),
            mouse_right: MouseState::new(),
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
    pub fn process_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = event.state == ElementState::Pressed;
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => {
                        self.forward = pressed;
                        true
                    }
                    PhysicalKey::Code(KeyCode::KeyS) => {
                        self.back = pressed;
                        true
                    }
                    PhysicalKey::Code(KeyCode::KeyA) => {
                        self.left = pressed;
                        true
                    }
                    PhysicalKey::Code(KeyCode::KeyD) => {
                        self.right = pressed;
                        true
                    }
                    PhysicalKey::Code(KeyCode::Space) => {
                        self.jump = pressed;
                        true
                    }
                    _ => false,
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                let new_pos = Vec2::new(position.x as f32, position.y as f32);
                if let Some(last) = self.last_mouse {
                    let delta = (new_pos - last) * self.sensitivity;
                    self.yaw += delta.x;
                    self.pitch = (self.pitch + delta.y).clamp(-89.9, 89.9);
                }
                self.last_mouse = Some(new_pos);
                true
            }

            WindowEvent::MouseInput { button, state, .. } => match button {
                winit::event::MouseButton::Left => {
                    self.mouse_left.update(state.is_pressed());
                    true
                }
                winit::event::MouseButton::Right => {
                    self.mouse_right.update(state.is_pressed());
                    true
                }
                _ => false,
            },

            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_amount = match delta {
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
                self.zoom += scroll_amount.signum() * 0.1;
                true
            }

            _ => false,
        }
    }

    pub fn input_flags(&self) -> [bool; 5] {
        [self.forward, self.left, self.back, self.right, self.jump]
    }

    pub fn reset_mouse(&mut self) {
        self.last_mouse = None;
    }

    pub fn mouse_buttons(&self) -> (bool, bool) {
        (self.mouse_left.is_down, self.mouse_right.is_down)
    }
    pub fn last_mouse_pos(&self) -> Option<Vec2> {
        self.last_mouse
    }
    pub fn mouse_pressed(&self) -> (bool, bool) {
        (self.mouse_left.just_pressed, self.mouse_right.just_pressed)
    }

    pub fn rotation(&self) -> (f32, f32) {
        (self.yaw, self.pitch)
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
