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
    pub position: [f32; 3],
    _padding: f32,
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
            position: [0.0,0.0,0.0],
            _padding: 0.0,
        }
    }

    pub fn update_view_proj_prespec(&mut self, camera: &mut PerspectiveCamera, width : u32, height : u32) {
        self.view_proj = camera.build_view_projection_matrix(width as f32, height as f32).into();
        self.position = [camera.position.x, camera.position.y, camera.position.z]
    }

    pub fn update_view_proj_ortho(&mut self, camera: &mut OrthographicCamera) {
        self.view_proj = camera.build_view_projection_matrix().into();
        self.position = [camera.position.x, camera.position.y, camera.position.z]
    }
}

pub struct PerspectiveCamera {
    pub position : Vec3,
    pub pitch : f32,
    pub yaw : f32,
    pub target: cgmath::Point3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    znear: f32,
    zfar: f32,
}

impl PerspectiveCamera {
    pub fn new() -> PerspectiveCamera {
        PerspectiveCamera {
            target: (0.0, 0.0, 0.0).into(),
            aspect: 1.0,
            fovy: 45.0,
            znear: 1.0,
            zfar: 10000.0,
            position: Vec3::new(0.0, 0.0, 0.0),
            pitch: 0.0,
            yaw: 0.0,
        }
    }


    fn build_view_projection_matrix(&mut self, width : f32, height : f32) -> cgmath::Matrix4<f32> {
        let front: Vec3 = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos()
        ).normalize();
        let new_target = self.position + front;
        self.target = Point3::new(new_target.x, new_target.y ,new_target.z);
        let position = Point3::new(self.position.x, self.position.y ,self.position.z);

        self.aspect = width / height;

        let view = cgmath::Matrix4::look_at_rh(position, self.target, cgmath::Vector3::unit_y());
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

pub struct OrthographicCamera {
    pub position : Vec3,
    pub target: cgmath::Point3<f32>,
    pub width: f32,
    pub height : f32,
    pub znear: f32,
    pub zfar: f32,
}

impl OrthographicCamera {
    pub fn new() -> OrthographicCamera {
        OrthographicCamera {
            target: (0.0, 0.0, 0.0).into(),
            znear: 0.1,
            zfar: 1000.0,
            position: Vec3::new(0.0, 0.0, 0.0),
            width: 1024.0,
            height: 1024.0,
        }
    }


    fn build_view_projection_matrix(&mut self) -> cgmath::Matrix4<f32> {
        let position = Point3::new(self.position.x, self.position.y ,self.position.z);

        let view = cgmath::Matrix4::look_at_rh(position, self.target, cgmath::Vector3::unit_y());
        let proj = cgmath::ortho(-(self.width/2.0),self.width/2.0,-(self.height/2.0),self.height/2.0,self.znear,self.zfar);
        
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

