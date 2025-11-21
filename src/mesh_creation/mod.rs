use crate::{game::chunk::Chunk, mesh_creation::create_mesh::create_chunk_mesh, render::types::ChunkMeshUpdate};

use std::sync::{Arc, mpsc::{Receiver, Sender}};

mod create_mesh;
mod create_triangles;
mod pix_colors;

pub struct ChunkMeshCreateRequest {
    pub chunk : Arc<Chunk>,
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
        let chunk_mesh_create_request = request_chunk_mesh_update_rx.recv();
        match chunk_mesh_create_request {
            Ok(request) => {
                //non transparent
                let mesh = create_chunk_mesh(&request, 1, false);
                let mesh_l2 = create_chunk_mesh(&request, 2, false);
                let mesh_l4 = create_chunk_mesh(&request, 4, false);
                let mesh_l8 = create_chunk_mesh(&request, 8, false);
                let _ = chunk_mesh_update_tx.send(ChunkMeshUpdate { 
                    chunk_pos: request.position, 
                    mesh: Some(mesh),
                    mesh_l2: Some(mesh_l2),
                    mesh_l4: Some(mesh_l4),
                    mesh_l8: Some(mesh_l8),
                    transparent: false,
                });

                //transparent
                let mesh = create_chunk_mesh(&request, 1, true);
                let mesh_l2 = create_chunk_mesh(&request, 2, true);
                let mesh_l4 = create_chunk_mesh(&request, 4, true);
                let mesh_l8 = create_chunk_mesh(&request, 8, true);
                let _ = chunk_mesh_update_tx.send(ChunkMeshUpdate { 
                    chunk_pos: request.position, 
                    mesh: Some(mesh),
                    mesh_l2: Some(mesh_l2),
                    mesh_l4: Some(mesh_l4),
                    mesh_l8: Some(mesh_l8), 
                    transparent: true,
                });
            },
            Err(_) => {},
        }

    }
}