use std::sync::Arc;

use crate::{
    entity::EntityClass,
    game::{InputEvent, chunk::Chunk, entity::EntityId, lose_terrain::LoseTerrainId},
    render::{
        update_state::{
            chunk_meshs::update_chunk_meshs, entities::update_entities,
            handle_input::handle_user_input, lose_objects::update_lose_objects,
        },
        wgpu::RenderState,
    },
    utils::{Mesh, Vec3},
};

mod chunk_meshs;
mod entities;
mod handle_input;
mod lose_objects;

pub struct ChunkMeshUpdate {
    pub chunk_pos: (i32, i32, i32),
    pub transparent: bool,
    pub mesh: Option<Mesh>,
    pub mesh_l2: Option<Mesh>,
    pub mesh_l4: Option<Mesh>,
    pub mesh_l8: Option<Mesh>,
    pub data: Option<Arc<Chunk>>,
    pub lose_object_id: Option<LoseTerrainId>, //says if it is for world or lose object and if so which
}

pub struct EntityRenderDataUpdate {
    pub id: EntityId,
    pub position: Vec3,
    pub entity_class: EntityClass,
}

pub struct LoseObjectRenderDataUpdate {
    pub id: LoseTerrainId,
    pub position: Vec3,
}

pub fn update_render_state(render_state: &mut RenderState) {
    if true {
        handle_user_input(render_state);
        update_entities(render_state);
        update_lose_objects(render_state);
    }
    update_chunk_meshs(render_state);

    //update player pos
    let _ = render_state
        .render_channels
        .input_event_tx
        .send(InputEvent::CameraPositionUpdate(Vec3::new(
            render_state.camera.position.x,
            render_state.camera.position.y,
            render_state.camera.position.z,
        )));
}
