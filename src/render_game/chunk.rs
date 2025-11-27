use crate::{render::{mesh::create_gpu_mesh, wgpu::{RenderState, get_distance_to_camera_unsquared}}, render_game::{ChunkInfo, GameData, LEVEL_1_LOD_DISTANCE, LEVEL_2_LOD_DISTANCE, LEVEL_3_LOD_DISTANCE}, utils::Mesh};

pub struct ChunkMeshUpdate {
    pub chunk_pos : (i32,i32,i32),
    pub transparent : bool,
    pub mesh : Option<Mesh>,
    pub mesh_l2 : Option<Mesh>,
    pub mesh_l4 : Option<Mesh>,
    pub mesh_l8 : Option<Mesh>
}

pub struct EntityRenderData {
    //    pub id : u64,
    //    pub alive : bool,
    //    pub position : (f32, f32, f32),
    //    pub entity_type : EntityType,
}

pub fn update_render_chunk_mesh(render_state : &mut RenderState, game_data : &mut GameData, chunk_pos : (i32,i32,i32), mesh : Option<&Mesh>, lod : u8, transparent : bool) {
    let key = (chunk_pos.0,chunk_pos.1,chunk_pos.2,transparent);
    if let Some(mesh_data) = mesh {
        let reference = create_gpu_mesh(render_state, &mesh_data);
        game_data.chunk_meshs.insert(key,ChunkInfo {
            buffer_number: reference.buffer_number.clone(),
            pointer: reference,
            lod : lod,
            size : mesh_data.vertices.len(),
        });
        
    }else{
        game_data.chunk_meshs.remove(&key);
        
    }
}


pub fn update_chunk_meshs(render_state : &mut RenderState, game_data : &mut GameData ) {
    loop {
        let mesh_update = game_data.render_channels.chunk_mesh_update_rx.try_recv();
        match mesh_update {
            Ok(mesh_update) => {
                update_render_chunk_mesh(render_state, game_data, mesh_update.chunk_pos, mesh_update.mesh.as_ref(), 0, mesh_update.transparent);

                game_data.chunk_mesh_data.insert((mesh_update.chunk_pos.0,mesh_update.chunk_pos.1,mesh_update.chunk_pos.2,mesh_update.transparent), mesh_update);
            },
            Err(_) => {
                break
            },
        }
    }

    struct MeshUpdatesToBuffer {
        mesh : Mesh,
        chunk_pos : (i32,i32,i32),
        transparent : bool,
        lod : u8
    }

    //convert meshs to lod
    //let mut mesh_update: Vec<MeshUpdatesToBuffer> = Vec::new();
    //for mesh in &game_data.chunk_meshs {        
    //    let distance = get_distance_to_camera_unsquared(&game_data.camera, mesh.0.0 as f32 * 16.0, mesh.0.1 as f32 * 16.0, mesh.0.2 as f32 * 16.0);
    //    if distance > LEVEL_3_LOD_DISTANCE*LEVEL_3_LOD_DISTANCE {
    //        if mesh.1.lod != 8 {
    //            if let Some(mesh_data_obj) = game_data.chunk_mesh_data.get(&mesh.0) {
    //                if let Some(mesh_data) = &mesh_data_obj.mesh_l8 {
    //                    mesh_update.push(MeshUpdatesToBuffer { 
    //                        mesh: mesh_data.clone(), 
    //                        chunk_pos: (mesh.0.0,mesh.0.1,mesh.0.2),
    //                        lod: 8,
    //                        transparent: mesh.0.3, 
    //                    });
    //                }
    //            }
    //        }
    //    }else if distance > LEVEL_2_LOD_DISTANCE*LEVEL_2_LOD_DISTANCE {
    //        if mesh.1.lod != 4 {
    //            if let Some(mesh_data_obj) = game_data.chunk_mesh_data.get(&mesh.0) {
    //                if let Some(mesh_data) = &mesh_data_obj.mesh_l4 {
    //                    mesh_update.push(MeshUpdatesToBuffer { 
    //                        mesh: mesh_data.clone(), 
    //                        chunk_pos: (mesh.0.0,mesh.0.1,mesh.0.2),
    //                        lod: 4,
    //                        transparent: mesh.0.3,
    //                    });
    //                }
    //            }
    //        }
    //    }else if distance > LEVEL_1_LOD_DISTANCE*LEVEL_1_LOD_DISTANCE {
    //        if mesh.1.lod != 2 {
    //            if let Some(mesh_data_obj) = game_data.chunk_mesh_data.get(&mesh.0) {
    //                if let Some(mesh_data) = &mesh_data_obj.mesh_l2 {
    //                    mesh_update.push(MeshUpdatesToBuffer { 
    //                        mesh: mesh_data.clone(), 
    //                        chunk_pos: (mesh.0.0,mesh.0.1,mesh.0.2),
    //                        lod: 2,
    //                        transparent: mesh.0.3,
    //                    });
    //                }
    //            }
    //        }
    //    }else {
    //        if mesh.1.lod != 1 {
    //            if let Some(mesh_data_obj) = game_data.chunk_mesh_data.get(&mesh.0) {
    //                if let Some(mesh_data) = &mesh_data_obj.mesh {
    //                    mesh_update.push(MeshUpdatesToBuffer { 
    //                        mesh: mesh_data.clone(), 
    //                        chunk_pos: (mesh.0.0,mesh.0.1,mesh.0.2),
    //                        lod: 1,
    //                        transparent: mesh.0.3,
    //                    });
    //                }
    //            }
    //        }
    //    }
    //}
    //for mesh in mesh_update {
    //    update_render_chunk_mesh(render_state, game_data, mesh.chunk_pos, Some(&mesh.mesh), mesh.lod, mesh.transparent);
    //}

}
