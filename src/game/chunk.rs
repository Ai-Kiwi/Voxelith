use crate::game::world::{PixelTypes,};


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