use std::{os::unix::thread, sync::mpsc::{Receiver, Sender}, thread::sleep};
use fastnoise_lite::{FastNoiseLite, NoiseType};
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



//fn get_height(power : f64, frequency : f64, height : f64, random : f64, block_pos : VoxelPosition,  noise: &Simplex) -> f64 {
    //    let value =  noise.get([(block_pos.x as f64  * (frequency / 1000.0) * WORLD_SCALE) + random, 0.0, (block_pos.z as f64 * (frequency / 1000.0) * WORLD_SCALE) + random]);
    //
    //    steepen_keep_range(value, power) * (height / WORLD_SCALE)
    //}
    
fn get_multi_octave_map( frequency : f32, block_pos : VoxelPosition, random : f32, noise: &FastNoiseLite) -> f32 {
    let new_freq = frequency * 25.0;
    let mut value =  noise.get_noise_2d((block_pos.x as f32  * new_freq * WORLD_SCALE) + (random * 32.0), (block_pos.z as f32 * new_freq * WORLD_SCALE) + (random * 32.0));
    value += noise.get_noise_2d((block_pos.x as f32  * new_freq * WORLD_SCALE * 2.0) + (random * 16.0), (block_pos.z as f32 * new_freq * WORLD_SCALE * 2.0) + (random * 16.0)) * 2.0;
    value += noise.get_noise_2d((block_pos.x as f32  * new_freq * WORLD_SCALE * 4.0) + (random * 8.0), (block_pos.z as f32 * new_freq * WORLD_SCALE * 4.0) + (random * 8.0)) * 4.0;
    value += noise.get_noise_2d((block_pos.x as f32  * new_freq * WORLD_SCALE * 8.0) + (random * 4.0), (block_pos.z as f32 * new_freq * WORLD_SCALE * 8.0) + (random * 4.0)) * 6.0;
    value += noise.get_noise_2d((block_pos.x as f32  * new_freq * WORLD_SCALE * 16.0) + (random * 2.0), (block_pos.z as f32 * new_freq * WORLD_SCALE * 16.0) + (random * 2.0)) * 8.0;
    value += noise.get_noise_2d((block_pos.x as f32  * new_freq * WORLD_SCALE * 32.0) + (random * 1.0), (block_pos.z as f32 * new_freq * WORLD_SCALE * 32.0) + (random * 1.0)) * 10.0;
    
    return value / (1.0 + 2.0 + 4.0 + 6.0 + 8.0 + 10.0)
}
    
const WORLD_SCALE : f32 = 0.05;
//const WORLD_SCALE : f32 = 15.0;

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

                let continentalness: f32 = get_multi_octave_map(0.00005, voxel_position, 0.0,noise);
                
                
                let continentalness: f32 = (((continentalness + 1.0) / 2.0).powf(1.5) - 0.5) * 2.0;
                //println!("{}",continentalness);
                
                //let continental_mask: f64 = if continentalness > 0.0 { 1.0 } else { -1.0 };
                //let continentalness: f64 = continentalness.abs().powf(0.05) * continental_mask;
                //let continentalness: f64 = continentalness;

                let continentalness: f32 = continentalness.signum() * continentalness.abs().powf(0.75); 

                let mut elevation: f32 = 0.0;

                //sharp mountains
                elevation += 750.0 * (get_multi_octave_map(0.0005, voxel_position, 0.0,noise)).abs().powf(3.0);
                //hill
                elevation += 500.0 * (get_multi_octave_map(0.0001, voxel_position, 0.0,noise)).abs().powf(2.0);

                elevation = elevation + (continentalness * 150.0);
                elevation = elevation / WORLD_SCALE;


                let humidity = get_multi_octave_map(0.0003, voxel_position, 6362346326.0,noise);
                let temperature = get_multi_octave_map(0.0001, voxel_position, 7456453.0,noise);
                let magic = get_multi_octave_map(0.000025, voxel_position, 7456453.0,noise);

                let biome = pick_biome(elevation, humidity, temperature, magic, continentalness);



                let mut pixel_block = PixelTypes::Air;
                match biome {
                    Biome::Plains => {
                        if (world_y as f32) < elevation - 4.0 {
                            pixel_block = PixelTypes::Stone
                        }else if (world_y as f32) < elevation- 1.0 {
                            pixel_block = PixelTypes::Dirt
                        }else if (world_y as f32) < elevation {
                            pixel_block = PixelTypes::Grass
                        }
                    },
                    Biome::Ocean => {
                        if (world_y as f32) < elevation - 4.0 {
                            pixel_block = PixelTypes::Stone
                        }else if (world_y as f32) < elevation- 1.0 {
                            pixel_block = PixelTypes::Dirt
                        }else if (world_y as f32) < elevation {
                            pixel_block = PixelTypes::Water
                        }
                    },
                    Biome::Desert => {
                        if (world_y as f32) < elevation - 4.0 {
                            pixel_block = PixelTypes::Stone
                        }else if (world_y as f32) < elevation- 1.0 {
                            pixel_block = PixelTypes::Dirt
                        }else if (world_y as f32) < elevation {
                            pixel_block = PixelTypes::Sand
                        }
                    },
                    Biome::SnowyPlains => {
                        if (world_y as f32) < elevation - 4.0 {
                            pixel_block = PixelTypes::Stone
                        }else if (world_y as f32) < elevation- 1.0 {
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

    create_chunk(&noise, (0,0,0));

    loop {
        let mut requests: Vec<_> = Vec::new();

        while let Ok(req) = chunk_generation_request_rx.try_recv() {
            requests.push(req);
            if requests.len() > 4 {
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