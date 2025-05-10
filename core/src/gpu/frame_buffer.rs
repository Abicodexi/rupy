#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderTargetKind {
    Scene,
    Hdr,
    Shadow,
    Bloom,
    Custom(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct FrameBufferSize(pub u32, u32);
impl From<winit::dpi::PhysicalSize<u32>> for FrameBufferSize {
    fn from(value: winit::dpi::PhysicalSize<u32>) -> Self {
        FrameBufferSize(value.width,value.height)
    }
}
impl From<(u32, u32)> for FrameBufferSize {
    fn from(value: (u32, u32)) -> Self {
        FrameBufferSize(value.0, value.1)
    }
}
impl From<wgpu::SurfaceConfiguration> for FrameBufferSize {
    fn from(value: wgpu::SurfaceConfiguration) -> Self {
        FrameBufferSize(value.width, value.height)
    }
}
impl From<&wgpu::SurfaceConfiguration> for FrameBufferSize {
    fn from(value: &wgpu::SurfaceConfiguration) -> Self {
        FrameBufferSize(value.width, value.height)
    }
}
impl Into<glyphon::Resolution> for FrameBufferSize {
    fn into(self) -> glyphon::Resolution {
        glyphon::Resolution { width: self.0, height: self.1 }
    }
}

pub struct FrameBuffer {
    color: crate::Texture,
    depth: Option<crate::Texture>,
    size: FrameBufferSize,
}

impl FrameBuffer {
    pub fn new_color_only(
        device: &wgpu::Device,
        size: FrameBufferSize,
        format: wgpu::TextureFormat,
        label: &str,
    ) -> Self {
        let color = crate::Texture::new(
            device,
            wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            format,
            1,
            wgpu::TextureViewDimension::D2,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            Some(wgpu::AddressMode::ClampToEdge),
            wgpu::FilterMode::Linear,
            None,
            Some(label),
        );
        Self {
            color,
            depth: None,
            size,
        }
    }

    pub fn new_with_depth(
        device: &wgpu::Device,
        size: FrameBufferSize,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
        label: &str,
    ) -> Self {
        let mut fb = Self::new_color_only(device, size, color_format, label);
        let depth = crate::Texture::new(
            device,
            wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            depth_format,
            1,
            wgpu::TextureViewDimension::D2,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            Some(wgpu::AddressMode::ClampToEdge),
            wgpu::FilterMode::Linear,
            Some(device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual),
                ..Default::default()
            })),
            Some("depth buffer"),
        );
        fb.depth = Some(depth);
        fb
    }
    pub fn color(&self) -> &crate::Texture {
        &self.color
    }
    pub fn depth(&self) -> &Option<crate::Texture> {
        &self.depth
    }
    pub fn color_attachment(&self) -> wgpu::RenderPassColorAttachment {
        wgpu::RenderPassColorAttachment {
            view: &self.color.view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
        }
    }

    pub fn depth_attachment(&self) -> Option<wgpu::RenderPassDepthStencilAttachment> {
        self.depth.as_ref().map(|d| wgpu::RenderPassDepthStencilAttachment {
            view: &d.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        })
    }

    pub fn resize(&mut self, device: &wgpu::Device, new_size: FrameBufferSize) {
        if self.size != new_size {
            let format = self.color.texture.format();
            self.color = crate::Texture::new(
                device,
                wgpu::Extent3d {
                    width: new_size.0,
                    height: new_size.1,
                    depth_or_array_layers: 1,
                },
                format,
                1,
                wgpu::TextureViewDimension::D2,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                Some(wgpu::AddressMode::ClampToEdge),
                wgpu::FilterMode::Linear,
                None,
                Some("resized framebuffer color"),
            );
            if let Some(depth) = self.depth.as_mut() {
                let format = depth.texture.format();
                *depth = crate::Texture::new(
                    device,
                    wgpu::Extent3d {
                        width: new_size.0,
                        height: new_size.1,
                        depth_or_array_layers: 1,
                    },
                    format,
                    1,
                    wgpu::TextureViewDimension::D2,
                    wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                    Some(wgpu::AddressMode::ClampToEdge),
                    wgpu::FilterMode::Linear,
                    None,
                    Some("resized framebuffer depth"),
                );
            }
            self.size = new_size;
        }
    }
}
