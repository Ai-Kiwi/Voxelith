use cgmath::{Point3, SquareMatrix};

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
    pub view_proj: [[f32; 4]; 4],
    pub view_proj_inverse: [[f32; 4]; 4],
    pub position: [f32; 3],
    _padding: f32,
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
            view_proj_inverse: cgmath::Matrix4::identity().into(),
            position: [0.0,0.0,0.0],
            _padding: 0.0,
        }
    }

    pub fn update_view_proj_prespec(&mut self, camera: &mut PerspectiveCamera, width : u32, height : u32) {
        let front: Vec3 = Vec3::new(
            camera.yaw.cos() * camera.pitch.cos(),
            camera.pitch.sin(),
            camera.yaw.sin() * camera.pitch.cos()
        ).normalize();
        let new_target = camera.position + front;
        camera.target = Point3::new(new_target.x, new_target.y ,new_target.z);
        let position = Point3::new(camera.position.x, camera.position.y ,camera.position.z);

        camera.aspect = (width as f32) / (height as f32);

        let view = cgmath::Matrix4::look_at_rh(position, camera.target, cgmath::Vector3::unit_y());
        let proj = cgmath::perspective(cgmath::Deg(camera.fovy), camera.aspect, camera.znear, camera.zfar);

        self.view_proj_inverse = (OPENGL_TO_WGPU_MATRIX * proj * view).invert().unwrap().into();
        self.view_proj = (OPENGL_TO_WGPU_MATRIX * proj * view).into();
        self.position = [camera.position.x, camera.position.y, camera.position.z]
    }

    pub fn update_view_proj_ortho(&mut self, camera: &mut OrthographicCamera) {
        let position = Point3::new(camera.position.x, camera.position.y ,camera.position.z);

        let view = cgmath::Matrix4::look_at_rh(position, camera.target, cgmath::Vector3::unit_y());
        let proj = cgmath::ortho(-(camera.width/2.0),camera.width/2.0,-(camera.height/2.0),camera.height/2.0,camera.znear,camera.zfar);
        self.view_proj_inverse = (OPENGL_TO_WGPU_MATRIX * proj * view).invert().unwrap().into();
        self.view_proj = (OPENGL_TO_WGPU_MATRIX * proj * view).into();        
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
}

