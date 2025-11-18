use std::time::Instant;

use wgpu::util::DeviceExt;

use crate::{render::{FullRenderChunkInfo, LEVEL_1_LOD_DISTANCE, LEVEL_2_LOD_DISTANCE, LEVEL_3_LOD_DISTANCE, RenderChunkMesh, wgpu::{RenderState, get_distance_to_camera_unsquared}}, utils::{Mesh, Vertex}};

fn create_chunk_mesh(render_state : &mut RenderState, mesh : &Mesh) -> RenderChunkMesh {
    let vertex_buffer = render_state.device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }
    );
    let index_buffer = render_state.device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        }
    );

    return RenderChunkMesh {
        vertex_buffer: vertex_buffer,
        index_buffer: index_buffer,
        num_indices: mesh.indices.len() as u32,
        mesh_data: mesh.clone(),
    };
}


pub fn create_large_mesh_merge(batch_x : i32, batch_y : i32, batch_z : i32, batch_size : u8, render_state : &mut RenderState) -> RenderChunkMesh {

    let mut chunk_info_upto_indices: Vec<u32> = Vec::new();
    let mut chunk_info_upto_vertices: Vec<Vertex> = Vec::new();

    let range: i32 = batch_size as i32 * 2;

    for x in 0..range {
        for y in 0..range {
            for z in 0..range {
                let chunk_x = batch_x * range + x;
                let chunk_y = batch_y * range + y;
                let chunk_z = batch_z * range + z;

                if let Some(chunk) = render_state.data.chunk_meshs.get(&(chunk_x, chunk_y, chunk_z)) {
                    let offset = chunk_info_upto_vertices.len() as u32;
                    if batch_size == 4 {
                        chunk_info_upto_vertices.extend_from_slice(&chunk.lod_8.mesh_data.vertices);
                        chunk_info_upto_indices.extend(chunk.lod_8.mesh_data.indices.iter().map(|i| i + offset));
                    }else if batch_size == 2 {
                        chunk_info_upto_vertices.extend_from_slice(&chunk.lod_4.mesh_data.vertices);
                        chunk_info_upto_indices.extend(chunk.lod_4.mesh_data.indices.iter().map(|i| i + offset));
                    }else if batch_size == 1 {
                        chunk_info_upto_vertices.extend_from_slice(&chunk.lod_2.mesh_data.vertices);
                        chunk_info_upto_indices.extend(chunk.lod_2.mesh_data.indices.iter().map(|i| i + offset));
                    }else{
                        chunk_info_upto_vertices.extend_from_slice(&chunk.main_lod.mesh_data.vertices);
                        chunk_info_upto_indices.extend(chunk.main_lod.mesh_data.indices.iter().map(|i| i + offset));
                        //println!("trying to update invalid batch level of {}",batch_size)
                    }
                }
            }
        }
    }

    let mesh_object: Mesh = Mesh { 
        vertices: chunk_info_upto_vertices,
        indices: chunk_info_upto_indices,
    };

    let mesh: RenderChunkMesh = create_chunk_mesh(render_state, &mesh_object);
    
    return mesh;
}



pub fn update_meshs(render_state : &mut RenderState) {
    loop {
        let mesh_update = render_state.render_channels.chunk_mesh_update_rx.try_recv();
        match mesh_update {
            Ok(mesh_update) => {
                
                let full_info = FullRenderChunkInfo {
                    main_lod: create_chunk_mesh(render_state,&mesh_update.mesh),
                    lod_2: create_chunk_mesh(render_state,&mesh_update.mesh_l2),
                    lod_4: create_chunk_mesh(render_state,&mesh_update.mesh_l4),
                    lod_8: create_chunk_mesh(render_state,&mesh_update.mesh_l8),
                };


                render_state.data.chunk_meshs.insert(mesh_update.chunk_pos,full_info);
                render_state.data.dirty_chunk_mesh_batches.insert((mesh_update.chunk_pos.0.div_euclid(2),mesh_update.chunk_pos.1.div_euclid(2),mesh_update.chunk_pos.2.div_euclid(2),1), ());
                render_state.data.dirty_chunk_mesh_batches.insert((mesh_update.chunk_pos.0.div_euclid(4),mesh_update.chunk_pos.1.div_euclid(4),mesh_update.chunk_pos.2.div_euclid(4),2), ());
                render_state.data.dirty_chunk_mesh_batches.insert((mesh_update.chunk_pos.0.div_euclid(8),mesh_update.chunk_pos.1.div_euclid(8),mesh_update.chunk_pos.2.div_euclid(8),4), ());
                //println!("render mesh update {} {} {}",mesh_update.chunk_pos.0,mesh_update.chunk_pos.1,mesh_update.chunk_pos.2);

            },
            Err(_) => {
                break
            },
        }
    }

    //update out dated lod
    let chunk_batch_started = Instant::now();
    let chunk_keys: Vec<_> = render_state.data.dirty_chunk_mesh_batches.keys().cloned().collect();
    for chunk_key in chunk_keys {
        let chunk_section_middle_x =  ((chunk_key.0 * 2 * chunk_key.3 as i32).div_euclid(8) * 8 * 16) as f32 + 64.0;
        let chunk_section_middle_y = ((chunk_key.1 * 2 * chunk_key.3 as i32).div_euclid(8) * 8 * 16) as f32 + 64.0;
        let chunk_section_middle_z = ((chunk_key.2 * 2 * chunk_key.3 as i32).div_euclid(8) * 8 * 16) as f32 + 64.0;

        let squared_distance_to_camera = get_distance_to_camera_unsquared(&render_state, chunk_section_middle_x, chunk_section_middle_y, chunk_section_middle_z);

        //let distance_to_camera = get_distance_to_camera(&render_state, chunk_key.0 as f32 * (32.0 * chunk_key.3 as f32), chunk_key.1 as f32 * (32.0 * chunk_key.3 as f32), chunk_key.2 as f32 * (32.0 * chunk_key.3 as f32));
        
        
        let lod_distance_squared = match chunk_key.3 {
            4 => LEVEL_3_LOD_DISTANCE*LEVEL_3_LOD_DISTANCE,
            2 => LEVEL_2_LOD_DISTANCE*LEVEL_2_LOD_DISTANCE,
            1 => LEVEL_1_LOD_DISTANCE*LEVEL_1_LOD_DISTANCE,
            _ => {
                println!("trying to update invalid lod distance batch chunk of {}", chunk_key.3);
                continue;
            }
        };
        
        if squared_distance_to_camera > lod_distance_squared {
            //println!("update chunk batch mesh {} {} {} l={}",chunk_key.0,chunk_key.1,chunk_key.2,chunk_key.3);
            render_state.data.dirty_chunk_mesh_batches.remove(&chunk_key);
            let render_mesh = create_large_mesh_merge(chunk_key.0, chunk_key.1, chunk_key.2, chunk_key.3, render_state);
            render_state.data.chunk_batch_meshs.insert(chunk_key, render_mesh);
        }
        //been more then 3ms so quit this. Helps stop spending to long on it and lagging
        if chunk_batch_started.elapsed().as_millis() > 3 {
            break;
        }
    }

}