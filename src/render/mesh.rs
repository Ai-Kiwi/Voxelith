use std::{mem, time::Instant};

use wgpu::CommandEncoderDescriptor;

use crate::{render::{self, FreeBufferSpace, LEVEL_1_LOD_DISTANCE, LEVEL_2_LOD_DISTANCE, LEVEL_3_LOD_DISTANCE, LEVEL_4_LOD_DISTANCE, RenderChunkMeshBufferReference, mesh, wgpu::{RenderState, get_distance_to_camera_unsquared}}, utils::{Mesh, Vertex}};

fn create_chunk_mesh(render_state : &mut RenderState, mesh : &Mesh, lod : u8) -> RenderChunkMeshBufferReference {
    let mesh_buffer_space_bytes: u32 = (mesh.vertices.len() * mem::size_of::<Vertex>()) as u32;
   if mesh_buffer_space_bytes == 0 {
        return RenderChunkMeshBufferReference {
            vertex_position: 0,
            vertex_length: 0,
            byte_vertex_position: 0,
            byte_vertex_length: 0,
            lod : lod
        };
   }
   
    let mut offset_bytes : Option<u32> = None;
    for space in &mut render_state.free_mesh_buffer_ranges {
        if space.byte_len >= mesh_buffer_space_bytes {
            offset_bytes = Some(space.byte_start);
            space.byte_start += mesh_buffer_space_bytes;
            space.byte_len -= mesh_buffer_space_bytes;
            break;
        }
    };

    render_state.free_mesh_buffer_ranges.retain(|x| x.byte_len != 0);
    
    match offset_bytes {
        Some(offset_bytes) => {
            render_state.queue.write_buffer(&render_state.chunk_mesh_buffer, offset_bytes.into(), bytemuck::cast_slice(&mesh.vertices));

            return RenderChunkMeshBufferReference {
                vertex_position: offset_bytes / mem::size_of::<Vertex>() as u32,
                vertex_length: mesh.vertices.len() as u32,
                byte_vertex_position: offset_bytes,
                byte_vertex_length: mesh_buffer_space_bytes,
                lod : lod
            };
        },
        None => panic!("failed to load chunk to buffer. Likely out of vram"),
    };
}


pub fn update_render_mesh(render_state : &mut RenderState, chunk_pos : (i32,i32,i32), mesh : Option<&Mesh>, lod : u8) {
    if let Some(chunk_data) = render_state.data.chunk_meshs.get(&chunk_pos) {
        render_state.free_mesh_buffer_ranges.push(FreeBufferSpace {
            byte_start: chunk_data.byte_vertex_position,
            byte_len: chunk_data.byte_vertex_length,
        });
    };
    if let Some(mesh_data) = mesh {
        let full_info = create_chunk_mesh(render_state, &mesh_data, lod);
        render_state.data.chunk_meshs.insert(chunk_pos,full_info);
    }else{
        render_state.data.chunk_meshs.remove(&chunk_pos);
    }
}


pub fn update_meshs(render_state : &mut RenderState) {
    let chunk_handling_started = Instant::now();
    loop {
        let mesh_update = render_state.render_channels.chunk_mesh_update_rx.try_recv();
        match mesh_update {
            Ok(mesh_update) => {
                update_render_mesh(render_state, mesh_update.chunk_pos, mesh_update.mesh.as_ref(), 0);

                render_state.data.chunk_mesh_data.insert(mesh_update.chunk_pos, mesh_update);
                //println!("render mesh update {} {} {}",mesh_update.chunk_pos.0,mesh_update.chunk_pos.1,mesh_update.chunk_pos.2);
            },
            Err(_) => {
                break
            },
        }
        if chunk_handling_started.elapsed().as_millis() > 3 {
            break;
        }
    }

    struct MeshUpdatesToBuffer {
        mesh : Mesh,
        chunk_pos : (i32,i32,i32),
        lod : u8
    }

    //convert meshs to lod
    let mut mesh_update: Vec<MeshUpdatesToBuffer> = Vec::new();
    for mesh in &render_state.data.chunk_meshs {        
        let distance = get_distance_to_camera_unsquared(render_state, mesh.0.0 as f32 * 16.0, mesh.0.1 as f32 * 16.0, mesh.0.2 as f32 * 16.0);
        if distance > LEVEL_4_LOD_DISTANCE*LEVEL_4_LOD_DISTANCE {
            //should unload but i haven't sorted that yet
        }else if distance > LEVEL_3_LOD_DISTANCE*LEVEL_3_LOD_DISTANCE {
            if mesh.1.lod != 8 {
                if let Some(mesh_data_obj) = render_state.data.chunk_mesh_data.get(mesh.0) {
                    if let Some(mesh_data) = &mesh_data_obj.mesh_l8 {
                        mesh_update.push(MeshUpdatesToBuffer { 
                            mesh: mesh_data.clone(), 
                            chunk_pos: mesh.0.clone(),
                            lod: 8 
                        });
                    }
                }
            }
        }else if distance > LEVEL_2_LOD_DISTANCE*LEVEL_2_LOD_DISTANCE {
            if mesh.1.lod != 4 {
                if let Some(mesh_data_obj) = render_state.data.chunk_mesh_data.get(mesh.0) {
                    if let Some(mesh_data) = &mesh_data_obj.mesh_l4 {
                        mesh_update.push(MeshUpdatesToBuffer { 
                            mesh: mesh_data.clone(), 
                            chunk_pos: mesh.0.clone(),
                            lod: 4, 
                        });
                    }
                }
            }
        }else if distance > LEVEL_1_LOD_DISTANCE*LEVEL_1_LOD_DISTANCE {
            if mesh.1.lod != 2 {
                if let Some(mesh_data_obj) = render_state.data.chunk_mesh_data.get(mesh.0) {
                    println!("1");
                    if let Some(mesh_data) = &mesh_data_obj.mesh_l2 {
                        println!("2");
                        mesh_update.push(MeshUpdatesToBuffer { 
                            mesh: mesh_data.clone(), 
                            chunk_pos: mesh.0.clone(),
                            lod: 2, 
                        });
                    }
                }
            }
        }else {
            if mesh.1.lod != 1 {
                if let Some(mesh_data_obj) = render_state.data.chunk_mesh_data.get(mesh.0) {
                    if let Some(mesh_data) = &mesh_data_obj.mesh {
                        mesh_update.push(MeshUpdatesToBuffer { 
                            mesh: mesh_data.clone(), 
                            chunk_pos: mesh.0.clone(),
                            lod: 1, 
                        });
                    }
                }
            }
        }
    }
    for mesh in mesh_update {
        update_render_mesh(render_state, mesh.chunk_pos, Some(&mesh.mesh), mesh.lod);
    }

    if chunk_handling_started.elapsed().as_millis() > 5 {
        println!("more then safe amount of time has been spent on lod math");
    }
}


