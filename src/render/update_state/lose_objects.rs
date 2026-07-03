use std::collections::HashMap;

use cgmath::Quaternion;
use dashmap::mapref::entry;

use crate::{entity::{self, EntityClass}, game::entity::EntityId, render::{entity_meshs::{MESHID_TEST, MeshId, MeshInstance, MeshInstanceId, MeshInstanceRaw}, wgpu::{ChunkListInfo, LoseObjectInfo, RenderState}}, utils::Vec3};
use crate::render::wgpu::EntityRenderData;

//when removing is added it should also remove the instance data.
pub fn update_lose_objects(render_state : &mut RenderState) {
    loop {
        let lose_objects_update = render_state.render_channels.lose_object_update_rx.try_recv();
        match lose_objects_update {
            Ok(object_update) => {
                let mut opt_object_loc: Option<usize> = render_state.lose_objects_loc.get(&object_update.id).copied();
                if opt_object_loc.is_none() {
                    //not stored so make new
                    let lose_object = LoseObjectInfo {
                        position: object_update.position,
                        id: object_update.id,
                        chunks: ChunkListInfo {
                            chunk_meshs: Vec::new(),
                            chunk_meshs_loc: HashMap::new(),
                        },
                    };
                    let loc = render_state.lose_objects.len();
                    render_state.lose_objects_loc.insert(object_update.id, loc);
                    render_state.lose_objects.push(lose_object);
                    opt_object_loc = Some(loc);
                };
                let object = render_state.lose_objects.get_mut(opt_object_loc.unwrap()).unwrap();
                object.id = object_update.id;
                object.position = object_update.position;
            },
            Err(_) => {
                break
            },
        }
    }
}