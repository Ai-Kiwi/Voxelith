use std::{collections::HashMap, sync::Arc};

use crate::game::{chunk::Chunk, pixel::PixelTypes};

pub struct WorldData {
    pub chunks : HashMap<(i32, i32,i32), Arc<Chunk>>,
    pub pixel_edit_queue : Vec<(i32,i32,i32,PixelTypes)>,
    pub chunk_mesh_updates_needed : HashMap<(i32,i32,i32),()>,
    pub chunks_loading : HashMap<(i32,i32,i32),()>,
    pub chunk_shadows : HashMap<(i32,i32,i32), Vec<u8>>,
}


impl WorldData {
    pub fn get_pixel_data(&self, pixel_x : i32, pixel_y : i32, pixel_z : i32) -> Option<PixelTypes> {
        let chunk_x = pixel_x.div_euclid(16);
        let chunk_y = pixel_y.div_euclid(16);
        let chunk_z = pixel_z.div_euclid(16);
        let local_x = pixel_x.rem_euclid(16);
        let local_y = pixel_y.rem_euclid(16);
        let local_z = pixel_z.rem_euclid(16);
        let chunk_data = self.chunks.get(&(chunk_x,chunk_y,chunk_z));
        match chunk_data {
            Some(chunk) => {
                let pixel = chunk.get_relative_pixel(local_x, local_y, local_z);
                return Some(pixel)
            },
            None => {
                return None
            },
        };
    }
    pub fn set_pixel_data(&mut self, pixel_pos : (i32,i32,i32), pixel : PixelTypes) {
        self.pixel_edit_queue.push((pixel_pos.0,pixel_pos.1,pixel_pos.2,pixel));
    }

    pub fn test_collision_area(&self, start_pixel_pos : (i32,i32,i32), end_pixel_pos : (i32,i32,i32)) -> bool {
        let start_chunk_x = start_pixel_pos.0.div_euclid(16);
        let start_chunk_y = start_pixel_pos.0.div_euclid(16);
        let start_chunk_z = start_pixel_pos.0.div_euclid(16);
        let end_chunk_x = end_pixel_pos.0.div_euclid(16);
        let end_chunk_y = end_pixel_pos.0.div_euclid(16);
        let end_chunk_z = end_pixel_pos.0.div_euclid(16);

        for chunk_y in start_chunk_y..end_chunk_y as i32 {
            for chunk_z in start_chunk_z..end_chunk_z as i32 {
                for chunk_x in start_chunk_x..end_chunk_x as i32 {
                    let chunk_data = self.chunks.get(&(chunk_x,chunk_y,chunk_z));
                    for local_y in 0..16 as i32 {
                        for local_z in 0..16 as i32 {
                            for local_x in 0..16 as i32 {
                                let block_x = (chunk_x * 16) + local_x;
                                let block_y = (chunk_y * 16) + local_y;
                                let block_z = (chunk_z * 16) + local_z;
                                if block_x <= end_chunk_x && block_x >= start_chunk_x && block_y <= end_chunk_y && block_y >= start_chunk_y && block_z <= end_chunk_z && block_z >= start_chunk_z {
                                    match chunk_data {
                                        Some(data) => {
                                            if data.get_relative_pixel(local_x, local_y, local_z).should_be_collision() {
                                                return true
                                            }
                                        }
                                        None => (),
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        return false;
    }
}