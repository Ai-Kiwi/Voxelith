#![windows_subsystem = "windows"]

use std::thread;

use futures::executor::block_on;

use crate::{game::{InputEvent, game_thread}, render::render_thread};
use std::sync::mpsc::channel;

pub mod render;
pub mod render_game;
mod game;
mod mesh_creation;
mod chunk_geneariton;
mod entity;
mod physics;
mod utils;
pub mod mesh_creator;
    
fn main() {
    env_logger::init();

    println!("running game");

    let _ = pollster::block_on(render_thread());
}



//cargo build --release --target x86_64-pc-windows-gnu
//cargo build --release --target x86_64-unknown-linux-musl