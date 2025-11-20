use std::{collections::HashMap, sync::Arc};

use crate::{game::chunk::{Chunk,}};

pub struct WorldData {
    pub chunks : HashMap<(i32, i32,i32), Arc<Chunk>>,
    //pub entities : HashMap<u64, Entity>,
    //pub entities_count : u64,
    pub pixel_edit_queue : Vec<(i32,i32,i32,PixelTypes)>,
    pub chunk_mesh_updates_needed : HashMap<(i32,i32,i32),()>,
    pub chunks_loading : HashMap<(i32,i32,i32),()>
}


impl WorldData {
    pub fn get_pixel_data(&self, pixel_x : i32, pixel_y : i32, pixel_z : i32) -> PixelTypes {
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
                return pixel
            },
            None => {
                return PixelTypes::Air
            },
        };
    }
    pub fn set_pixel_data(&mut self, pixel_pos : (i32,i32,i32), pixel : PixelTypes) {
        self.pixel_edit_queue.push((pixel_pos.0,pixel_pos.1,pixel_pos.2,pixel));
    }
}


#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PixelTypes {
    Air,
    Grass,
    Dirt,
    Stone,
    Water,
    Sand,
    Snow,
}