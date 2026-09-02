use crate::{
    chunk_geneariton::{NewChunkInfo, chunk_generation_thread},
    entity::{Entity, EntityRenderComponent},
    game::{
        chunk::{Chunk, handle_chunk_loaded},
        entity::{Entities, EntityId, handle_entity_update},
        handle_inputs::handle_user_inputs,
        lose_terrain::{LoseObject, LoseTerrain, LoseTerrainId, handle_lose_object_update},
        mesh_updates::handle_chunk_mesh_updates,
        pixel::PixelTypes,
        pixel_updates::handle_pixel_updates,
        world::WorldData,
    },
    mesh_creation::{ChunkMeshCreateRequest, chunk_mesh_creation_thread},
    physics::{PhysicsObject, tick_physics},
    render::update_state::{ChunkMeshUpdate, EntityRenderDataUpdate, LoseObjectRenderDataUpdate},
    utils::{Vec2, Vec3},
};
use futures::executor::block_on;
use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc,
        mpsc::{Receiver, Sender, channel},
    },
    thread,
    time::{Duration, Instant, SystemTime},
};

pub mod chunk;
pub mod entity;
mod handle_inputs;
pub mod lose_terrain;
mod mesh_updates;
pub mod pixel;
mod pixel_updates;
pub mod world;

pub enum InputEvent {
    Move(Vec2),
    Jump,
    Click(Vec3, Vec3),
    PlaceClick(Vec3, Vec3),
    CameraPositionUpdate(Vec3), //Attack,
                                //OpenInventory,
                                //UseItem(u32), // item ID
                                //DropItem(u32),
                                // ... other game actions
}

pub const MAX_CHUNK_LOAD_DISTANCE: i32 = 70;
pub const MIN_CHUNK_UNLOAD_DISTANCE: i32 = 75;

pub struct Game {
    pub world: WorldData,
    pub entities: Entities,
    pub lose_terrain: LoseTerrain,
}