pub fn chunk_buffer_cleanup(render_state : &mut RenderState) {
    let chunk_cleanup_started = Instant::now();

    //delete next to each other 
    for i in 0..render_state.free_mesh_buffer_ranges.len() {
        if render_state.free_mesh_buffer_ranges.len() > i + 1 {
            //run merge test for free space
            if render_state.free_mesh_buffer_ranges[i].byte_start + render_state.free_mesh_buffer_ranges[i].byte_len == render_state.free_mesh_buffer_ranges[i + 1].byte_start {
                render_state.free_mesh_buffer_ranges[i].byte_len += render_state.free_mesh_buffer_ranges[i + 1].byte_len;
                render_state.free_mesh_buffer_ranges.remove(i+1);
            }
        }
    }

    //move items to clean up gaps
    let mut chunk_list: Vec<_> = render_state.data.chunk_meshs.iter_mut().collect();
    chunk_list.retain(|x| x.1.byte_vertex_length != 0);
    chunk_list.sort_by_key(|a| a.1.byte_vertex_position);

    let mut command_encoder = render_state.device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Chunk buffer defrag"),
    });

    let mut next_chunk_pos = 0;
    for chunk in chunk_list {
        //println!("chunk at {} ({} {} {}) size of {}",chunk.1.byte_vertex_position, chunk.0.0,chunk.0.1,chunk.0.2,chunk.1.byte_vertex_length);
        if next_chunk_pos != chunk.1.byte_vertex_position {
            //find the free spot for this
            let mut i = 0;
            'space_test : for free_space in &mut render_state.free_mesh_buffer_ranges {
                if free_space.byte_start == next_chunk_pos {
                    //write data to temp buffer
                    command_encoder.copy_buffer_to_buffer(
                        &render_state.chunk_mesh_buffer, 
                         chunk.1.byte_vertex_position as u64, 
                        &render_state.temporary_move_buffer, 
                        0, 
                        Some(chunk.1.byte_vertex_length as u64),
                    );
                    //write data back in new place
                    command_encoder.copy_buffer_to_buffer(
                        &render_state.temporary_move_buffer, 
                         0, 
                        &render_state.chunk_mesh_buffer, 
                        free_space.byte_start as u64, 
                        Some(chunk.1.byte_vertex_length as u64),
                    );


                    //save changes to chunk info
                    chunk.1.byte_vertex_position = free_space.byte_start;
                    chunk.1.vertex_position = free_space.byte_start / mem::size_of::<Vertex>() as u32;

                    //save changes to byte info
                    free_space.byte_start += chunk.1.byte_vertex_length;
                    break 'space_test;
                }
                i+=1;

            }
            if render_state.free_mesh_buffer_ranges.len() > i + 1 {
                //run merge test for free space
                if render_state.free_mesh_buffer_ranges[i].byte_start + render_state.free_mesh_buffer_ranges[i].byte_len == render_state.free_mesh_buffer_ranges[i + 1].byte_start {
                    render_state.free_mesh_buffer_ranges[i].byte_len += render_state.free_mesh_buffer_ranges[i + 1].byte_len;
                    render_state.free_mesh_buffer_ranges.remove(i+1);
                }
            }


        }
        next_chunk_pos = chunk.1.byte_vertex_length + chunk.1.byte_vertex_position; //will go 1 larger then te amount which is expected. As it is 0 based

        if chunk_cleanup_started.elapsed().as_millis() > 3 {
            break;
        }
    }

    let command_buffer = command_encoder.finish();
    render_state.queue.submit(Some(command_buffer));



    //remove buffers that are empty
    //render_state.free_mesh_buffer_ranges.retain(|x| x.byte_len != 0);
}