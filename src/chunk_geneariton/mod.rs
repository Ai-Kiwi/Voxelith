use core::f32;
use std::{sync::mpsc::{Receiver, Sender}, thread::sleep, time::Instant};
use fastnoise_lite::{FastNoiseLite, NoiseType};
use rayon::{ThreadPoolBuilder, iter::{IntoParallelRefIterator, ParallelIterator}};
use crate::{game::{chunk::{self, Chunk}, pixel::PixelTypes}, utils::VoxelPosition};

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



//fn get_height(power : f64, frequency : f64, height : f64, random : f64, block_pos : VoxelPosition,  noise: &Simplex) -> f64 {
    //    let value =  noise.get([(block_pos.x as f64  * (frequency / 1000.0) * WORLD_SCALE) + random, 0.0, (block_pos.z as f64 * (frequency / 1000.0) * WORLD_SCALE) + random]);
    //
    //    steepen_keep_range(value, power) * (height / WORLD_SCALE)
    //}
    
fn get_multi_octave_map( frequency : f32, block_pos : VoxelPosition, random : f32, noise: &FastNoiseLite) -> f32 {
    let mut value =  noise.get_noise_2d((block_pos.x as f32  * frequency * WORLD_SCALE) + (random * 4.0), (block_pos.z as f32 * frequency * WORLD_SCALE) + (random * 4.0));
    value += noise.get_noise_2d((block_pos.x as f32  * frequency * WORLD_SCALE * 2.0) + (random * 2.0), (block_pos.z as f32 * frequency * WORLD_SCALE * 2.0) + (random * 2.0)) * 2.0;
    value += noise.get_noise_2d((block_pos.x as f32  * frequency * WORLD_SCALE * 4.0) + (random * 1.0), (block_pos.z as f32 * frequency * WORLD_SCALE * 4.0) + (random * 1.0)) * 4.0;
    
    return value / (1.0 + 2.0 + 4.0)
}
    
const WORLD_SCALE : f32 = 1.0;
//const WORLD_SCALE : f32 = 10.0;

enum Biome {
    Plains,
    Ocean,
    Desert,
    //SnowyTundra,
    SnowyPlains
}

struct BiomeRule {
    biome: Biome,
    elevation_range: (f32, f32),
    humidity_range: (f32, f32),
    temperature_range: (f32, f32),
    magic_range: (f32, f32),
    continentalness : (f32, f32),
    weight: f32, // optional
}

const BIOME_RULES : [BiomeRule; 4] = [
    BiomeRule {
        biome: Biome::Plains,
        elevation_range: (-f32::INFINITY, f32::INFINITY),
        humidity_range: (-1.0, 1.0),
        temperature_range: (-1.0, 1.0),
        magic_range: (-1.0, 1.0),
        continentalness: (-1.0, 1.0),
        weight: 0.0,
    },
    BiomeRule {
        biome: Biome::Desert,
        elevation_range: (-f32::INFINITY, f32::INFINITY),
        humidity_range: (-1.0, 1.0),
        temperature_range: (0.35, 1.0),
        magic_range: (-1.0, 1.0),
        continentalness: (-1.0, 1.0),
        weight: 1.0,
    },
    BiomeRule {
        biome: Biome::SnowyPlains,
        elevation_range: (-f32::INFINITY, f32::INFINITY),
        humidity_range: (0.0, 1.0),
        temperature_range: (-1.0, -0.4),
        magic_range: (-1.0, 1.0),
        continentalness: (-1.0, 1.0),
        weight: 1.0,
    },
    BiomeRule {
        biome: Biome::Ocean,
        elevation_range: (-f32::INFINITY, 0.0),
        humidity_range: (-1.0, 1.0),
        temperature_range: (-1.0, 1.0),
        magic_range: (-1.0, 1.0),
        continentalness: (-1.0, 0.0),
        weight: 10.0,
    }
];

