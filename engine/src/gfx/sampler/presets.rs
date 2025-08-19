use wgpu::{
    AddressMode, CompareFunction, FilterMode, Sampler, SamplerBorderColor, SamplerDescriptor,
};

/// A set of useful, well-named sampler presets you can reach for quickly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SamplerPreset {
    /// Point sampling, no mips, clamp to edge.
    NearestClamp,
    /// Linear sampling, no mips, clamp to edge.
    LinearClamp,
    /// Linear sampling, mipmapped, repeat.
    LinearMipmapRepeat,
    /// Linear sampling, no mips, repeat.
    LinearRepeat,
    /// Hardware anisotropic filtering (clamped to device limit). `1` ≈ no anisotropy.
    Anisotropic(u16),
    /// Depth compare sampler for shadow mapping.
    ShadowDepth,
    /// Border (opaque black) for cube/sky sampling beyond edges.
    LinearClampBorderBlack,
}

impl SamplerPreset {
    /// Stable key string for caching (e.g., "sampler:aniso16").
    pub fn as_string(&self) -> String {
        match *self {
            SamplerPreset::NearestClamp => "sampler:nearest_clamp".to_string(),
            SamplerPreset::LinearClamp => "sampler:linear_clamp".to_string(),
            SamplerPreset::LinearMipmapRepeat => "sampler:linear_mipmap_repeat".to_string(),
            SamplerPreset::LinearRepeat => "sampler:linear_repeat".to_string(),
            SamplerPreset::Anisotropic(n) => format!("sampler:aniso{n}"),
            SamplerPreset::ShadowDepth => "sampler:shadow_depth".to_string(),
            SamplerPreset::LinearClampBorderBlack => "sampler:linear_clamp_border_black".to_string(),
        }
    }
}

/// Returns a `SamplerDescriptor` appropriate for the given preset.
/// You can tweak the returned descriptor before creating the sampler if needed.
pub fn sampler_descriptor_for(preset: SamplerPreset, label: Option<&str>) -> SamplerDescriptor {
    match preset {
        SamplerPreset::NearestClamp => SamplerDescriptor {
            label,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            ..Default::default()
        },

        SamplerPreset::LinearClamp => SamplerDescriptor {
            label,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            ..Default::default()
        },

        SamplerPreset::LinearMipmapRepeat => SamplerDescriptor {
            label,
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 1000.0,
            ..Default::default()
        },

        SamplerPreset::LinearRepeat => SamplerDescriptor {
            label,
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            ..Default::default()
        },

        SamplerPreset::Anisotropic(aniso) => SamplerDescriptor {
            label,
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 1000.0,
            anisotropy_clamp: aniso.max(1) as u16, // wgpu clamps to device limit internally
            ..Default::default()
        },

        SamplerPreset::ShadowDepth => SamplerDescriptor {
            label,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            compare: Some(CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            ..Default::default()
        },

        SamplerPreset::LinearClampBorderBlack => SamplerDescriptor {
            label,
            address_mode_u: AddressMode::ClampToBorder,
            address_mode_v: AddressMode::ClampToBorder,
            address_mode_w: AddressMode::ClampToBorder,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            border_color: Some(SamplerBorderColor::OpaqueBlack),
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            ..Default::default()
        },
    }
}

/// Convenience: build a sampler from a preset.
pub fn create_sampler_from_preset(
    device: &wgpu::Device,
    preset: SamplerPreset,
    label: Option<&str>,
) -> Sampler {
    device.create_sampler(&sampler_descriptor_for(preset, label))
}

