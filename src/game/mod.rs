use std::{collections::HashMap, sync::{Arc, mpsc::{Receiver, Sender, channel}}, thread, time::Duration};
use futures::executor::block_on;

use crate::{chunk_geneariton::{NewChunkInfo, chunk_generation_thread}, game::world::WorldData, mesh_creation::{ChunkMeshCreateRequest, chunk_mesh_creation_thread}, render::types::{ChunkMeshUpdate, EntityRenderData}, utils::{Vec2, Vec3, raycast_test}};

pub mod world;
pub mod chunk;

pub enum InputEvent {
    Move(Vec2),
    Jump,
    Click(Vec3,Vec3)
    //Attack,
    //OpenInventory,
    //UseItem(u32), // item ID
    //DropItem(u32),
    // ... other game actions
}


pub async fn game_thread(chunk_mesh_update_tx : Sender<ChunkMeshUpdate>, entity_render_tx : Sender<EntityRenderData>, input_event_rx : &mut Receiver<InputEvent>) {
    let _ = entity_render_tx;
    let mut world: WorldData = WorldData {
        chunks: HashMap::new(),
        //entities: HashMap::new(),
        //entities_count: 0,
        pixel_edit_queue: Vec::new(),
        chunk_mesh_updates_needed: HashMap::new()
    };


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


    for x in 0..=50 {
        for z in 0..=50 {
            for y in -40..=40 {
                let _ = chunk_generation_request_tx.send((x,y,z));
            }
        }
    }

    println!("starting game loop");
    loop {
        //chunk load 
        loop {
            let chunk_generated = chunk_generated_rx.try_recv();
            match chunk_generated {
                Ok(new_chunk) => {
                    let chunk = new_chunk.chunk;
                    let arc = Arc::new(chunk);
                    world.chunks.insert(new_chunk.position, arc);

                    //tell chunk itself and ones around that they need mesh update/render
                    world.chunk_mesh_updates_needed.insert((new_chunk.position.0, new_chunk.position.1, new_chunk.position.2), ());

                    world.chunk_mesh_updates_needed.insert((new_chunk.position.0 + 1, new_chunk.position.1, new_chunk.position.2), ());
                    world.chunk_mesh_updates_needed.insert((new_chunk.position.0 - 1, new_chunk.position.1, new_chunk.position.2), ());

                    world.chunk_mesh_updates_needed.insert((new_chunk.position.0, new_chunk.position.1 + 1, new_chunk.position.2), ());
                    world.chunk_mesh_updates_needed.insert((new_chunk.position.0, new_chunk.position.1 - 1, new_chunk.position.2), ());

                    world.chunk_mesh_updates_needed.insert((new_chunk.position.0, new_chunk.position.1, new_chunk.position.2 + 1), ());
                    world.chunk_mesh_updates_needed.insert((new_chunk.position.0, new_chunk.position.1, new_chunk.position.2 - 1), ());
                },
                Err(_) => {
                    break
                },
            }
        }

        //input event handling
        loop {
            let player_input_event = input_event_rx.try_recv();
            match player_input_event {
                Ok(player_input_update) => {
                    match player_input_update {
                        InputEvent::Move(_vec2) => {
                            //world.entities.get_mut(&0).unwrap().physics.velocity.0;
                            //world.entities.get_mut(&0).unwrap().physics.velocity.2;
                        },
                        InputEvent::Jump => {
                            println!("jump");
                            //if world.entities.get(&0).unwrap().physics.grounded == true {
                            //    world.entities.get_mut(&0).unwrap().physics.velocity.1 = 6.0;
                            //}
                        },
                        InputEvent::Click(position, facing_dir) => {
                            let mut i = 0;
                            'rayloop : for ray in raycast_test(position, facing_dir.normalize()) {
                                i+=1;
                                if world.get_pixel_data(ray.x as i32,ray.y as i32,ray.z as i32) != world::PixelTypes::Air {
                                    for x in -5..=5 {
                                        for y in -5..=5 {
                                            for z in -5..=5 {
                                                world.set_pixel_data((ray.x as i32 + x,ray.y as i32 + y ,ray.z as i32 + z), world::PixelTypes::Air);
                                            }
                                        }
                                    }
                                    break 'rayloop;
                                }
                                
                                
                                //world.set_pixel_data((ray.x as i32,ray.y as i32,ray.z as i32), world::PixelTypes::Stone);

                                if i > 500 {
                                    break;
                                }
                            }
                        },
                    }
                },
                Err(_) => {
                    break
                },
            }
        }

        //apply pixels which have been edited
        let mut edited_chunk_data: HashMap<(i32, i32, i32), chunk::Chunk> = HashMap::new();
        for pixel_data in &world.pixel_edit_queue {
            let chunk_x = pixel_data.0.div_euclid(16);
            let chunk_y = pixel_data.1.div_euclid(16);
            let chunk_z = pixel_data.2.div_euclid(16);
            let local_x = pixel_data.0.rem_euclid(16);
            let local_y = pixel_data.1.rem_euclid(16);
            let local_z = pixel_data.2.rem_euclid(16);
            let full_position = (chunk_x,chunk_y,chunk_z);

            if !edited_chunk_data.contains_key(&full_position) {
                if let Some(chunk_data) = world.chunks.get(&full_position) {
                    edited_chunk_data.insert(full_position, (**chunk_data).clone());
                }else {
                    continue;
                }
            }
            if let Some(chunk_data) = edited_chunk_data.get_mut(&(chunk_x,chunk_y,chunk_z)) {
                chunk_data.set_relative_pixel(local_x as usize, local_y as usize, local_z as usize, pixel_data.3);

                world.chunk_mesh_updates_needed.insert((chunk_x,chunk_y,chunk_z), ());

                if local_x == 0 { world.chunk_mesh_updates_needed.insert((chunk_x - 1,chunk_y,chunk_z), ()); }
                if local_x == 15 { world.chunk_mesh_updates_needed.insert((chunk_x + 1,chunk_y,chunk_z), ()); }
                if local_y == 0 { world.chunk_mesh_updates_needed.insert((chunk_x,chunk_y - 1,chunk_z), ()); }
                if local_y == 15 { world.chunk_mesh_updates_needed.insert((chunk_x,chunk_y + 1,chunk_z), ()); }
                if local_z == 0 { world.chunk_mesh_updates_needed.insert((chunk_x,chunk_y,chunk_z - 1), ()); }
                if local_z == 15 { world.chunk_mesh_updates_needed.insert((chunk_x,chunk_y,chunk_z + 1), ()); }
            }
        }
        world.pixel_edit_queue.clear();
        let chunk_keys: Vec<_> = edited_chunk_data.keys().cloned().collect();
        for chunk in chunk_keys {
            if let Some(chunk_value) = edited_chunk_data.remove(&chunk) {
                world.chunks.insert(chunk, Arc::new(chunk_value));
            }
        }





        //chunk mesh handling
        for chunk_mesh_updates in &mut world.chunk_mesh_updates_needed {
            if let Some(chunk) = world.chunks.get(chunk_mesh_updates.0) {
                let _ = request_chunk_mesh_update_tx.send(ChunkMeshCreateRequest {
                    chunk: chunk.clone(),
                    position: *chunk_mesh_updates.0,
                    x_positive_chunk_neighbor: world.chunks.get(&(chunk_mesh_updates.0.0 + 1,chunk_mesh_updates.0.1,chunk_mesh_updates.0.2)).cloned(),
                    x_negative_chunk_neighbor: world.chunks.get(&(chunk_mesh_updates.0.0 - 1,chunk_mesh_updates.0.1,chunk_mesh_updates.0.2)).cloned(),
                    y_positive_chunk_neighbor: world.chunks.get(&(chunk_mesh_updates.0.0,chunk_mesh_updates.0.1 + 1,chunk_mesh_updates.0.2)).cloned(),
                    y_negative_chunk_neighbor: world.chunks.get(&(chunk_mesh_updates.0.0,chunk_mesh_updates.0.1 - 1,chunk_mesh_updates.0.2)).cloned(),
                    z_positive_chunk_neighbor: world.chunks.get(&(chunk_mesh_updates.0.0,chunk_mesh_updates.0.1,chunk_mesh_updates.0.2 + 1)).cloned(),
                    z_negative_chunk_neighbor: world.chunks.get(&(chunk_mesh_updates.0.0,chunk_mesh_updates.0.1,chunk_mesh_updates.0.2 - 1)).cloned(),
                });
            }
        }
        world.chunk_mesh_updates_needed.clear();

        //physics loop
        //tick_physics(&mut world);

       

        //send updates for entities
        //for entity in &mut world.entities {
        //    if entity.1.updated == true {
        //        let _ = entity_render_tx.send(EntityRenderData {
        //            id: *entity.0,
        //            alive: true,
        //            position: entity.1.position,
        //            entity_type: crate::entity::EntityType::Player,
        //        });
        //        entity.1.updated = false;
        //    }
        //}


        thread::sleep(Duration::from_millis(16));
    }

}