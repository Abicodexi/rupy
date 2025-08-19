use crate::{
    gfx::bind_group::{
        global_uniform_layout, material_storage_layout, normal_texture_layout,
        skybox_cubemap_layout,
    },
    AssetRequest, CacheKey, Entity, Position, Renderable, Rotation, Scale, Texture, World,
    GROUND_Y,
};
use crossbeam::channel::Sender;
use wgpu::Device;

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
    world: &mut World,
    format: wgpu::TextureFormat,
    device: &Device,
) -> Entity {
    let depth_stencil = wgpu::DepthStencilState {
        format: Texture::DEPTH_FORMAT,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::LessEqual,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    };

    // === Goblin asset ===
    let goblin_obj = "goblin.obj";
    let _goblin_key = CacheKey::from(goblin_obj);
    asset_tx
        .send(AssetRequest::LoadModel {
            file: goblin_obj.to_string(),
            v_shader: "normal.vert.wgsl".to_string(),
            f_shader: "normal.frag.wgsl".to_string(),
            bind_group_layouts: vec![
                global_uniform_layout(device),
                skybox_cubemap_layout(device),
                material_storage_layout(device),
                normal_texture_layout(device),
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
            format,
            depth_stencil: Some(depth_stencil.clone()),
        })
        .ok();

    let bossman = world.spawn();
    let renderable = Renderable::new(vec![
        CacheKey::from("Cube.001_0"),
        CacheKey::from("BezierCircle_0"),
        CacheKey::from("Cylinder.008_0"),
        CacheKey::from("Cylinder.008_1"),
        CacheKey::from("Cylinder.003_0"),
        CacheKey::from("Cylinder.003_1"),
        CacheKey::from("Cylinder.005_0"),
        CacheKey::from("Cylinder.005_1"),
        CacheKey::from("Cylinder.007_0"),
        CacheKey::from("Cylinder.001_0"),
        CacheKey::from("Cylinder.004_0"),
        CacheKey::from("Cube.010_0"),
        CacheKey::from("Cube.008_0"),
        CacheKey::from("Cube.006_0"),
        CacheKey::from("Cube.015_0"),
        CacheKey::from("Cylinder.006_0"),
        CacheKey::from("Cylinder.016_0"),
        CacheKey::from("Cylinder.016_1"),
        CacheKey::from("Cylinder.002_0"),
        CacheKey::from("Cylinder.002_1"),
        CacheKey::from("Cylinder.014_0"),
        CacheKey::from("Cylinder.014_1"),
        CacheKey::from("Cube.020_0"),
        CacheKey::from("Cube.021_0"),
        CacheKey::from("Cube.022_0"),
        CacheKey::from("Cylinder.012_0"),
        CacheKey::from("Cylinder.012_1"),
        CacheKey::from("Cylinder.011_0"),
        CacheKey::from("Cube.019_0"),
    ]);
    world.insert_scale(bossman, Scale::new(2.0, 2.0, 2.0));
    world.insert_position(bossman, Position::new(4.5, 5.5, 5.0));
    world.insert_renderable(bossman, renderable);

    // === Cube wall objects ===
    let cube_obj = "cube.obj";
    let cube_key = CacheKey::from(cube_obj);
    asset_tx
        .send(AssetRequest::LoadModel {
            file: cube_obj.to_string(),
            v_shader: "normal.vert.wgsl".to_string(),
            f_shader: "normal.frag.wgsl".to_string(),
            bind_group_layouts: vec![
                global_uniform_layout(device),
                skybox_cubemap_layout(device),
                material_storage_layout(device),
                normal_texture_layout(device),
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
            format,
            depth_stencil: Some(depth_stencil),
        })
        .ok();

    // Use Position and Scale as struct types
    let mut cube_instances: Vec<(Position, Option<Rotation>, Option<Scale>)> = Vec::new();
    let scale = Some(Scale::new(0.5, 0.5, 0.5));
    let above_ground = GROUND_Y + 1.0;
    let size = 10;
    let wall_height = 15;
    let wall_y_offset = 0.0;

    // Floor
    for x in 0..(size + 20) {
        for z in 0..(size + 20) {
            cube_instances.push((
                Position::new(14.0 - x as f32, above_ground, z as f32),
                None,
                scale,
            ));
        }
    }

    // Ceiling
    for x in 0..size {
        for z in 0..size {
            cube_instances.push((
                Position::new(x as f32, (wall_height - 1) as f32 + above_ground, z as f32),
                None,
                scale,
            ));
        }
    }

    // Front and Back walls
    for x in 0..size {
        for y in 0..wall_height {
            cube_instances.push((
                Position::new(x as f32, y as f32 + wall_y_offset + above_ground, 0.0),
                None,
                scale,
            ));
        }
    }

    // Left & Right walls
    for z in 0..size {
        for y in 0..wall_height {
            cube_instances.push((
                Position::new(0.0, y as f32 + wall_y_offset + above_ground, z as f32),
                None,
                scale,
            ));
            cube_instances.push((
                Position::new(
                    (size - 1) as f32,
                    y as f32 + wall_y_offset + above_ground,
                    z as f32,
                ),
                None,
                scale,
            ));
        }
    }

    let wall_entity = world.spawn();
    let mut cube_renderable = Renderable::new(vec![cube_key]);

    // You must convert these into `Vec3` etc. in your renderer update loop
    cube_renderable.instances = cube_instances.into_iter().collect();

    world.insert_renderable(wall_entity, cube_renderable);

    wall_entity
}
