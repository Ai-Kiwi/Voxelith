use std::{collections::HashMap, sync::{Arc, mpsc::Sender}};

use crate::{game::{chunk::Chunk, lose_terrain::LoseTerrain, world::WorldData}, mesh_creation::ChunkMeshCreateRequest};

pub fn handle_chunk_mesh_updates(world : &mut WorldData, lose_terrain : &mut LoseTerrain , request_chunk_mesh_update_tx : &Sender<ChunkMeshCreateRequest>) {
    for chunk_mesh_updates in &mut world.chunk_mesh_updates_needed {
        if let Some(chunk) = world.chunks.get(chunk_mesh_updates.0) {
            let _ = request_chunk_mesh_update_tx.send(ChunkMeshCreateRequest {
                chunk: Some(chunk.clone()),
                position: *chunk_mesh_updates.0,
                x_positive_chunk_neighbor: world.chunks.get(&(chunk_mesh_updates.0.0 + 1,chunk_mesh_updates.0.1,chunk_mesh_updates.0.2)).cloned(),
                x_negative_chunk_neighbor: world.chunks.get(&(chunk_mesh_updates.0.0 - 1,chunk_mesh_updates.0.1,chunk_mesh_updates.0.2)).cloned(),
                y_positive_chunk_neighbor: world.chunks.get(&(chunk_mesh_updates.0.0,chunk_mesh_updates.0.1 + 1,chunk_mesh_updates.0.2)).cloned(),
                y_negative_chunk_neighbor: world.chunks.get(&(chunk_mesh_updates.0.0,chunk_mesh_updates.0.1 - 1,chunk_mesh_updates.0.2)).cloned(),
                z_positive_chunk_neighbor: world.chunks.get(&(chunk_mesh_updates.0.0,chunk_mesh_updates.0.1,chunk_mesh_updates.0.2 + 1)).cloned(),
                z_negative_chunk_neighbor: world.chunks.get(&(chunk_mesh_updates.0.0,chunk_mesh_updates.0.1,chunk_mesh_updates.0.2 - 1)).cloned(),
                lose_object_id: None,
            });
        }else{
            let _ = request_chunk_mesh_update_tx.send(ChunkMeshCreateRequest {
                chunk: None,
                position: *chunk_mesh_updates.0,
                x_positive_chunk_neighbor: None,
                x_negative_chunk_neighbor: None,
                y_positive_chunk_neighbor: None,
                y_negative_chunk_neighbor: None,
                z_positive_chunk_neighbor: None,
                z_negative_chunk_neighbor: None,
                lose_object_id: None,
            });
        }
    }
    world.chunk_mesh_updates_needed.clear();

    //loop over all lose terrain objects
    for lose_object_id in &lose_terrain.mesh_updated {
        if let Some(lose_object) = lose_terrain.objects.get_mut(lose_object_id) {
            for chunk in &lose_object.chunks_updated {
                let _ = request_chunk_mesh_update_tx.send(ChunkMeshCreateRequest {
                    chunk:  get_chunk(&(chunk.0,chunk.1,chunk.2), &lose_object.chunks, &lose_object.chunks_loc),
                    position: *chunk,
                    x_positive_chunk_neighbor: get_chunk(&(chunk.0 + 1,chunk.1,chunk.2), &lose_object.chunks, &lose_object.chunks_loc),
                    x_negative_chunk_neighbor: get_chunk(&(chunk.0 - 1,chunk.1,chunk.2), &lose_object.chunks, &lose_object.chunks_loc),
                    y_positive_chunk_neighbor: get_chunk(&(chunk.0,chunk.1 + 1,chunk.2), &lose_object.chunks, &lose_object.chunks_loc),
                    y_negative_chunk_neighbor: get_chunk(&(chunk.0,chunk.1 - 1,chunk.2), &lose_object.chunks, &lose_object.chunks_loc),
                    z_positive_chunk_neighbor: get_chunk(&(chunk.0,chunk.1,chunk.2 + 1), &lose_object.chunks, &lose_object.chunks_loc),
                    z_negative_chunk_neighbor: get_chunk(&(chunk.0,chunk.1,chunk.2 - 1), &lose_object.chunks, &lose_object.chunks_loc),
                    lose_object_id: Some(*lose_object_id),
                });
            }
            lose_object.chunks_updated.clear();
        }
    }
    lose_terrain.mesh_updated.clear();
}

fn get_chunk(position : &(i32,i32,i32), vector : &Vec<Arc<Chunk>>, hashmap : &HashMap<(i32,i32,i32),usize>) -> Option<Arc<Chunk>> {
    let chunk_pos = hashmap.get(&position)?;
    return Some(vector[*chunk_pos].clone());
}