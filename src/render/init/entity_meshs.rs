use std::collections::HashMap;

use wgpu::{Buffer, Device, util::DeviceExt};

use crate::mesh_creator::MeshCreator;

pub struct InitEntityMeshs {
    buffer : Buffer
}

pub struct MeshEntityLocationReference {
    start : usize,
    length : usize,
}



impl InitEntityMeshs {
    pub fn new(device : &Device) -> InitEntityMeshs {
        let contents: Vec<u8> = Vec::new();
        let mut mesh_id_reference: HashMap<u64,MeshEntityLocationReference> = HashMap::new();

        //let vertices = MeshCreator::load_mesh_data_to_vertices(include_bytes!("../meshs/mesh.sevm")).unwrap();
        //mesh_id_reference.insert(1, MeshEntityLocationReference { start: contents.len(), length: vertices.len()});
        

        



        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Entity Meshs Buffer"),
            usage: wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&contents),
        });
        InitEntityMeshs {
            buffer: buffer,
        }
    }
}



//game loop will share render mesh update
//
//entity doing it
//