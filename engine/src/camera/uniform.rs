use cgmath::Zero;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    inv_proj: [[f32; 4]; 4],
    inv_view: [[f32; 4]; 4],
    view_pos: [f32; 3],
    _pad: f32,
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
            inv_proj: cgmath::Matrix4::identity().into(),
            inv_view: cgmath::Matrix4::identity().into(),
            view_pos: cgmath::Vector3::zero().into(),
            _pad: 0.0,
        }
    }
    pub fn view_pos(&self) -> [f32; 3] {
        self.view_pos
    }
    pub fn update(
        &mut self,
        vp: (
            cgmath::Matrix4<f32>,
            cgmath::Matrix4<f32>,
            cgmath::Matrix4<f32>,
        ),
        view_pos: cgmath::Vector3<f32>,
    ) {
        let (vp, inv_proj, inv_view) = vp;
        self.view_proj = vp.into();
        self.inv_proj = inv_proj.into();
        self.inv_view = inv_view.into();
        self.view_pos = [view_pos.x, view_pos.y, view_pos.z];
    }
}
