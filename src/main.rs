#![windows_subsystem = "windows"]
use crate::render::render_thread;

mod chunk_geneariton;
mod entity;
mod game;
mod mesh_creation;
pub mod mesh_creator;
mod physics;
pub mod render;
mod utils;

fn main() {
    env_logger::init();

    println!("running game");

    let _ = pollster::block_on(render_thread());
}

//cargo build --release --target x86_64-pc-windows-gnu
//cargo build --release --target x86_64-unknown-linux-musl
