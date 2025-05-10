
pub struct RenderTargetManager {
    targets: std::collections::HashMap<crate::RenderTargetKind, crate::FrameBuffer>,
}

impl RenderTargetManager {
    pub fn new() -> Self {
        Self {
            targets: std::collections::HashMap::new()
        }
    }

    pub fn insert(
        &mut self,
        fb: crate::FrameBuffer,
        kind: crate::RenderTargetKind
    ) {
        self.targets.insert(kind, fb);
        
    }

    pub fn resize<S: Into<crate::FrameBufferSize> + std::marker::Copy>(&mut self, device: &wgpu::Device, size: S) {
        for fb in &mut self.targets.values_mut() {
            fb.resize(device, size.into());
        }
    }

    pub fn get(&self, kind: &crate::RenderTargetKind) -> Option<&crate::FrameBuffer> {
        self.targets.get(kind)
    }

    pub fn get_mut(&mut self, kind: &crate::RenderTargetKind) -> Option<&mut crate::FrameBuffer> {
        self.targets.get_mut(kind)
    }

    pub fn get_attachment(
        &self,
        kind:&crate::RenderTargetKind,
    ) -> Option<(wgpu::RenderPassColorAttachment, Option<wgpu::RenderPassDepthStencilAttachment>)> {
        self.get(kind).map(|fb| (fb.color_attachment(), fb.depth_attachment()))
    }
}
