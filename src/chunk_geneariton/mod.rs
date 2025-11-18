use std::sync::mpsc::{Receiver, Sender};

use noise::{NoiseFn, Simplex};

use crate::game::{chunk::{Chunk}, world::PixelTypes};

pub struct NewChunkInfo {
    pub position : (i32, i32, i32),
    pub chunk : Chunk,
}

fn create_chunk(noise : &Simplex, chunk_pos : (i32, i32, i32)) -> Chunk {
    let mut created = Chunk {
        data: Vec::new(),
    };
    //println!("making chunk");
    for y in 0..16 {
        for z in 0..16 {
            for x in 0..16 {
                let world_x = x + (16 * chunk_pos.0);
                let world_y = y  + (16 * chunk_pos.1);
                let world_z = z + (16 * chunk_pos.2);

                //if y < 15 {
                //if y as i32 + (x as i32 - 64).abs() + (z as i32 - 64).abs() < 256 {
                //    pixel_block = PixelTypes::Grass;
                //}else {
                //    pixel_block = PixelTypes::Air;
                //}

                let value = noise.get([world_x as f64  * 0.0025, 0.0, world_z as f64 * 0.0025]);
                let ground_height = 50.0 + (value * 15.0);
                

                //types of noise
                //hummadity
                //magicness
                //tempature
                //Continentalness
                //Erosion Map
                //Weirdness (used for peaks and valleys)

                //Vegetation
                //tree
                //Roughness map
                //river map

                //start adding 3d noise to have layered terrain

                //switch to fractal noise



                
                //created.set_relative_pixel(x, y, z, PixelTypes::Grass);

                let pixel_block= if (world_y as f64) < ground_height - 5.0 {
                    PixelTypes::Stone
                }else if (world_y as f64) < ground_height - 3.0 {
                    PixelTypes::Dirt
                }else if (world_y as f64) < ground_height {
                    PixelTypes::Grass
                }else{
                    PixelTypes::Air
                };

                created.data.push(pixel_block);
                
            }
        }
    }



    created
}

pub async fn chunk_generation_thread(chunk_generation_request_rx : &mut Receiver<(i32,i32,i32)>, chunk_generated_tx : Sender<NewChunkInfo>) {
    let noise: Simplex = Simplex::new(1237456345);

    create_chunk(&noise, (0,0,0));

    loop {
        //on create new chunk request
        loop {
            let chunk_generation_request = chunk_generation_request_rx.try_recv();
            match chunk_generation_request {
                Ok(request) => {
                    let new_chunk = create_chunk(&noise, request);
                    let _ = chunk_generated_tx.send(NewChunkInfo { 
                        position: request, 
                        chunk: new_chunk
                    });
                },
                Err(_) => {
                    break
                },
            }
        }

    }
}