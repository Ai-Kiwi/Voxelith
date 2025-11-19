use std::{mem, time::Instant};

use wgpu::CommandEncoderDescriptor;

use crate::{render::{FreeBufferSpace, RenderChunkMeshBufferReference, wgpu::RenderState}, utils::{Mesh, Vertex}};

fn create_chunk_mesh(render_state : &mut RenderState, mesh : &Mesh) -> RenderChunkMeshBufferReference {
    let mesh_buffer_space_bytes: u32 = (mesh.vertices.len() * mem::size_of::<Vertex>()) as u32;
   if mesh_buffer_space_bytes == 0 {
        return RenderChunkMeshBufferReference {
            vertex_position: 0,
            vertex_length: 0,
            byte_vertex_position: 0,
            byte_vertex_length: 0,
            mesh: mesh.clone(),
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
                mesh: mesh.clone(),
            };
        },
        None => panic!("failed to load chunk to buffer. Likely out of vram"),
    };
}



pub fn update_meshs(render_state : &mut RenderState) {
    loop {
        let mesh_update = render_state.render_channels.chunk_mesh_update_rx.try_recv();
        match mesh_update {
            Ok(mesh_update) => {
                
                if let Some(chunk_data) = render_state.data.chunk_meshs.get(&mesh_update.chunk_pos) {
                    render_state.free_mesh_buffer_ranges.push(FreeBufferSpace {
                        byte_start: chunk_data.byte_vertex_position,
                        byte_len: chunk_data.byte_vertex_length,
                    });
                };

                let full_info = create_chunk_mesh(render_state, &mesh_update.mesh);



                render_state.data.chunk_meshs.insert(mesh_update.chunk_pos,full_info);
                //println!("render mesh update {} {} {}",mesh_update.chunk_pos.0,mesh_update.chunk_pos.1,mesh_update.chunk_pos.2);

            },
            Err(_) => {
                break
            },
        }
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