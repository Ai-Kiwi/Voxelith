use std::collections::HashMap;

use wgpu::{Buffer, Device, Instance, util::DeviceExt};

use crate::{mesh_creator::MeshCreator, render::entity_meshs::{MESHID_TEST, MeshEntityLocationReference, MeshId, MeshInstance, MeshInstanceId, MeshInstanceRaw}};

pub struct InitEntityMeshs {
    pub mesh_id_reference : HashMap<MeshId,MeshEntityLocationReference>,
    pub meshs_buffer : Buffer,
    pub mesh_instances_buffer : Buffer,
    pub free_mesh_instances : Vec<MeshInstanceId>,
    pub mesh_instances : HashMap<MeshInstanceId,MeshInstance>,
}

//instance id ->
//mesh id -> 


impl InitEntityMeshs {
    pub fn new(device : &Device) -> InitEntityMeshs {
        let mut contents: Vec<u8> = Vec::new();
        let mut mesh_id_reference: HashMap<MeshId,MeshEntityLocationReference> = HashMap::new();

        let vertices = MeshCreator::load_mesh_data_to_vertices(include_bytes!("../meshs/test.sevm")).unwrap();
        mesh_id_reference.insert(MESHID_TEST, MeshEntityLocationReference { start: contents.len() as u32, length: contents.len() as u32 + vertices.len() as u32});
        contents.extend_from_slice(bytemuck::cast_slice(&vertices));




        let meshs_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Entity Meshs Buffer"),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(&contents),
        });

        const MESH_INSTANCES_SIZE: u64 = 1048576;

        let mesh_instances_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Meshs Instance Buffer"),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            size: MESH_INSTANCES_SIZE,
            mapped_at_creation: false,
        });

        
        let size: u64 = MESH_INSTANCES_SIZE / size_of::<MeshInstanceRaw>() as u64;
        let free_mesh_instances: Vec<MeshInstanceId> = (0..size).map(|x| {MeshInstanceId(x)}).collect();
        let mesh_instances : HashMap<MeshInstanceId,MeshInstance> = HashMap::new();

        InitEntityMeshs {
            //entity meshs
            mesh_id_reference,
            meshs_buffer,
            //mesh instances
            mesh_instances_buffer,
            free_mesh_instances,
            mesh_instances,
        }
    }
}



//game loop will share render mesh update
//
//entity doing it
//