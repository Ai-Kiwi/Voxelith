use std::{collections::{BTreeMap, HashMap}, sync::{Arc, mpsc::{Receiver, Sender}}};

use crate::{game::{InputEvent, entity::EntityId}, render::{camera::PerspectiveCamera, mesh::GpuMeshReference, wgpu::RenderState}, render_game::{chunk::{ChunkMeshUpdate, update_chunk_meshs}, entities::{EntityRenderData, EntityRenderDataUpdate, update_entities}, handle_input::handle_user_input}, utils::Vec3};


pub const LEVEL_1_LOD_DISTANCE: f32 = 480.0;
pub const LEVEL_2_LOD_DISTANCE: f32 = LEVEL_1_LOD_DISTANCE * 2.0;
pub const LEVEL_3_LOD_DISTANCE: f32 = LEVEL_2_LOD_DISTANCE * 2.0;
pub const LEVEL_4_LOD_DISTANCE: f32 = LEVEL_3_LOD_DISTANCE * 2.0;

pub mod chunk;
pub mod entities;
mod handle_input;
pub mod render_frame;

pub struct RenderThreadChannels {
    pub chunk_mesh_update_rx : Receiver<ChunkMeshUpdate>, 
    pub entity_render_rx : Receiver<EntityRenderDataUpdate>, 
    pub input_event_tx: Sender<InputEvent>,
}

pub struct ChunkInfo {
    pointer : Arc<GpuMeshReference>,
    buffer_number : usize,
    size : usize
}

pub struct GameData {
    pub camera : PerspectiveCamera,
    pub chunk_meshs : HashMap<(i32,i32,i32,bool),ChunkInfo>,
    pub cache_chunk_meshs : BTreeMap<(i32,i32,i32,bool),ChunkInfo>,
    pub chunk_mesh_data : HashMap<(i32,i32,i32,bool),ChunkMeshUpdate>,
    pub render_channels : RenderThreadChannels,
    pub entities: HashMap<EntityId,EntityRenderData>,
}

pub fn tick_game_render_logic(render_state : &mut RenderState, game_data : &mut GameData, open : bool) {
    if open {
        handle_user_input(render_state, game_data);
        update_entities(render_state, game_data);
    }
    update_chunk_meshs(render_state, game_data);

    //update player pos
    let _ = game_data.render_channels.input_event_tx.send(InputEvent::CameraPositionUpdate(Vec3::new(
        game_data.camera.position.x, 
        game_data.camera.position.y, 
        game_data.camera.position.z
    )));
}