fn pick_biome(e: f32, h: f32, t: f32, m: f32, c : f32) -> &'static Biome {
    BIOME_RULES.iter()
        .filter(|rule| e >= rule.elevation_range.0 && e <= rule.elevation_range.1)
        .filter(|rule| h >= rule.humidity_range.0 && h <= rule.humidity_range.1)
        .filter(|rule| t >= rule.temperature_range.0 && t <= rule.temperature_range.1)
        .filter(|rule| m >= rule.magic_range.0 && m <= rule.magic_range.1)
        .filter(|rule| c >= rule.continentalness.0 && c <= rule.continentalness.1)
        .max_by(|a, b| a.weight.partial_cmp(&b.weight).unwrap())
        .map(|rule| &rule.biome)
        .unwrap_or(&Biome::Plains)
}



    
fn create_chunk(noise : &FastNoiseLite, chunk_pos : (i32, i32, i32)) -> Chunk {
    let mut created = Chunk {
        data: Vec::new(),
    };

    let mut biome_and_voxel_data: Vec<(f32,&Biome)> = Vec::new();

    for z in 0..16 {
        for x in 0..16 {
            let world_x = x + (16 * chunk_pos.0);
            let world_y = 16 * chunk_pos.1;
            let world_z = z + (16 * chunk_pos.2);
            let voxel_position = VoxelPosition {
                x: world_x,
                y: 0,
                z: world_z,
            };

            //if y < 15 {
            //if y as i32 + (x as i32 - 64).abs() + (z as i32 - 64).abs() < 256 {
            //    pixel_block = PixelTypes::Grass;
            //}else {
            //    pixel_block = PixelTypes::Air;
            //}

            //let value = noise.get([world_x as f64  * 0.0025, 0.0, world_z as f64 * 0.0025]);

            let continentalness: f32 = get_multi_octave_map(0.003125, voxel_position, 0.0,noise);
            
            
            let continentalness: f32 = (((continentalness + 1.0) / 2.0).powf(1.5) - 0.5) * 2.0;
            //println!("{}",continentalness);
            
            //let continental_mask: f64 = if continentalness > 0.0 { 1.0 } else { -1.0 };
            //let continentalness: f64 = continentalness.abs().powf(0.05) * continental_mask;
            //let continentalness: f64 = continentalness;

            let continentalness: f32 = continentalness.signum() * continentalness.abs().powf(0.9); 
            let mountainness = (get_multi_octave_map(0.0125, voxel_position, 0.0,noise)).abs().powf(3.0);
            let hillness = (get_multi_octave_map(0.0025, voxel_position, 0.0,noise)).abs().powf(2.0);
            let roughness = get_multi_octave_map(0.25, voxel_position, 23453.0,noise);

            let mut elevation: f32 = 0.0;

            //sharp mountains
            elevation += 150.0 * mountainness;
            //hill
            elevation += 300.0 * hillness;
            //slight extra stuff

            elevation = elevation + (continentalness * 150.0);
            elevation = elevation / WORLD_SCALE;

            let humidity = get_multi_octave_map(0.0075, voxel_position, 53455.0,noise);
            let temperature = get_multi_octave_map(0.0025, voxel_position, 34545.0,noise);
            let magic = get_multi_octave_map(0.000625, voxel_position, 345435.0,noise);

            let biome = pick_biome(elevation, humidity, temperature, magic, continentalness);

            elevation +=  roughness * (1.0 + (hillness * 2.0) * (mountainness * 5.0).clamp(0.0, 2.0));

            biome_and_voxel_data.push((elevation, biome));

            //causes alot of issues with rendering for some reason so im going to ignore
            //if elevation + 25.0 < world_y as f32 && world_y > 0 {
            //    for _ in 0..(16*16*16) {
            //        created.data.push(PixelTypes::Air);
            //    }
            //    return created;
            //}
        }
    }
    
    for y in 0..16 as i32 {
        for z in 0..16 as i32 {
            for x in 0..16 as i32 {
                let data = biome_and_voxel_data.get((x + (16 * z)) as usize).unwrap();
                let mut pixel_block = PixelTypes::Air;
                let elevation = data.0;
                let world_x  = x + (16 * chunk_pos.0) as i32;
                let world_y  = y + (16 * chunk_pos.1) as i32;
                let world_z  = z + (16 * chunk_pos.2) as i32;
                match data.1 {
                    Biome::Plains => {
                        if (world_y as f32) < elevation - 4.0 {
                            pixel_block = PixelTypes::Stone
                        }else if (world_y as f32) < elevation - 1.0 {
                            pixel_block = PixelTypes::Dirt
                        }else if (world_y as f32) < elevation {
                            pixel_block = PixelTypes::Grass
                        }
                    },
                    Biome::Ocean => {
                        if (world_y as f32) < 0.0 {
                            pixel_block = PixelTypes::Water
                        }
                        if (world_y as f32) < elevation - 3.0 {
                            pixel_block = PixelTypes::Stone
                        }else if (world_y as f32) < elevation {
                            pixel_block = PixelTypes::Dirt
                        }
                    },
                    Biome::Desert => {
                        if (world_y as f32) < elevation - 4.0 {
                            pixel_block = PixelTypes::Stone
                        }else if (world_y as f32) < elevation {
                            pixel_block = PixelTypes::Sand
                        }
                    },
                    Biome::SnowyPlains => {
                        if (world_y as f32) < elevation - 4.0 {
                            pixel_block = PixelTypes::Stone
                        }else if (world_y as f32) < elevation - 1.0 {
                            pixel_block = PixelTypes::Dirt
                        }else if (world_y as f32) < elevation {
                            pixel_block = PixelTypes::Snow
                        }
                    },
                }
                created.data.push(pixel_block);
            }
        }
    }
    
    created
}

//biomes
//snow  --done
//ice structures everywhere

//plain --done
//small non touchable grass

//jungle
//large trees
//large palm tree kinda things
//tons of vines
//very densely populated

//ice
//large sizes

//dark woods
//tons of trees, very dark

//crystal forest
//all follow colors, blue, green, red, purple, red
//bright colorful trees
//crytstels

//swamp
//dark water
//large vines
//trees in water

//volcano
//large sizes
//lava pooring down size





//canyon
//clay biomes
//flower biome
//rock biome
//desert  --done

//floating islands
//lava hell theme


//add later



pub async fn chunk_generation_thread(chunk_generation_request_rx : &mut Receiver<(i32,i32,i32)>, chunk_generated_tx : Sender<NewChunkInfo>) {
    let mut noise = FastNoiseLite::new();
    noise.set_seed(Some(34653452));
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));


    loop {
        let mut requests: Vec<_> = Vec::new();

        for _ in 0..5 {
            match chunk_generation_request_rx.try_recv() {
                Ok(req) => {
                    requests.push(req);
                    if requests.len() > 4 {
                        break;
                    }
                },
                Err(err) => {
                    match err {
                        std::sync::mpsc::TryRecvError::Empty => (),
                        std::sync::mpsc::TryRecvError::Disconnected => {
                            return //must have been dropped so close
                        },
                    }
                },
            }
        }

        if requests.is_empty() {
            // sleep briefly to avoid burning CPU
            sleep(std::time::Duration::from_millis(10));
            continue;
        }

        let results : Vec<_> = requests
        .iter() //.par_iter for mulithread
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