use crossbeam::channel::Sender;

use crate::{
    AssetRequest, CacheKey, Entity, Position, RenderBindGroupLayouts, Scale, Texture, World,
    GROUND_Y,
};

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
    asset_tx: &Sender<AssetRequest>,
    bind_group_layouts: &RenderBindGroupLayouts,
    world: &mut World,
    format: wgpu::TextureFormat,
) -> Entity {
    let depth_stencil = wgpu::DepthStencilState {
        format: Texture::DEPTH_FORMAT,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::LessEqual,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    };
    let bossman = world.spawn();
    let goblin_obj = "goblin.obj";
    let goblin_key = CacheKey::from(goblin_obj);
    asset_tx
        .send(AssetRequest::LoadModel {
            file: goblin_obj.to_string(),
            v_shader: "normal.vert.wgsl".to_string(),
            f_shader: "normal.frag.wgsl".to_string(),
            bind_group_layouts: vec![
                bind_group_layouts.uniform().clone(),
                bind_group_layouts.equirect_dst().clone(),
                bind_group_layouts.material_storage().clone(),
                bind_group_layouts.normal().clone(),
            ],
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            color_target: wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            },
            depth_stencil: Some(depth_stencil.clone()),
        })
        .ok();

    world.insert_scale(bossman, Scale::new(10.0, 10.0, 10.0));
    world.insert_position(bossman, Position::new(4.5, 5.5, 5.0));
    world.insert_renderable(bossman, goblin_key.into());

    let size = 40;
    let wall_height = 15;
    let wall_y_offset = 0.0;
    let cube_obj = "cube.obj";
    let cube_key = CacheKey::from(cube_obj);
    asset_tx
        .send(AssetRequest::LoadModel {
            file: cube_obj.to_string(),
            v_shader: "normal.vert.wgsl".to_string(),
            f_shader: "normal.frag.wgsl".to_string(),
            bind_group_layouts: vec![
                bind_group_layouts.uniform().clone(),
                bind_group_layouts.equirect_dst().clone(),
                bind_group_layouts.material_storage().clone(),
                bind_group_layouts.normal().clone(),
            ],
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            color_target: wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            },
            depth_stencil: Some(depth_stencil.clone()),
        })
        .ok();

    let above_ground = GROUND_Y + 1.0;
    for x in 0..(size + 20) {
        for z in 0..(size + 20) {
            let entity = world.spawn();

            world.insert_scale(entity, Scale::new(0.5, 0.5, 0.5));
            world.insert_position(
                entity,
                Position::new(14.0 - x as f32, above_ground, z as f32),
            );
            world.insert_renderable(entity, cube_key.into());
        }
    }

    //  Ceiling
    for x in 0..size {
        for z in 0..size {
            let entity = world.spawn();

            world.insert_scale(entity, Scale::new(0.5, 0.5, 0.5));

            world.insert_position(
                entity,
                Position::new(x as f32, (wall_height - 1) as f32 + above_ground, z as f32),
            );
            world.insert_renderable(entity, cube_key.into());
        }
    }

    // Front & Back walls

    for x in 0..size {
        for y in 0..wall_height {
            let e1 = world.spawn();
            world.insert_scale(e1, Scale::new(0.5, 0.5, 0.5));
            world.insert_position(
                e1,
                Position::new(x as f32, y as f32 + wall_y_offset + above_ground, 0.0),
            );
            world.insert_renderable(e1, cube_key.into());
        }
    }

    //  Left & Right walls
    for z in 0..size {
        for y in 0..wall_height {
            // left wall
            let e1 = world.spawn();
            world.insert_scale(e1, Scale::new(0.5, 0.5, 0.5));
            world.insert_position(
                e1,
                Position::new(0.0, y as f32 + wall_y_offset + above_ground, z as f32),
            );
            world.insert_renderable(e1, cube_key.into());

            // right wall
            let e2 = world.spawn();
            world.insert_scale(e2, Scale::new(0.5, 0.5, 0.5));
            world.insert_position(
                e2,
                Position::new(
                    (size - 1) as f32,
                    y as f32 + wall_y_offset + above_ground,
                    z as f32,
                ),
            );
            world.insert_renderable(e2, cube_key.into());
        }
    }
    bossman
}
