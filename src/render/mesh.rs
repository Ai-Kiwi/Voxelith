use std::{mem, sync::{Arc, Weak}};

use log::set_boxed_logger;
use wgpu::CommandEncoderDescriptor;

use crate::{render::{mesh, wgpu::{RenderState, get_distance_to_camera_unsquared}}, render_game::MAP_VRAM_SIZE, utils::{Mesh, Vertex}};

pub struct MeshBufferReference {
    pub id : u64,
    pub byte_vertex_position : u32,
    pub byte_vertex_length : u32,
    pub vertex_position : u32,
    pub vertex_length : u32,
    pub alive_pointer : Weak<GpuMeshReference>
}

pub struct GpuMeshReference {
    pub id : u64
}

pub struct FreeBufferSpace {
    pub byte_start: u32,
    pub byte_len: u32,
}

pub fn create_gpu_mesh(render_state : &mut RenderState, mesh : &Mesh) -> Arc<GpuMeshReference> {
    let mesh_buffer_space_bytes: u32 = (mesh.vertices.len() * mem::size_of::<Vertex>()) as u32;
    if mesh_buffer_space_bytes == 0 {
        return Arc::new(GpuMeshReference {id : 0});
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
            render_state.queue.write_buffer(&render_state.mesh_buffer, offset_bytes.into(), bytemuck::cast_slice(&mesh.vertices));

            
            let alive_pointer = Arc::new(GpuMeshReference { id: render_state.mesh_id_upto });
            render_state.meshs.insert(render_state.mesh_id_upto,MeshBufferReference {
                vertex_position: offset_bytes / mem::size_of::<Vertex>() as u32,
                vertex_length: mesh.vertices.len() as u32,
                byte_vertex_position: offset_bytes,
                byte_vertex_length: mesh_buffer_space_bytes,
                alive_pointer: Arc::downgrade(&alive_pointer),
                id: render_state.mesh_id_upto,
            });
            
            render_state.mesh_id_upto += 1;


            return alive_pointer;
        },
        None => panic!("failed to load mesh to buffer. Likely out of vram"),
    };
}

const MIN_FREE_SPACE_SIZE: u32 =  (0.5 * 1024.0 * 1024.0) as u32;


pub fn mesh_buffer_cleanup(render_state : &mut RenderState) {
    //delete dead meshs
    render_state.meshs.retain(|key, mesh| {
        let alive = mesh.alive_pointer.strong_count() > 0;
        if alive == false {
            //mesh has been dropped
            render_state.free_mesh_buffer_ranges.push(FreeBufferSpace {
                byte_start: mesh.byte_vertex_position,
                byte_len: mesh.byte_vertex_length,
            });
        }
        alive
    });
    
    //setup free space info
    let mut free_spaces: Vec<_> = render_state.free_mesh_buffer_ranges.iter_mut().collect();
    free_spaces.sort_by(|a, b|  a.byte_start.cmp(&b.byte_start));
    
    //delete next to each other and merge into one
    for i in 0..free_spaces.len() {
        if free_spaces.len() > i + 1 {
            //run merge test for free space
            if free_spaces[i].byte_start + free_spaces[i].byte_len == free_spaces[i + 1].byte_start {
                free_spaces[i].byte_len += free_spaces[i + 1].byte_len;
                free_spaces[i + 1].byte_len = 0;
            }
        }
    }
    
    let mut free_space = 0;
    let mut real_free_space = 0;
    let mut fragments = 0;
    let mut need_resizing_fragments = 0;

    for space in &free_spaces {
        free_space += space.byte_len;
        fragments += 1;
        if space.byte_len < MIN_FREE_SPACE_SIZE {
            need_resizing_fragments += 1;
        }else{
            real_free_space += space.byte_len;
        }
    };

    
    //not critical so don't bother
    //leaving till later lets huge areas build up as well which can be skipped
    if need_resizing_fragments < 60000 || (real_free_space as f32 / MAP_VRAM_SIZE as f32) < 0.25 {
        return;
    }
    if (real_free_space as f32 / MAP_VRAM_SIZE as f32) < 0.2 {
        println!("free space {}% usable {}% across {} fragments {} need resizing", ((free_space as f32 / MAP_VRAM_SIZE as f32) * 100.0).round(), ((real_free_space as f32 / MAP_VRAM_SIZE as f32) * 100.0).round(), fragments, need_resizing_fragments);
    }

    //move items to clean up gaps
    let mut mesh_list: Vec<_> = render_state.meshs.iter_mut().collect();
    mesh_list.retain(|mesh| mesh.1.byte_vertex_length != 0);
    mesh_list.sort_by_key(|mesh| mesh.1.byte_vertex_position);

    let mut command_encoder = render_state.device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Chunk buffer defrag"),
    });



    let mut next_mesh_pos = 0;
    for mesh in mesh_list {
        if next_mesh_pos != mesh.1.byte_vertex_position {
            continue;
        }

        //find the free spot for this
        let mut i = 0;
        'space_test : for free_space in &mut free_spaces {
            if free_space.byte_start != next_mesh_pos || free_space.byte_len > MIN_FREE_SPACE_SIZE {
                i+=1;
                continue;
            }
            
            //write data to temp buffer
            command_encoder.copy_buffer_to_buffer(
                &render_state.mesh_buffer, 
                    mesh.1.byte_vertex_position as u64, 
                &render_state.temporary_move_buffer, 
                0, 
                Some(mesh.1.byte_vertex_length as u64),
            );
            //write data back in new place
            command_encoder.copy_buffer_to_buffer(
                &render_state.temporary_move_buffer, 
                    0, 
                &render_state.mesh_buffer, 
                free_space.byte_start as u64, 
                Some(mesh.1.byte_vertex_length as u64),
            );


            //save changes to mesh info
            mesh.1.byte_vertex_position = free_space.byte_start;
            mesh.1.vertex_position = free_space.byte_start / mem::size_of::<Vertex>() as u32;

            //save changes to byte info
            free_space.byte_start += mesh.1.byte_vertex_length;

            break 'space_test;
        }

        //test if need to extend the free buffer to another one if one is in front of block
        if free_spaces.len() > i + 1 {
            //run merge test for free space
            if free_spaces[i].byte_start + free_spaces[i].byte_len == free_spaces[i + 1].byte_start {
                free_spaces[i].byte_len += free_spaces[i + 1].byte_len;
                free_spaces.remove(i+1);
            }
        }

        next_mesh_pos = mesh.1.byte_vertex_length + mesh.1.byte_vertex_position; //will go 1 larger then te amount which is expected. As it is 0 based

        //if chunk_cleanup_started.elapsed().as_millis() > 3 {
        //    break;
        //}
    }

    let command_buffer = command_encoder.finish();
    render_state.queue.submit(Some(command_buffer));



    //remove buffers that are empty
    //render_state.free_mesh_buffer_ranges.retain(|x| x.byte_len != 0);
}