use std::sync::{Arc, mpsc::{Receiver, Sender}};

use crate::{chunk_geneariton::NewChunkInfo, game::{self, MAX_CHUNK_LOAD_DISTANCE, MIN_CHUNK_UNLOAD_DISTANCE, pixel::PixelTypes, world::WorldData}, render, utils::Vec3};


#[derive(Clone)]
pub struct Chunk {
    pub data : Vec<PixelTypes>,
}

impl Chunk {
    pub fn set_relative_pixel(self : &mut Chunk, x : usize, y : usize, z : usize, pixel : PixelTypes) {
        self.data[(x) + (z * 16) + (y * 16 * 16)] = pixel;
    }
    pub fn get_relative_pixel(self : &Chunk, x : i32, y : i32, z : i32) -> PixelTypes {
        self.data[(x as usize) + (z as usize * 16) + (y as usize * 16 * 16)]
    }
}

pub fn handle_chunk_loaded(world : &mut WorldData, chunk_generated_rx : &Receiver<NewChunkInfo>, player_position : &Vec3, chunk_generation_request_tx : &Sender<(i32, i32, i32)>) {
    let middle_chunk_x = (player_position.x + 8.0).div_euclid(16.0) as i32;
    let middle_chunk_y = (player_position.y + 8.0).div_euclid(16.0) as i32;
    let middle_chunk_z = (player_position.z + 8.0).div_euclid(16.0) as i32;
    let mut range = 0;
    let mut range_increased = false;
    if world.chunks_loading.iter().count() > 25 {
        range_increased = true
    }
    while range_increased == false {
        //chunk x range
        for chunk_x in (middle_chunk_x - range)..=(middle_chunk_x + range) {
            for chunk_y in (middle_chunk_y - range)..=(middle_chunk_y + range) {
                if world.chunks.contains_key(&(chunk_x,chunk_y,middle_chunk_z + range)) == false && world.chunks_loading.contains_key(&(chunk_x,chunk_y,middle_chunk_z + range)) == false {
                    world.chunks_loading.insert((chunk_x,chunk_y,middle_chunk_z + range), ());
                    let _ = chunk_generation_request_tx.send((chunk_x,chunk_y,middle_chunk_z + range));
                    range_increased = true;
                }
    
                if world.chunks.contains_key(&(chunk_x,chunk_y,middle_chunk_z - range)) == false && world.chunks_loading.contains_key(&(chunk_x,chunk_y,middle_chunk_z - range)) == false {
                    world.chunks_loading.insert((chunk_x,chunk_y,middle_chunk_z - range), ());
                    let _ = chunk_generation_request_tx.send((chunk_x,chunk_y,middle_chunk_z - range));
                    range_increased = true;
                }
            }
        }
    
        //chunk z range
        for chunk_z in (middle_chunk_z - (range - 1))..=(middle_chunk_z + (range - 1)) {
            for chunk_y in (middle_chunk_y - range)..=(middle_chunk_y + range) {
                if world.chunks.contains_key(&(middle_chunk_x + range,chunk_y,chunk_z)) == false && world.chunks_loading.contains_key(&(middle_chunk_x + range,chunk_y,chunk_z)) == false {
                    world.chunks_loading.insert((middle_chunk_x + range,chunk_y,chunk_z), ());
                    let _ = chunk_generation_request_tx.send((middle_chunk_x + range,chunk_y,chunk_z));
                    range_increased = true;
                }
    
                if world.chunks.contains_key(&(middle_chunk_x - range,chunk_y,chunk_z)) == false && world.chunks_loading.contains_key(&(middle_chunk_x - range,chunk_y,chunk_z)) == false {
                    world.chunks_loading.insert((middle_chunk_x - range,chunk_y,chunk_z), ());
                    let _ = chunk_generation_request_tx.send((middle_chunk_x - range,chunk_y,chunk_z));
                    range_increased = true;
                }
            }
        }
    
        //top and bottom
        for chunk_z in (middle_chunk_z - (range - 1))..=(middle_chunk_z + (range - 1)) {
            for chunk_x in (middle_chunk_x - (range - 1))..=(middle_chunk_x + (range - 1)) {
                if world.chunks.contains_key(&(chunk_x,(middle_chunk_y + range),chunk_z)) == false && world.chunks_loading.contains_key(&(chunk_x,(middle_chunk_y + range),chunk_z)) == false {
                    world.chunks_loading.insert((chunk_x,(middle_chunk_y + range),chunk_z), ());
                    let _ = chunk_generation_request_tx.send((chunk_x,(middle_chunk_y + range),chunk_z));
                    range_increased = true;
                }
    
                if world.chunks.contains_key(&(chunk_x,(middle_chunk_y - range),chunk_z)) == false && world.chunks_loading.contains_key(&(chunk_x,(middle_chunk_y - range),chunk_z)) == false {
                    world.chunks_loading.insert((chunk_x,(middle_chunk_y - range),chunk_z), ());
                    let _ = chunk_generation_request_tx.send((chunk_x,(middle_chunk_y - range),chunk_z));
                    range_increased = true;
                }
            }
        }
        if range > MAX_CHUNK_LOAD_DISTANCE {
            break;
        }
        range += 1;
    }
    
    //unload chunk
    world.chunks.retain(|position, _| {
        let keep = (position.0 - middle_chunk_x).abs() < MIN_CHUNK_UNLOAD_DISTANCE && (position.1 - middle_chunk_y).abs() < MIN_CHUNK_UNLOAD_DISTANCE && (position.2 - middle_chunk_z).abs() < MIN_CHUNK_UNLOAD_DISTANCE;
        if !keep {
            world.chunk_mesh_updates_needed.insert(*position, ());
        }
        return keep
    });

    loop {
        let chunk_generated = chunk_generated_rx.try_recv();
        match chunk_generated {
            Ok(new_chunk) => {
                let chunk = new_chunk.chunk;
                let arc = Arc::new(chunk);
                world.chunks.insert(new_chunk.position, arc);

                //remove if planning to load
                if world.chunks_loading.contains_key(&new_chunk.position) {
                    world.chunks_loading.remove(&new_chunk.position);
                }

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
}