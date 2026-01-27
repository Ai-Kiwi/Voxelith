use std::collections::HashMap;

use cgmath::{Quaternion, Vector3};
use wgpu::{Buffer, Device, Instance, util::DeviceExt};

use crate::{mesh_creator::MeshCreator, render::entity_meshs::{MESHID_TEST, MeshEntityLocationReference, MeshId, MeshInstance, MeshInstanceId, MeshInstanceRaw}};

pub struct InitEntityMeshs {
    pub mesh_id_reference : HashMap<MeshId,MeshEntityLocationReference>,
    pub meshs_buffer : Buffer,
    pub instances : HashMap<MeshId,MeshInstancesBufferInfo>,
    pub blank_instance_info : Buffer,
}

pub struct MeshInstancesBufferInfo {
    pub instances_buffer : Buffer,
    pub mesh_instances : HashMap<MeshInstanceId,MeshInstance>,
    pub instance_id_upto : u64,
    pub max_count : u64
}

pub fn create_new_mesh_instances(device : &Device, max_count : u64) -> MeshInstancesBufferInfo {
    let mesh_instances_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Meshs Instance Buffer"),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        size: max_count * size_of::<MeshInstanceRaw>() as u64,
        mapped_at_creation: false,
    });
    MeshInstancesBufferInfo { 
        instances_buffer: mesh_instances_buffer, 
        mesh_instances: HashMap::new(), 
        instance_id_upto: 0,
        max_count: max_count,
    }
}

impl InitEntityMeshs {
    pub fn new(device : &Device, queue : &wgpu::Queue) -> InitEntityMeshs {
        let mut contents: Vec<u8> = Vec::new();
        let mut mesh_id_reference: HashMap<MeshId,MeshEntityLocationReference> = HashMap::new();
        let mut mesh_instances: HashMap<MeshId,MeshInstancesBufferInfo> = HashMap::new();

        let vertices = MeshCreator::load_mesh_data_to_vertices(include_bytes!("../meshs/test.sevm")).unwrap();
        mesh_id_reference.insert(MESHID_TEST, MeshEntityLocationReference { start: contents.len() as u32, length: contents.len() as u32 + vertices.len() as u32});
        contents.extend_from_slice(bytemuck::cast_slice(&vertices));
        mesh_instances.insert(MESHID_TEST, create_new_mesh_instances(device, 500));



        let meshs_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Entity Meshs Buffer"),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(&contents),
        });
        
        //makes sure first instance is blank. This is so if no instance is picked it is just auto nothing
        
        let blank_instance = MeshInstance {
            position: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
        };
        let blank_instance_info = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Entity Meshs Buffer"),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            contents: bytemuck::bytes_of(&blank_instance.to_raw()),
        });
        

        InitEntityMeshs {
            //entity meshs
            mesh_id_reference,
            meshs_buffer,
            //mesh instances
            instances: mesh_instances,
            blank_instance_info,
        }
    }
}



//game loop will share render mesh update
//
//entity doing it
//