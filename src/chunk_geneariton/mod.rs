use std::{os::unix::thread, sync::mpsc::{Receiver, Sender}, thread::sleep};

use noise::{NoiseFn, Simplex, core::simplex::simplex_3d, math};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{game::{chunk::{self, Chunk}, world::PixelTypes}, utils::VoxelPosition};

pub struct NewChunkInfo {
    pub position : (i32, i32, i32),
    pub chunk : Chunk,
}

fn steepen_keep_range(v: f64, power: f64) -> f64 {
    if power == 1.0 {
        return v
    }
    let sign = v.signum();
    let n = v.abs().powf(power);
    let max = 1.0_f64.powf(power);
    sign * (n / max)
}

const WORLD_SCALE : f64 = 16.0;

//fn get_height(power : f64, frequency : f64, height : f64, random : f64, block_pos : VoxelPosition,  noise: &Simplex) -> f64 {
//    let value =  noise.get([(block_pos.x as f64  * (frequency / 1000.0) * WORLD_SCALE) + random, 0.0, (block_pos.z as f64 * (frequency / 1000.0) * WORLD_SCALE) + random]);
//
//    steepen_keep_range(value, power) * (height / WORLD_SCALE)
//}

fn get_multi_octave_map( frequency : f64, block_pos : VoxelPosition, random : f64, noise: &Simplex) -> f64 {
    let mut value =  noise.get([(block_pos.x as f64  * frequency * WORLD_SCALE) + (random * 32.0), 0.0, (block_pos.z as f64 * frequency * WORLD_SCALE) + (random * 32.0)]);
    value += noise.get([(block_pos.x as f64  * frequency * WORLD_SCALE * 2.0) + (random * 16.0), 0.0, (block_pos.z as f64 * frequency * WORLD_SCALE * 2.0) + (random * 16.0)]) * 2.0;
    value += noise.get([(block_pos.x as f64  * frequency * WORLD_SCALE * 4.0) + (random * 8.0), 0.0, (block_pos.z as f64 * frequency * WORLD_SCALE * 4.0) + (random * 8.0)]) * 4.0;
    value += noise.get([(block_pos.x as f64  * frequency * WORLD_SCALE * 8.0) + (random * 4.0), 0.0, (block_pos.z as f64 * frequency * WORLD_SCALE * 8.0) + (random * 4.0)]) * 6.0;
    value += noise.get([(block_pos.x as f64  * frequency * WORLD_SCALE * 16.0) + (random * 2.0), 0.0, (block_pos.z as f64 * frequency * WORLD_SCALE * 16.0) + (random * 2.0)]) * 8.0;
    value += noise.get([(block_pos.x as f64  * frequency * WORLD_SCALE * 32.0) + (random * 1.0), 0.0, (block_pos.z as f64 * frequency * WORLD_SCALE * 32.0) + (random * 1.0)]) * 10.0;



    return value / (1.0 + 2.0 + 4.0 + 6.0 + 8.0 + 10.0)
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
                let voxel_position = VoxelPosition {
                    x: world_x,
                    y: world_y,
                    z: world_z,
                };

                //if y < 15 {
                //if y as i32 + (x as i32 - 64).abs() + (z as i32 - 64).abs() < 256 {
                //    pixel_block = PixelTypes::Grass;
                //}else {
                //    pixel_block = PixelTypes::Air;
                //}

                //let value = noise.get([world_x as f64  * 0.0025, 0.0, world_z as f64 * 0.0025]);

                let continentalness = get_multi_octave_map(0.00000001, voxel_position, 634643.0,noise);


                let mut ground_height: f64 = 0.0;

                let ground_roughness: f64 = (get_multi_octave_map(0.000001, voxel_position, 6235235235.0,noise) + 1.0) * 0.5;

                //ground_height += 500.0 * ground_roughness;

                ground_height += 500.0 * (get_multi_octave_map(0.0001, voxel_position, 2134315324.0,noise)).abs() * ground_roughness * continentalness;



                //mountains
                //ground_height += get_height(2.0, 0.3, 750.0, 645653.0, voxel_position, noise);
                //slight details
                //ground_height += get_height(1.0, 1.5, 45.0, 645653.0, voxel_position, noise);
                //plains
                //ground_height += get_height(1.0, 0.015, 300.0, 645653.0, voxel_position, noise);

                //Continentalness
                //ground_height += get_height(0.95, 0.0005, 300.0, 645653.0, voxel_position, noise);


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

                ground_height = ground_height / WORLD_SCALE;

                
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
        let mut requests: Vec<_> = Vec::new();

        while let Ok(req) = chunk_generation_request_rx.try_recv() {
            requests.push(req);
            if requests.len() > 25 {
                break;
            }
        }

        if requests.is_empty() {
            // sleep briefly to avoid burning CPU
            sleep(std::time::Duration::from_millis(10));
            continue;
        }

        let results : Vec<_> = requests
        .par_iter()
        .map(|pos| {
            let chunk = create_chunk(&noise, *pos);
            (*pos, chunk)
        })
        .collect();

        for (pos, chunk) in results {
            let _ = chunk_generated_tx.send(NewChunkInfo { 
                position: pos, 
                chunk: chunk
            });
        }
    }
}