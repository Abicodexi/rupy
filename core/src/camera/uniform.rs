#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub inv_proj: [[f32; 4]; 4],
    pub inv_view: [[f32; 4]; 4],
    pub world_pos: [f32; 3], // 12 bytes
    pub _pad: f32,           // +4 bytes = 16-byte aligned
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
            inv_proj: cgmath::Matrix4::identity().into(),
            inv_view: cgmath::Matrix4::identity().into(),
            world_pos: [0.0; 3],
            _pad: 0.0,
        }
    }

    pub fn update(
        &mut self,
        vp: (
            cgmath::Matrix4<f32>,
            cgmath::Matrix4<f32>,
            cgmath::Matrix4<f32>,
        ),
        world_pos: cgmath::Vector3<f32>,
    ) {
        let (vp, inv_proj, inv_view) = vp;
        self.view_proj = vp.into();
        self.inv_proj = inv_proj.into();
        self.inv_view = inv_view.into();
        self.world_pos = [world_pos.x, world_pos.y, world_pos.z];
    }
}
