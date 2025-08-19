use crate::gfx::buffer::{FrameBuffer, RenderTargetKind};

pub struct RenderTargetManager {
    targets: std::collections::HashMap<RenderTargetKind, FrameBuffer>,
}

impl RenderTargetManager {
    pub fn new() -> Self {
        Self {
            targets: std::collections::HashMap::new(),
        }
    }

    pub fn insert(&mut self, fb: FrameBuffer, kind: RenderTargetKind) {
        self.targets.insert(kind, fb);
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: f32, height: f32) {
        for fb in &mut self.targets.values_mut() {
            fb.resize(device, width, height);
        }
    }

    pub fn get(&self, kind: &RenderTargetKind) -> Option<&FrameBuffer> {
        self.targets.get(kind)
    }

    pub fn get_mut(&mut self, kind: &RenderTargetKind) -> Option<&mut FrameBuffer> {
        self.targets.get_mut(kind)
    }

    pub fn get_attachment(
        &self,
        kind: &RenderTargetKind,
    ) -> Option<(
        wgpu::RenderPassColorAttachment,
        Option<wgpu::RenderPassDepthStencilAttachment>,
    )> {
        self.get(kind)
            .map(|fb| (fb.color_attachment(), fb.depth_attachment()))
    }
}
