use std::{collections::HashMap, sync::{Arc, mpsc::{Receiver, Sender, channel}}, thread, time::{Duration, Instant}};
use futures::executor::block_on;
use rayon::vec;
use wgpu::Instance;

use crate::{chunk_geneariton::{NewChunkInfo, chunk_generation_thread}, game::{chunk::handle_chunk_loaded, entity::handle_entity_update, handle_inputs::handle_user_inputs, mesh_updates::handle_chunk_mesh_updates, pixel_updates::handle_pixel_updates, world::WorldData}, mesh_creation::{ChunkMeshCreateRequest, chunk_mesh_creation_thread}, render::types::{ChunkMeshUpdate, EntityRenderData}, utils::{Vec2, Vec3, raycast_test}};

pub mod world;
pub mod chunk;
mod pixel_updates;
mod mesh_updates;
mod handle_inputs;
mod entity;
pub mod pixel;

pub enum InputEvent {
    Move(Vec2),
    Jump,
    Click(Vec3,Vec3),
    CameraPositionUpdate(Vec3)
    //Attack,
    //OpenInventory,
    //UseItem(u32), // item ID
    //DropItem(u32),
    // ... other game actions
}

pub const MAX_CHUNK_LOAD_DISTANCE: i32 = 100;


pub async fn game_thread(chunk_mesh_update_tx : Sender<ChunkMeshUpdate>, entity_render_tx : Sender<EntityRenderData>, input_event_rx : &mut Receiver<InputEvent>) {
    let _ = entity_render_tx;
    let mut world: WorldData = WorldData {
        chunks: HashMap::new(),
        //entities: HashMap::new(),
        //entities_count: 0,
        pixel_edit_queue: Vec::new(),
        chunk_mesh_updates_needed: HashMap::new(),
        chunks_loading: HashMap::new(),
    };
    let mut player_position = Vec3::new(0.0, 0.0, 0.0);


    let (chunk_generation_request_tx, mut chunk_generation_request_rx) = channel::<(i32,i32,i32)>();
    let (chunk_generated_tx, chunk_generated_rx) = channel::<NewChunkInfo>();
    let (request_chunk_mesh_update_tx, mut request_chunk_mesh_update_rx) = channel::<ChunkMeshCreateRequest>();

    //chunk generation thread start
    let _ = thread::spawn(move || {
        block_on(chunk_generation_thread(&mut chunk_generation_request_rx, chunk_generated_tx));
    });
    //chunk mesh creation thread start
    let _ = thread::spawn(move || {
        block_on(chunk_mesh_creation_thread(chunk_mesh_update_tx, &mut request_chunk_mesh_update_rx));
    });

    let mut last_tick_time = Instant::now();

    println!("starting game loop");
    loop {
        handle_chunk_loaded(&mut world, &chunk_generated_rx, &player_position, &chunk_generation_request_tx);

        handle_user_inputs(&mut world, &mut player_position, input_event_rx);

        handle_pixel_updates(&mut world);

        handle_chunk_mesh_updates(&mut world, &request_chunk_mesh_update_tx);


        //physics loop
        //tick_physics(&mut world);

        handle_entity_update();

        //60tps
        thread::sleep(Duration::from_millis(((1.0 / 60.0) - last_tick_time.elapsed().as_secs_f32()) as u64));
        last_tick_time = Instant::now();
    }
}