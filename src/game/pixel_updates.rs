use std::{collections::HashMap, sync::Arc};

use crate::game::{chunk, world::WorldData};

pub fn handle_pixel_updates(world : &mut WorldData) {
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
}