pub async fn game_thread(
    chunk_mesh_update_tx: Sender<ChunkMeshUpdate>,
    entity_render_tx: Sender<EntityRenderDataUpdate>,
    input_event_rx: &mut Receiver<InputEvent>,
    lose_object_update_tx: Sender<LoseObjectRenderDataUpdate>,
) {
    println!("starting main game loop");
    let mut game = Game {
        world: WorldData {
            chunks: HashMap::new(),
            pixel_edit_queue: Vec::new(),
            chunk_mesh_updates_needed: HashMap::new(),
            chunks_loading: HashMap::new(),
            chunk_shadows: HashMap::new(),
        },
        entities: Entities {
            entities: HashMap::new(),
            entities_count: 0,
            updated: HashMap::new(),
        },
        lose_terrain: LoseTerrain {
            objects: HashMap::new(),
            object_count: 0,
            position_updated: HashSet::new(),
            mesh_updated: HashSet::new(),
        },
    };
    let mut player_position = Vec3::new(0.0, 0.0, 0.0);

    let (chunk_generation_request_tx, mut chunk_generation_request_rx) =
        channel::<(i32, i32, i32)>();
    let (chunk_generated_tx, chunk_generated_rx) = channel::<NewChunkInfo>();
    let (request_chunk_mesh_update_tx, mut request_chunk_mesh_update_rx) =
        channel::<ChunkMeshCreateRequest>();

    //example entries.
    for i in 0..5 {
        game.entities.entities.insert(
            EntityId(i),
            Entity {
                id: EntityId(i),
                position: Vec3::new(0.0, 0.0, 0.0),
                physics: PhysicsObject {
                    hitbox: Vec3::new(2.5, 8.5, 2.5),
                    moveable: true,
                    velocity: Vec3::new(0.0, 0.0, 0.0),
                    gravity: true,
                    grounded: false,
                },
                entity_class: crate::entity::EntityClass::Player,
                render_component: Some(EntityRenderComponent {
                    entity_meshs: Vec::new(),
                }),
            },
        );
        game.entities.updated.insert(EntityId(i), ());
    }

    game.entities.entities_count = 1;

    //example lose objects
    let mut lose_chunk_data: Vec<Arc<Chunk>> = Vec::new();
    let mut chunk = Chunk {
        solid_items: 0,
        data: [PixelTypes::Air; 16 * 16 * 16],
    };
    chunk.set_relative_pixel(5, 5, 5, PixelTypes::Stone);
    chunk.set_relative_pixel(5, 6, 5, PixelTypes::Stone);
    chunk.set_relative_pixel(5, 5, 6, PixelTypes::Stone);

    lose_chunk_data.push(Arc::new(chunk));

    let mut lose_chunk_data_loc = HashMap::new();
    lose_chunk_data_loc.insert((0, 0, 0), 0);

    let mut chunk_updates_hashset = HashSet::new();
    chunk_updates_hashset.insert((0, 0, 0));

    game.lose_terrain.objects.insert(
        LoseTerrainId(0),
        LoseObject {
            position: Vec3::new(0.0, 0.0, 0.0),
            velocity: Vec3::new(0.0, -0.1, 0.0),
            id: LoseTerrainId(0),
            min_max_x: (0, 0),
            min_max_y: (0, 0),
            min_max_z: (0, 0),
            chunks: lose_chunk_data,
            chunks_loc: lose_chunk_data_loc,
            chunks_updated: chunk_updates_hashset,
        },
    );
    game.lose_terrain.mesh_updated.insert(LoseTerrainId(0));
    game.lose_terrain.position_updated.insert(LoseTerrainId(0));

    game.lose_terrain.object_count += 1;

    //chunk generation thread start
    let _ = thread::spawn(move || {
        block_on(chunk_generation_thread(
            &mut chunk_generation_request_rx,
            chunk_generated_tx,
        ));
    });
    //chunk mesh creation thread start
    let _ = thread::spawn(move || {
        block_on(chunk_mesh_creation_thread(
            chunk_mesh_update_tx,
            &mut request_chunk_mesh_update_rx,
        ));
    });

    let mut last_tick_time = Instant::now();

    let mut start_time = SystemTime::now();
    println!("starting game loop");
    'game_loop: loop {
        handle_chunk_loaded(
            &mut game.world,
            &chunk_generated_rx,
            &player_position,
            &chunk_generation_request_tx,
        );

        if handle_user_inputs(&mut game.world, &mut player_position, input_event_rx) == false {
            break 'game_loop;
        }

        handle_pixel_updates(&mut game.world);

        handle_chunk_mesh_updates(
            &mut game.world,
            &mut game.lose_terrain,
            &request_chunk_mesh_update_tx,
        );

        //physics loop
        tick_physics(&mut game);

        handle_entity_update(&mut game, &entity_render_tx);
        handle_lose_object_update(&mut game.lose_terrain, &lose_object_update_tx);

        //for time being rotate

        for i in 0..5 {
            let obj = game.entities.entities.get_mut(&EntityId(i)).unwrap();
            obj.position.x =
                (start_time.elapsed().unwrap().as_secs_f32() * i as f32).sin() * i as f32 + 50.0;
            obj.position.z =
                (start_time.elapsed().unwrap().as_secs_f32() * i as f32).cos() * i as f32;
            game.entities.updated.insert(EntityId(i), ());
        }

        //60tps
        let sleep_time = Duration::from_millis(
            ((1000.0 / 60.0) - last_tick_time.elapsed().as_secs_f32()) as u64,
        );
        if sleep_time < Duration::from_secs(0) {
            println!("main game loop is lagging. Took more then needed time")
        }
        thread::sleep(sleep_time);
        last_tick_time = Instant::now();
    }

    println!("close main game loop")
    //start unloading data. This means doing things like saving all the chunks and entities in the world
}
