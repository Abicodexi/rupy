use crate::TextureDescriptor;

/// What kind of render target this framebuffer is used for.
/// (Not strictly required by FrameBuffer itself, but useful for naming/metrics.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderTargetKind {
    Scene,
    Hdr,
    Shadow,
    Bloom,
    Custom(&'static str),
}

pub struct FrameBuffer {
    color: crate::Texture,
    depth: Option<crate::Texture>,
    size: (u32, u32),
}

impl FrameBuffer {
    /// Create a color-only render target (no depth).
    pub fn new_color_only(
        device: &wgpu::Device,
        size: (u32, u32),
        format: wgpu::TextureFormat,
        label: &str,
    ) -> Self {
        let color = crate::Texture::new(
            device,
            TextureDescriptor {
                size: wgpu::Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                },
                format,
                mip_level_count: 1,
                view_dim: wgpu::TextureViewDimension::D2,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                address_mode: Some(wgpu::AddressMode::ClampToEdge),
                mag_filter: wgpu::FilterMode::Nearest,
                sampler: None,
            },
            Some(label),
        );

        Self {
            color,
            depth: None,
            size,
        }
    }

    /// Create a color render target with depth.
    pub fn new_with_depth(
        device: &wgpu::Device,
        size: (u32, u32),
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
        label: &str,
    ) -> Self {
        let mut fb = Self::new_color_only(device, size, color_format, label);

        let depth = crate::Texture::new(
            device,
            TextureDescriptor {
                size: wgpu::Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                },
                format: depth_format,
                mip_level_count: 1,
                view_dim: wgpu::TextureViewDimension::D2,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                address_mode: Some(wgpu::AddressMode::ClampToEdge),
                mag_filter: wgpu::FilterMode::Nearest,
                sampler: Some(device.create_sampler(&wgpu::SamplerDescriptor {
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Nearest,
                    min_filter: wgpu::FilterMode::Nearest,
                    mipmap_filter: wgpu::FilterMode::Nearest,
                    compare: Some(wgpu::CompareFunction::LessEqual),
                    ..Default::default()
                })),
            },
            Some("depth buffer"),
        );

        fb.depth = Some(depth);
        fb
    }

    #[inline]
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    #[inline]
    pub fn color(&self) -> &crate::Texture {
        &self.color
    }

    #[inline]
    pub fn depth(&self) -> &Option<crate::Texture> {
        &self.depth
    }

    /// Build a color attachment that clears to black and stores the result.
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

    /// Build a depth attachment that clears depth to 1.0 and stores it.
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

    /// Resize both color and (if present) depth textures.
    pub fn resize(&mut self, device: &wgpu::Device, width: f32, height: f32) {
        let new_size = (width as u32, height as u32);
        if self.size == new_size {
            return;
        }
        self.size = new_size;

        // Recreate color with same format.
        let color_format = self.color.texture.format();
        self.color = crate::Texture::new(
            device,
            TextureDescriptor {
                size: wgpu::Extent3d {
                    width: new_size.0,
                    height: new_size.1,
                    depth_or_array_layers: 1,
                },
                format: color_format,
                mip_level_count: 1,
                view_dim: wgpu::TextureViewDimension::D2,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                address_mode: Some(wgpu::AddressMode::ClampToEdge),
                mag_filter: wgpu::FilterMode::Nearest,
                sampler: None,
            },
            Some("framebuffer color"),
        );

        // If we had depth, recreate with same format & sampler.
        if let Some(old_depth) = self.depth.take() {
            let depth_format = old_depth.texture.format();
            // Reuse the sampler if you like; here we create a fresh one with compare func.
            let depth = crate::Texture::new(
                device,
                TextureDescriptor {
                    size: wgpu::Extent3d {
                        width: new_size.0,
                        height: new_size.1,
                        depth_or_array_layers: 1,
                    },
                    format: depth_format,
                    mip_level_count: 1,
                    view_dim: wgpu::TextureViewDimension::D2,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    address_mode: Some(wgpu::AddressMode::ClampToEdge),
                    mag_filter: wgpu::FilterMode::Nearest,
                    sampler: Some(device.create_sampler(&wgpu::SamplerDescriptor {
                        address_mode_u: wgpu::AddressMode::ClampToEdge,
                        address_mode_v: wgpu::AddressMode::ClampToEdge,
                        address_mode_w: wgpu::AddressMode::ClampToEdge,
                        mag_filter: wgpu::FilterMode::Nearest,
                        min_filter: wgpu::FilterMode::Nearest,
                        mipmap_filter: wgpu::FilterMode::Nearest,
                        compare: Some(wgpu::CompareFunction::LessEqual),
                        ..Default::default()
                    })),
                },
                Some("depth buffer"),
            );
            self.depth = Some(depth);
            drop(old_depth);
        }
    }
}

