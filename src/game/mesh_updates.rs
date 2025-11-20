use std::sync::mpsc::Sender;

use crate::{game::world::WorldData, mesh_creation::ChunkMeshCreateRequest};

pub fn handle_chunk_mesh_updates(world : &mut WorldData, request_chunk_mesh_update_tx : &Sender<ChunkMeshCreateRequest>) {
    for chunk_mesh_updates in &mut world.chunk_mesh_updates_needed {
    if let Some(chunk) = world.chunks.get(chunk_mesh_updates.0) {
        let _ = request_chunk_mesh_update_tx.send(ChunkMeshCreateRequest {
            chunk: chunk.clone(),
            position: *chunk_mesh_updates.0,
            x_positive_chunk_neighbor: world.chunks.get(&(chunk_mesh_updates.0.0 + 1,chunk_mesh_updates.0.1,chunk_mesh_updates.0.2)).cloned(),
            x_negative_chunk_neighbor: world.chunks.get(&(chunk_mesh_updates.0.0 - 1,chunk_mesh_updates.0.1,chunk_mesh_updates.0.2)).cloned(),
            y_positive_chunk_neighbor: world.chunks.get(&(chunk_mesh_updates.0.0,chunk_mesh_updates.0.1 + 1,chunk_mesh_updates.0.2)).cloned(),
            y_negative_chunk_neighbor: world.chunks.get(&(chunk_mesh_updates.0.0,chunk_mesh_updates.0.1 - 1,chunk_mesh_updates.0.2)).cloned(),
            z_positive_chunk_neighbor: world.chunks.get(&(chunk_mesh_updates.0.0,chunk_mesh_updates.0.1,chunk_mesh_updates.0.2 + 1)).cloned(),
            z_negative_chunk_neighbor: world.chunks.get(&(chunk_mesh_updates.0.0,chunk_mesh_updates.0.1,chunk_mesh_updates.0.2 - 1)).cloned(),
        });
        }
    }
    world.chunk_mesh_updates_needed.clear();
}