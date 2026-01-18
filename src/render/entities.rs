struct EntityInstance {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
}

impl EntityInstance {
    fn to_raw(&self) -> EntityInstanceRaw {
        EntityInstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation)).into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct EntityInstanceRaw {
    model: [[f32; 4]; 4],
}