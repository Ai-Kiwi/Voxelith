use std::{collections::HashMap, sync::{Arc, mpsc::{Receiver, Sender, channel}}, thread, time::{Duration, Instant}};
use futures::executor::block_on;
use egui::{CentralPanel};
use crate::{chunk_geneariton::{NewChunkInfo, chunk_generation_thread}, entity::Entity, game::{chunk::{Chunk, handle_chunk_loaded}, entity::{Entities, handle_entity_update}, handle_inputs::handle_user_inputs, mesh_updates::handle_chunk_mesh_updates, pixel_updates::handle_pixel_updates, world::WorldData}, lighting_updates::{self, lighting_update_thread}, mesh_creation::{ChunkMeshCreateRequest, chunk_mesh_creation_thread}, render::types::{ChunkMeshUpdate, EntityRenderData}, utils::{Vec2, Vec3, raycast_test}};

pub mod world;
pub mod chunk;
mod pixel_updates;
mod mesh_updates;
mod handle_inputs;
pub mod entity;
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

pub struct Game {
    world : WorldData,
    entities : Entities,
}

//snapshot system
pub struct GameSnapshot {
    pub chunks : HashMap<(i32, i32,i32), Arc<Chunk>>,
}

impl Game {
    pub fn create_snapshot(&self) -> Arc<GameSnapshot> {
        let snapshot = GameSnapshot {
            chunks: self.world.chunks.clone(),
        };
        return Arc::new(snapshot)
    }
}




pub async fn game_thread(chunk_mesh_update_tx : Sender<ChunkMeshUpdate>, entity_render_tx : Sender<EntityRenderData>, input_event_rx : &mut Receiver<InputEvent>) {
    println!("starting main game loop");
    let _ = entity_render_tx;
    let mut game = Game {
        world : WorldData {
            chunks: HashMap::new(),
            //entities: HashMap::new(),
            //entities_count: 0,
            pixel_edit_queue: Vec::new(),
            chunk_mesh_updates_needed: HashMap::new(),
            chunks_loading: HashMap::new(),
            chunk_shadows: HashMap::new(),
        },
        entities: Entities {
            entities: HashMap::new(),
            entities_count: 0,
        }
    };
    let mut player_position = Vec3::new(0.0, 0.0, 0.0);

    let (chunk_generation_request_tx, mut chunk_generation_request_rx) = channel::<(i32,i32,i32)>();
    let (chunk_generated_tx, chunk_generated_rx) = channel::<NewChunkInfo>();
    let (request_chunk_mesh_update_tx, mut request_chunk_mesh_update_rx) = channel::<ChunkMeshCreateRequest>();
    let (game_snapshot_update_tx, mut game_snapshot_update_rx) =  channel::<Arc<GameSnapshot>>();


    //chunk generation thread start
    let _ = thread::spawn(move || {
        block_on(chunk_generation_thread(&mut chunk_generation_request_rx, chunk_generated_tx));
    });
    //chunk mesh creation thread start
    let _ = thread::spawn(move || {
        block_on(chunk_mesh_creation_thread(chunk_mesh_update_tx, &mut request_chunk_mesh_update_rx));
    });
    //lighting update thread start
    let _ = thread::spawn(move || {
        block_on(lighting_update_thread(&mut game_snapshot_update_rx));
    });

    let mut last_tick_time = Instant::now();

    println!("starting game loop");
    'game_loop : loop {
        handle_chunk_loaded(&mut game.world, &chunk_generated_rx, &player_position, &chunk_generation_request_tx);

        handle_user_inputs(&mut game.world, &mut player_position, input_event_rx);

        handle_pixel_updates(&mut game.world);

        handle_chunk_mesh_updates(&mut game.world, &request_chunk_mesh_update_tx);


        //physics loop
        //tick_physics(&mut world);

        handle_entity_update();

        let snapshot: Arc<GameSnapshot> = game.create_snapshot();
        match game_snapshot_update_tx.send(snapshot) {
            Ok(_) => (),
            Err(_) => break 'game_loop, //game must have stopped so close loop
        }

        //60tps
        let sleep_time = Duration::from_millis(((1.0 / 60.0) - last_tick_time.elapsed().as_secs_f32()) as u64);
        if sleep_time < Duration::from_secs(0) {
            println!("main game loop is lagging. Took more then needed time")
        }
        thread::sleep(sleep_time);
        last_tick_time = Instant::now();
    }

    println!("close main game loop")
    //start unloading data. This means doing things like saving all the chunks and entities in the world
}