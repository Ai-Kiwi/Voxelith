use std::thread;

use futures::executor::block_on;

use crate::{game::{InputEvent, game_thread}, render::{render_thread, types::{ChunkMeshUpdate, EntityRenderData}}, utils::{Vec3, raycast_test}};
use std::sync::mpsc::channel;

mod render;
mod game;
mod mesh_creation;
mod chunk_geneariton;
mod utils;
mod entity;
mod physics;
mod lighting_updates;
    
fn main() {
    env_logger::init();

    println!("running game");

    let _ = pollster::block_on(render_thread());
}



//cargo build --target x86_64-pc-windows-gnu