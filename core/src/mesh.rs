use crate::{CacheKey, WgpuBuffer};

pub enum Mesh {
    Shared { key: CacheKey, count: u32 },
    Unique { buffer: WgpuBuffer, count: u32 },
}
