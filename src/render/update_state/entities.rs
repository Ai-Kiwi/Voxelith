use cgmath::Quaternion;
use dashmap::mapref::entry;

use crate::{entity::{self, EntityClass}, game::entity::EntityId, render::{entity_meshs::{MESHID_TEST, MeshId, MeshInstance, MeshInstanceId, MeshInstanceRaw}, wgpu::RenderState}, utils::Vec3};
use crate::render::wgpu::EntityRenderData;

//when removing is added it should also remove the instance data.
pub fn update_entities(render_state : &mut RenderState) {
    loop {
        let entity_render_update = render_state.render_channels.entity_render_rx.try_recv();
        match entity_render_update {
            Ok(entity_update) => {
                let mut opt_entity_loc: Option<usize> = render_state.entities_loc.get(&entity_update.id).copied();
                if opt_entity_loc.is_none() {
                    //not stored so make new
                    let mesh_id = MESHID_TEST;
                    let instances_buffer = render_state.mesh_instances.get_mut(&mesh_id).unwrap();
                    let new_instance = MeshInstanceId(instances_buffer.instance_id_upto);
                    instances_buffer.instance_id_upto += 1;
                    let new_entity_instance = MeshInstance {
                        position: (0.0,0.0,0.0).into(),
                        rotation: Quaternion::new(0.0, 0.0, 0.0, 0.0),
                    };
                    render_state.queue.write_buffer(&instances_buffer.instances_buffer, new_instance.0 * (size_of::<MeshInstanceRaw>() as u64), bytemuck::bytes_of(&new_entity_instance.to_raw()));

                    instances_buffer.mesh_instances.insert(new_instance,new_entity_instance);

                    let entity_obj = EntityRenderData {
                        id: entity_update.id,
                        position: Vec3::new(0.0, 0.0, 0.0),
                        entity_class: EntityClass::Player,
                        render_mesh_id: mesh_id,
                        instance_id: new_instance,
                    };
                    let loc = render_state.entities.len();
                    render_state.entities_loc.insert(entity_update.id, loc);
                    render_state.entities.push(entity_obj);
                    opt_entity_loc = Some(loc);
                };
                let entity = render_state.entities.get_mut(opt_entity_loc.unwrap()).unwrap();
                entity.id = entity_update.id;
                entity.position = entity_update.position;
                entity.entity_class = entity_update.entity_class;
                //update instance
                let instance_buffer = render_state.mesh_instances.get_mut(&entity.render_mesh_id).unwrap();
                let instance = instance_buffer.mesh_instances.get_mut(&entity.instance_id).unwrap();
                instance.position = (entity.position.x, entity.position.y, entity.position.z).into();
                render_state.queue.write_buffer(&instance_buffer.instances_buffer, &entity.instance_id.0 * (size_of::<MeshInstanceRaw>() as u64), bytemuck::bytes_of(&instance.to_raw()));
            },
            Err(_) => {
                break
            },
        }
    }
}