use crate::{CacheKey, WgpuBuffer, WgpuBufferCache};

pub enum Mesh {
    Shared { key: CacheKey, count: u32 },
    Unique { buffer: WgpuBuffer, count: u32 },
}

impl Mesh {
    pub fn draw(
        &self,
        rpass: &mut wgpu::RenderPass,
        pipeline: &wgpu::RenderPipeline,
        bind_groups: Vec<&wgpu::BindGroup>,
        wgpu_buffer_cache: &WgpuBufferCache,
    ) {
        rpass.set_pipeline(pipeline);
        let mut index: u32 = 0;
        for bind_group in bind_groups {
            rpass.set_bind_group(index as u32, bind_group, &[]);
            index += 1;
        }

        match self {
            Mesh::Shared { key, count } => {
                if let Some(vb) = wgpu_buffer_cache.get_buffer(key) {
                    rpass.set_vertex_buffer(0, vb.buffer.slice(..));
                    rpass.draw(0..*count, 0..1);
                }
            }
            Mesh::Unique { buffer, count } => {
                rpass.set_vertex_buffer(0, buffer.buffer.slice(..));
                rpass.draw(0..*count, 0..1);
            }
        }
    }
}
