use cgmath::Quaternion;
use wgpu::wgc::instance;

use crate::{entity::EntityClass, game::entity::EntityId, render::{self, entity_meshs::{MESHID_TEST, MeshId, MeshInstance, MeshInstanceId, MeshInstanceRaw}, wgpu::RenderState}, render_game::GameData, utils::Vec3};

pub struct EntityRenderData {
    pub id : EntityId,
    pub position : Vec3,
    pub entity_class : EntityClass,
    pub render_mesh_id : MeshId, //THIS IS VERY TEMP, WILL CHANGE TO REAL SYSTEM LATER BUT FOR NOW WILL WORK
    pub instance_id : MeshInstanceId,
}

pub struct EntityRenderDataUpdate {
    pub id : EntityId,
    pub position : Vec3,
    pub entity_class : EntityClass,
}



pub fn update_entities(render_state : &mut RenderState, game_data : &mut GameData) {
    loop {
        let entity_render_update = game_data.render_channels.entity_render_rx.try_recv();
        match entity_render_update {
            Ok(entity_update) => {
                if game_data.entities.contains_key(&entity_update.id) == false {
                    let mesh_id = MESHID_TEST;
                    let instances_buffer = render_state.mesh_instances.get_mut(&mesh_id).unwrap();
                    let new_instance = MeshInstanceId(instances_buffer.instance_id_upto);
                    instances_buffer.instance_id_upto += 1;
                    let new_entity_instance = MeshInstance {
                        position: (0.0,0.0,0.0).into(),
                        rotation: Quaternion::new(0.0, 0.0, 0.0, 0.0),
                    };
                    game_data.entities.insert(entity_update.id,EntityRenderData {
                        id: entity_update.id,
                        position: Vec3::new(0.0, 0.0, 0.0),
                        entity_class: EntityClass::Player,
                        render_mesh_id: mesh_id,
                        instance_id: new_instance,
                    });
                    render_state.queue.write_buffer(&instances_buffer.instances_buffer, new_instance.0 * (size_of::<MeshInstanceRaw>() as u64), bytemuck::bytes_of(&new_entity_instance.to_raw()));
                    instances_buffer.mesh_instances.insert(new_instance,new_entity_instance);
                    println!("created new entity data")
                }
                let entity_render_data = game_data.entities.get_mut(&entity_update.id).unwrap();
                entity_render_data.id = entity_update.id;
                entity_render_data.position = entity_update.position;
                entity_render_data.entity_class = entity_update.entity_class;
                //update instance
                let instance_buffer = render_state.mesh_instances.get_mut(&entity_render_data.render_mesh_id).unwrap();
                let instance = instance_buffer.mesh_instances.get_mut(&entity_render_data.instance_id).unwrap();
                instance.position = (entity_render_data.position.x, entity_render_data.position.y, entity_render_data.position.z).into();
                render_state.queue.write_buffer(&instance_buffer.instances_buffer, &entity_render_data.instance_id.0 * (size_of::<MeshInstanceRaw>() as u64), bytemuck::bytes_of(&instance.to_raw()));
            },
            Err(_) => {
                break
            },
        }
    }
}