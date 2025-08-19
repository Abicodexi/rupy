pub mod layout;
pub mod builder;
pub mod manager;

pub use layout::{
    camera_layout,
    debug_layout,
    diffuse_layout,
    global_uniform_layout,
    light_layout,
    material_storage_layout,
    normal_texture_layout,
    ortho_uniform_layout,
    skybox_cubemap_layout,
    skybox_projection_input_layout,
    sprite_2d_array_layout,
};

pub use builder::{
    camera_group,
    light_group,
    global_uniform_group,
    ortho_uniform_group,
    debug_group,
    diffuse_group,
    normal_textures_group,
    sprite_2d_array_group,
    skybox_cubemap_group,
    skybox_projection_input_group,
    material_storage_group,
    hdr_group,
};

pub use manager::{BindGroupManager};
