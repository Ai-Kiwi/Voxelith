use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{game::chunk::Chunk, mesh_creation::create_mesh::create_chunk_mesh, render_game::chunk::ChunkMeshUpdate};

use std::{sync::{Arc, mpsc::{Receiver, Sender}}, thread::sleep};

mod create_mesh;
pub mod create_triangles;
mod pix_colors;

pub struct ChunkMeshCreateRequest {
    pub chunk : Option<Arc<Chunk>>,
    pub position : (i32,i32,i32),

    pub x_positive_chunk_neighbor : Option<Arc<Chunk>>,
    pub x_negative_chunk_neighbor : Option<Arc<Chunk>>,
    pub y_positive_chunk_neighbor : Option<Arc<Chunk>>,
    pub y_negative_chunk_neighbor : Option<Arc<Chunk>>,
    pub z_positive_chunk_neighbor : Option<Arc<Chunk>>,
    pub z_negative_chunk_neighbor : Option<Arc<Chunk>>
}

pub async fn chunk_mesh_creation_thread(chunk_mesh_update_tx : Sender<ChunkMeshUpdate>, request_chunk_mesh_update_rx : &mut Receiver<ChunkMeshCreateRequest>)  {
    loop { 
        let mut requests: Vec<_> = Vec::new();
        for _ in 0..25 {
            match request_chunk_mesh_update_rx.try_recv() {
                Ok(request) => {
                    requests.push(request);
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
        
        let _ = requests.par_iter().for_each(|request| {
            //non transparent
            let mesh = create_chunk_mesh(&request, 1, false);
            //let mesh_l2 = create_chunk_mesh(&request, 2, false);
            //let mesh_l4 = create_chunk_mesh(&request, 4, false);
            //let mesh_l8 = create_chunk_mesh(&request, 8, false);
            let _ = chunk_mesh_update_tx.send(ChunkMeshUpdate { 
                chunk_pos: request.position, 
                mesh: Some(mesh),
                mesh_l2: None,
                mesh_l4: None,
                mesh_l8: None,
                transparent: false,
                data: request.chunk.clone(),
            });

            //transparent
            let mesh = create_chunk_mesh(&request, 1, true);
            //let mesh_l2 = create_chunk_mesh(&request, 2, true);
            //let mesh_l4 = create_chunk_mesh(&request, 4, true);
            //let mesh_l8 = create_chunk_mesh(&request, 8, true);
            let _ = chunk_mesh_update_tx.send(ChunkMeshUpdate { 
                chunk_pos: request.position, 
                mesh: Some(mesh),
                mesh_l2: None,
                mesh_l4: None,
                mesh_l8: None, 
                transparent: true,
                data: request.chunk.clone(),
            });
        });
    }
}