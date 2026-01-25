#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct MeshId(pub u32);

pub const MESHID_TEST: MeshId = MeshId(1);



pub struct MeshEntityLocationReference {
    pub start : u32,
    pub length : u32,
}

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub struct MeshInstanceId (pub u64);

pub struct MeshInstance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

impl MeshInstance {
    pub fn to_raw(&self) -> MeshInstanceRaw {
        MeshInstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation)).into(),
        }
    }
}


#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshInstanceRaw {
    model: [[f32; 4]; 4],
}