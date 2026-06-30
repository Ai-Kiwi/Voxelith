
use std::sync::Arc;

use crate::{entity::EntityClass, game::{InputEvent, chunk::Chunk, entity::EntityId}, render::{update_state::{chunk_meshs::update_chunk_meshs, entities::update_entities, handle_input::handle_user_input}, wgpu::RenderState}, utils::{Mesh, Vec3}};

mod handle_input;
mod chunk_meshs;
mod entities;

pub struct ChunkMeshUpdate {
    pub chunk_pos : (i32,i32,i32),
    pub transparent : bool,
    pub mesh : Option<Mesh>,
    pub mesh_l2 : Option<Mesh>,
    pub mesh_l4 : Option<Mesh>,
    pub mesh_l8 : Option<Mesh>,
    pub data : Option<Arc<Chunk>>,
}

pub struct EntityRenderDataUpdate {
    pub id : EntityId,
    pub position : Vec3,
    pub entity_class : EntityClass,
}

pub fn update_render_state(render_state : &mut RenderState) {
    if true {
        handle_user_input(render_state);
        update_entities(render_state);
    }
    update_chunk_meshs(render_state);

    //update player pos
    let _ = render_state.render_channels.input_event_tx.send(InputEvent::CameraPositionUpdate(Vec3::new(
        render_state.camera.position.x, 
        render_state.camera.position.y, 
        render_state.camera.position.z
    )));
}
