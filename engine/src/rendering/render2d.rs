use crate::{
    camera::OrthoUniform, create_sprite2d_pipeline, BindGroup, CacheKey, CacheStorage, EngineError,
    ModelManager, RenderBindGroupLayouts, RenderPipelineManager, Texture, Vertex2d, WgpuBuffer,
};

pub struct Renderer2d {
    pub ortho_buffer: WgpuBuffer,
    pub ortho_bind_group: wgpu::BindGroup,

    pub texture_bind_group: wgpu::BindGroup,

    pub vertex_buffer: WgpuBuffer,
    pub index_buffer: WgpuBuffer,
    pub max_sprites: usize,

    pub staging_vertices: Vec<Vertex2d>,

    pub sprite2d_pipeline_key: CacheKey,
}

impl Renderer2d {
    pub fn new(
        width: u32,
        height: u32,
        model_manager: &mut ModelManager,
    ) -> Result<Self, EngineError> {
        let ortho_uniform = OrthoUniform::new(width as f32, height as f32);
        let ortho_buffer = WgpuBuffer::from_data(
            &model_manager.device,
            bytemuck::bytes_of(&ortho_uniform),
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            Some("Ortho Uniform Buffer"),
        );
        let ortho_bind_group_layout = RenderBindGroupLayouts::ortho_uniform();
        let ortho_bind_group = model_manager
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("OrthoBindGroup"),
                layout: &ortho_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ortho_buffer.get().as_entire_binding(),
                }],
            });

        let max_sprites = 1_000;
        let vertex_capacity = max_sprites * 4;
        let index_capacity = max_sprites * 6;

        let vertex_buffer = WgpuBuffer::with_capacity(
            &model_manager.device,
            (vertex_capacity * std::mem::size_of::<Vertex2d>()) as wgpu::BufferAddress,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            Some("2D Vertex Buffer"),
        );

        let mut initial_indices = Vec::with_capacity(index_capacity);
        for i in 0..max_sprites {
            let base = (i * 4) as u32;
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
            &model_manager.device,
            bytemuck::cast_slice(&initial_indices) as &[u8],
            wgpu::BufferUsages::INDEX,
            Some("2D Index Buffer"),
        );
        let diffuse_texture = pollster::FutureExt::block_on(Texture::from_file(
            &model_manager.queue,
            &model_manager.device,
            "cube-diffuse.jpg",
        ))?;
        let texture_bind_group = BindGroup::texture(&model_manager.device, &diffuse_texture);

        let sprite2d_pipeline_key = CacheKey::from("sprite2d");

        Ok(Self {
            ortho_buffer,
            ortho_bind_group,
            texture_bind_group,
            vertex_buffer,
            index_buffer,
            max_sprites,
            staging_vertices: Vec::with_capacity(vertex_capacity),
            sprite2d_pipeline_key,
        })
    }

    pub fn build_pipelines(
        &mut self,
        device: &wgpu::Device,
        cfg: &wgpu::SurfaceConfiguration,
        pipeline_manager: &mut RenderPipelineManager,
    ) -> Result<(), EngineError> {
        if !pipeline_manager.contains(&self.sprite2d_pipeline_key) {
            let sprite2d_pipeline = create_sprite2d_pipeline(device, cfg.format)?;
            pipeline_manager.insert(self.sprite2d_pipeline_key, sprite2d_pipeline.into());
        }
        Ok(())
    }

    pub fn resize(&mut self, queue: &wgpu::Queue, device: &wgpu::Device, width: u32, height: u32) {
        let ortho = OrthoUniform::new(width as f32, height as f32);
        self.ortho_buffer
            .write_data(queue, device, bytemuck::bytes_of(&ortho), None);
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
    ) {
        if self.staging_vertices.len() + 4 > self.max_sprites * 4 {
            return; // too many quads
        }

        let (u0, v0, u1, v1) = (uv_rect[0], uv_rect[1], uv_rect[2], uv_rect[3]);
        let verts = [
            Vertex2d {
                position: [x, y],
                tex_coords: [u0, v0],
                color,
            },
            Vertex2d {
                position: [x, y + h],
                tex_coords: [u0, v1],
                color,
            },
            Vertex2d {
                position: [x + w, y + h],
                tex_coords: [u1, v1],
                color,
            },
            Vertex2d {
                position: [x + w, y],
                tex_coords: [u1, v0],
                color,
            },
        ];

        self.staging_vertices.extend_from_slice(&verts);
    }

    pub fn flush(&mut self, rpass: &mut wgpu::RenderPass<'_>, model_manager: &ModelManager) {
        let num_verts = self.staging_vertices.len();
        if num_verts == 0 {
            return;
        }

        model_manager.queue.write_buffer(
            &self.vertex_buffer.get(),
            0,
            bytemuck::cast_slice(&self.staging_vertices),
        );

        let pipeline = model_manager
            .materials
            .pipelines
            .render
            .get(&self.sprite2d_pipeline_key)
            .unwrap();
        rpass.set_pipeline(pipeline);
        rpass.set_bind_group(0, &self.ortho_bind_group, &[]);
        rpass.set_bind_group(1, &self.texture_bind_group, &[]);

        rpass.set_vertex_buffer(0, self.vertex_buffer.get().slice(..));
        rpass.set_index_buffer(self.index_buffer.get().slice(..), wgpu::IndexFormat::Uint32);

        let index_count = (num_verts as u32 / 4) * 6;
        rpass.draw_indexed(0..index_count, 0, 0..1);

        self.staging_vertices.clear();
    }
    pub fn draw_filled_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        let uv = [0.0, 0.0, 1.0, 1.0];
        self.push_quad(x, y, w, h, uv, color);
    }

    pub fn draw_image(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        let uv = [0.0, 0.0, 1.0, 1.0];
        self.push_quad(x, y, w, h, uv, color);
    }
}
