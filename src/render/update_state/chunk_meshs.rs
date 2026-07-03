use std::{collections::HashMap, sync::Arc};

use winit::dpi::Position;

use crate::{game::{chunk::Chunk, lose_terrain::LoseTerrainId}, render::{mesh::{self, create_gpu_mesh}, wgpu::{ChunkInfo, ChunkListInfo, LoseObjectInfo, RenderState}}, utils::{Mesh, Vec3}};

fn get_chunk_list(render_state : &mut RenderState, lose_object_id : Option<LoseTerrainId>) -> &mut ChunkListInfo {
    let chunk_list : &mut ChunkListInfo = if let Some(some_lose_object_id) = lose_object_id {
        let mut lose_object: Option<usize> = render_state.lose_objects_loc.get(&some_lose_object_id).copied();
        if lose_object.is_none() {
            let new_object_index = render_state.lose_objects.len();
            render_state.lose_objects.push(LoseObjectInfo {
                position: Vec3::new(0.0, 0.0, 0.0),
                id: some_lose_object_id,
                chunks: ChunkListInfo { chunk_meshs: Vec::new(), chunk_meshs_loc: HashMap::new() },
            });
            render_state.lose_objects_loc.insert(some_lose_object_id, new_object_index);

            lose_object = Some(new_object_index);
        }
        &mut render_state.lose_objects.get_mut(lose_object.unwrap()).unwrap().chunks
    } else {
        &mut render_state.chunks
    };
    return chunk_list;
}

pub fn update_render_chunk_mesh(render_state : &mut RenderState, chunk_pos : (i32,i32,i32), mesh : Option<&Mesh>, transparent : bool, lose_object_id : Option<LoseTerrainId>) {
    let key = crate::render::wgpu::ChunkIndexInfo { x: chunk_pos.0, y: chunk_pos.1, z: chunk_pos.2, transparent : transparent};
    if let Some(mesh_data) = mesh && mesh_data.vertices.len() > 0 {
        let reference = create_gpu_mesh(render_state, &mesh_data);
        let chunk_info = ChunkInfo {
            buffer_number: reference.buffer_number.clone(),
            pointer: reference.clone(),
            size : mesh_data.vertices.len(),
            index_info: key.clone(),
            lose_object_id,
        };

        let chunk_list: &mut ChunkListInfo = get_chunk_list(render_state, lose_object_id);

        let old_chunk_loc: Option<&usize> = chunk_list.chunk_meshs_loc.get(&key);
        match old_chunk_loc {
            Some(old_loc) => {
                chunk_list.chunk_meshs[*old_loc] = chunk_info;
            },
            None => {
                chunk_list.chunk_meshs_loc.insert(chunk_info.index_info, chunk_list.chunk_meshs.len());
                chunk_list.chunk_meshs.push(chunk_info);
            },
        }
    }else{
        let chunk_list = get_chunk_list(render_state, lose_object_id);
        let old_chunk_loc = chunk_list.chunk_meshs_loc.remove(&key);
        match old_chunk_loc {
            Some(old_loc) => {
                chunk_list.chunk_meshs.swap_remove(old_loc);
                let old_replaced = chunk_list.chunk_meshs.get_mut(old_loc);
                match old_replaced {
                    Some(replaced) => {
                        chunk_list.chunk_meshs_loc.insert(replaced.index_info, old_loc);
                    },
                    None => (),
                }
            },
            None => (),
        }
    }
}


pub fn update_chunk_meshs(render_state : &mut RenderState ) {
    loop {
        let mesh_update = render_state.render_channels.chunk_mesh_update_rx.try_recv();
        match mesh_update {
            Ok(mesh_update) => {
                update_render_chunk_mesh(render_state, mesh_update.chunk_pos, mesh_update.mesh.as_ref(), mesh_update.transparent, mesh_update.lose_object_id);
            },
            Err(_) => {
                break
            },
        }
    }
}
