use crate::{BindGroupLayouts, ModelManager, Position, Scale, Vertex, VertexInstance, World};

pub enum ScreenCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

impl ScreenCorner {
    pub fn pos(&self, surface_width: u32, surface_height: u32, margin: f32) -> [f32; 2] {
        match self {
            ScreenCorner::TopLeft => [margin, margin],
            ScreenCorner::TopRight => [surface_width as f32 - margin, margin],
            ScreenCorner::BottomLeft => [margin, surface_height as f32 - margin],
            ScreenCorner::BottomRight => [
                surface_width as f32 - margin,
                surface_height as f32 - margin,
            ],
            ScreenCorner::Center => [surface_width as f32 * 0.5, surface_height as f32 * 0.5],
        }
    }
}

pub fn debug_scene(
    model_manager: &mut ModelManager,
    world: &mut World,
    surface_config: &wgpu::SurfaceConfiguration,
    depth_stencil: wgpu::DepthStencilState,
) {
    let size = 10;
    let wall_height = 15;
    let wall_y_offset = 0.0;
    if let Some(model_key) = World::load_object(
        model_manager,
        "cube.obj",
        "v_normal.wgsl",
        &[Vertex::LAYOUT, VertexInstance::LAYOUT],
        vec![
            BindGroupLayouts::uniform().clone(),
            BindGroupLayouts::equirect_dst().clone(),
            BindGroupLayouts::material_storage().clone(),
            BindGroupLayouts::normal().clone(),
        ],
        surface_config,
        wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Front),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        wgpu::ColorTargetState {
            format: surface_config.format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::all(),
        },
        Some(depth_stencil),
    ) {
        for x in 0..(size + 10) {
            for z in 0..(size + 10) {
                let entity = world.spawn();

                world.insert_scale(entity, Scale::new(0.5, 0.5, 0.5));
                world.insert_position(entity, Position::new(14.0 - x as f32, 0.0, z as f32));
                world.insert_renderable(entity, model_key.into());
            }
        }

        //  Ceiling at y = wall_height – 1
        for x in 0..size {
            for z in 0..size {
                let entity = world.spawn();

                world.insert_scale(entity, Scale::new(0.5, 0.5, 0.5));

                world.insert_position(
                    entity,
                    Position::new(x as f32, (wall_height - 1) as f32, z as f32),
                );
                world.insert_renderable(entity, model_key.into());
            }
        }

        // Front & Back walls (vary x & y, fix z)

        for x in 0..size {
            for y in 0..wall_height {
                // front wall at z = 0
                let e1 = world.spawn();
                world.insert_scale(e1, Scale::new(0.5, 0.5, 0.5));
                world.insert_position(e1, Position::new(x as f32, y as f32 + wall_y_offset, 0.0));
                world.insert_renderable(e1, model_key.into());
            }
        }

        //  Left & Right walls (vary z & y, fix x)
        for z in 0..size {
            for y in 0..wall_height {
                // left wall at x = 0
                let e1 = world.spawn();
                world.insert_scale(e1, Scale::new(0.5, 0.5, 0.5));
                world.insert_position(e1, Position::new(0.0, y as f32 + wall_y_offset, z as f32));
                world.insert_renderable(e1, model_key.into());

                // right wall at x = size - 1
                let e2 = world.spawn();
                world.insert_scale(e2, Scale::new(0.5, 0.5, 0.5));
                world.insert_position(
                    e2,
                    Position::new((size - 1) as f32, y as f32 + wall_y_offset, z as f32),
                );
                world.insert_renderable(e2, model_key.into());
            }
        }
    }
}
