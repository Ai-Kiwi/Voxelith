use cgmath::Point3;

use crate::utils::Vec3;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &mut Camera, width : u32, height : u32) {
        self.view_proj = camera.build_view_projection_matrix(width as f32, height as f32).into();
    }
}

pub struct Camera {
    pub position : Vec3,
    pub pitch : f32,
    pub yaw : f32,
    pub target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    pub aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: 1.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 10000.0,
            position: Vec3::new(0.0, 0.0, 0.0),
            pitch: 0.0,
            yaw: 0.0,
        }
    }


    fn build_view_projection_matrix(&mut self, width : f32, height : f32) -> cgmath::Matrix4<f32> {
    let front = Vec3::new(
        self.yaw.cos() * self.pitch.cos(),
        self.pitch.sin(),
        self.yaw.sin() * self.pitch.cos()
    ).normalize();
    let new_target = self.position + front;
    self.target = Point3::new(new_target.x, new_target.y ,new_target.z);
    let position = Point3::new(self.position.x, self.position.y ,self.position.z);

    self.aspect = width / height;

    let view = cgmath::Matrix4::look_at_rh(position, self.target, self.up);
    let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

    return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}
