use crate::{EngineError, Vertex2d, WgpuBuffer};

pub struct Renderer2d {
    pub max_sprites: usize,
    pub staging_vertices: Vec<Vertex2d>,
    pub vertex_buffer: WgpuBuffer,
    pub index_buffer: WgpuBuffer,
}

impl Renderer2d {
    pub fn new(device: &wgpu::Device) -> Result<Self, EngineError> {
        let max_sprites = 1_000;
        let vertex_capacity = max_sprites * 4;
        let index_capacity = max_sprites * 6;

        let vertex_buffer = WgpuBuffer::with_capacity(
            device,
            (vertex_capacity * std::mem::size_of::<Vertex2d>()) as wgpu::BufferAddress,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            Some("2D Vertex Buffer"),
        );

        let mut initial_indices = Vec::with_capacity(index_capacity);
        for i in 0..max_sprites {
            let base = (i * 4) as u16;
            initial_indices.extend_from_slice(&[
                base,
                base + 1,
                base + 2,
                base + 2,
                base + 3,
                base + 0,
            ]);
        }
        let index_buffer = WgpuBuffer::from_data(
            device,
            bytemuck::cast_slice(&initial_indices) as &[u8],
            wgpu::BufferUsages::INDEX,
            Some("2D Index Buffer"),
        );

        Ok(Self {
            vertex_buffer,
            index_buffer,
            max_sprites,
            staging_vertices: Vec::with_capacity(vertex_capacity),
        })
    }

    pub fn begin_batch(&mut self) {
        self.staging_vertices.clear();
    }

    pub fn push_quad(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        uv_rect: [f32; 4],
        color: [f32; 4],
        texture_index: i32,
    ) {
        let (u0, v0, u1, v1) = (uv_rect[0], uv_rect[1], uv_rect[2], uv_rect[3]);
        let verts = [
            Vertex2d::new([x, y], [u0, v0], color, texture_index),
            Vertex2d::new([x, y + h], [u0, v1], color, texture_index),
            Vertex2d::new([x + w, y + h], [u1, v1], color, texture_index),
            Vertex2d::new([x + w, y], [u1, v0], color, texture_index),
        ];

        self.staging_vertices.extend_from_slice(&verts);
    }

    pub fn flush(&mut self, queue: &wgpu::Queue, rpass: &mut wgpu::RenderPass<'_>) {
        let num_verts = self.staging_vertices.len();
        if num_verts == 0 {
            return;
        }

        queue.write_buffer(
            &self.vertex_buffer.get(),
            0,
            bytemuck::cast_slice(&self.staging_vertices),
        );

        rpass.set_vertex_buffer(0, self.vertex_buffer.get().slice(..));
        rpass.set_index_buffer(self.index_buffer.get().slice(..), wgpu::IndexFormat::Uint16);

        let index_count = (num_verts as u32 / 4) * 6;
        rpass.draw_indexed(0..index_count, 0, 0..1);

        self.staging_vertices.clear();
    }
    pub fn draw_filled_rect(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [f32; 4],
        uv: [f32; 4],
        texture_index: i32,
    ) {
        self.push_quad(x, y, w, h, uv, color, texture_index);
    }

    pub fn draw_image(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [f32; 4],
        uv: [f32; 4],
        texture_index: i32,
    ) {
        self.push_quad(x, y, w, h, uv, color, texture_index);
    }
}
