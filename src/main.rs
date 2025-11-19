use std::thread;

use futures::executor::block_on;

use crate::{game::{InputEvent, game_thread}, render::{render_thread, types::{ChunkMeshUpdate, EntityRenderData}}, utils::{Vec3, raycast_test}};
use std::sync::mpsc::channel;

mod render;
mod game;
mod mesh_creation;
mod chunk_geneariton;
mod utils;
    
fn main() {
    env_logger::init();

    println!("running main game");


    let (chunk_mesh_update_tx, chunk_mesh_update_rx) = channel::<ChunkMeshUpdate>();
    let (entity_render_tx, entity_render_rx) = channel::<EntityRenderData>();
    let (input_event_tx, mut input_event_rx) = channel::<InputEvent>();

    //game loop thread start
    let _ = thread::spawn(move || {
        block_on(game_thread(chunk_mesh_update_tx, entity_render_tx, &mut input_event_rx));
    });

    let _ = pollster::block_on(render_thread(chunk_mesh_update_rx, entity_render_rx, input_event_tx));
}



//cargo build --target x86_64-pc-windows-gnu