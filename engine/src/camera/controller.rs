use crate::TextRegion;
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
                let down = event.state == ElementState::Pressed;

                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => {
                        self.forward = down;
                        true
                    }
                    PhysicalKey::Code(KeyCode::KeyS) => {
                        self.back = down;
                        true
                    }
                    PhysicalKey::Code(KeyCode::KeyA) => {
                        self.left = down;
                        true
                    }
                    PhysicalKey::Code(KeyCode::KeyD) => {
                        self.right = down;
                        true
                    }
                    PhysicalKey::Code(KeyCode::Space) => {
                        self.jump = down;
                        true
                    }
                    _ => false,
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x as f32, position.y as f32);
                if let Some((lx, ly)) = self.last_mouse {
                    let dx = (x - lx) * self.sensitivity;
                    let dy = (y - ly) * self.sensitivity;
                    self.yaw += dx;
                    self.pitch = (self.pitch + dy).clamp(-89.9, 89.9);
                }
                self.last_mouse = Some((x, y));
                true
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let MouseScrollDelta::LineDelta(_, _scroll) = delta {}
                true
            }
            _ => false,
            _ => false,
        }
    }
    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.zoom = -match delta {
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
        };
    }

    pub fn rotation(&self) -> (f32, f32) {
        (self.yaw, self.pitch)
    }

    pub fn inputs(&self) -> [bool; 5] {
        [self.forward, self.right, self.back, self.left, self.jump]
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
