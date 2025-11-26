use core::f32;
use std::{collections::HashMap, os::unix::thread, sync::{Arc, mpsc::{Receiver, Sender}}, thread::sleep, time::Instant};
use fastnoise_lite::{FastNoiseLite, NoiseType};
use rayon::{ThreadPoolBuilder, iter::{IntoParallelRefIterator, ParallelIterator}, vec};
use crate::{game::{GameSnapshot, chunk::{self, Chunk}, pixel::PixelTypes, world::WorldData}, utils::VoxelPosition};


pub async fn lighting_update_thread(game_chunk_snapshot_rx : &mut Receiver<Arc<GameSnapshot>>) {
    let mut world_data = WorldData {
        chunks: HashMap::new(),
        pixel_edit_queue: Vec::new(),
        chunk_mesh_updates_needed: HashMap::new(),
        chunks_loading: HashMap::new(),
        chunk_shadows: HashMap::new(),
    };
    
    loop {
        match game_chunk_snapshot_rx.try_recv() {
            Ok(req) => {
                world_data.chunks = req.chunks.clone()
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

        for mesh in &world_data.chunks {
            let mut shadow_data: Vec<u8> = Vec::new();
            for local_y in 0..16 {
                for local_z in 0..16 {
                    for local_x in 0..16 {
                        let world_pixel_x = (mesh.0.0 * 16) + local_x;
                        let world_pixel_y = (mesh.0.1 * 16) + local_y;
                        let world_pixel_z = (mesh.0.2 * 16) + local_z;

                        shadow_data.push((local_y * 15) as u8);
                    }
                }
            }
        }

        
    }
}