use std::{mem, sync::{Arc, Weak}, time::Instant};

use dashmap::DashMap;
use wgpu::{CommandEncoderDescriptor, util::DeviceExt};

use crate::{render::{MAP_VRAM_SIZE, wgpu::RenderState}, utils::{Mesh, Vertex}};

pub struct MeshBufferReference {
    pub id : u64,
    pub buffer_number : usize,
    pub byte_vertex_position : u32,
    pub byte_vertex_length : u32,
    pub vertex_position : u32,
    pub vertex_length : u32,
    pub alive_pointer : Weak<GpuMeshReference>
}

pub struct GpuMeshReference {
    pub id : u64,
    pub buffer_number : usize,
}

pub struct FreeBufferSpace {
    pub byte_start: u32,
    pub byte_len: u32,
}

pub struct MeshBuffer {
    pub mesh_buffer: wgpu::Buffer,
    pub free_mesh_buffer_ranges : Vec<FreeBufferSpace>,
    pub meshs : DashMap<u64,MeshBufferReference>,
    pub opaque_indirect_buffer: wgpu::Buffer,
    pub transparent_indirect_buffer: wgpu::Buffer,
    pub opaque_count_buffer: wgpu::Buffer,
    pub transparent_count_buffer: wgpu::Buffer,

    pub stat_percent_mesh_buffer_use : f32,
    pub stat_percent_mesh_buffer_usable : f32,
    pub stat_fragments_mesh_buffer : i32,
    pub stat_bad_fragments_mesh_buffer : i32,
    pub stat_buffer_defragmentation : bool
}

pub fn create_new_mesh_buffer(render_state : &mut RenderState) {
    let mesh_buffer = render_state.device.create_buffer( &wgpu::BufferDescriptor {
        label: Some("Multi chunk buffer"),
        usage: wgpu::BufferUsages::VERTEX
        | wgpu::BufferUsages::COPY_DST
        | wgpu::BufferUsages::STORAGE
        | wgpu::BufferUsages::INDIRECT
        | wgpu::BufferUsages::COPY_SRC,
        size: MAP_VRAM_SIZE,
        mapped_at_creation: false, 
    });

    let opaque_indirect_buffer = render_state.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Indirect Buffer"),
        size: 16 * 1024 * 1024, // 16 MB
        usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let transparent_indirect_buffer = render_state.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Indirect Buffer"),
        size: 16 * 1024 * 1024, // 16 MB
        usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let opaque_count_buffer = render_state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Opaque Count Buffer"),
        contents: bytemuck::cast_slice(&[0 as u32]),
        usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
    });

    let transparent_count_buffer = render_state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Transparent Count Buffer"),
        contents: bytemuck::cast_slice(&[0 as u32]),
        usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
    });

    let mut free_mesh_buffer_ranges = Vec::new();
    free_mesh_buffer_ranges.push(FreeBufferSpace {
        byte_start: 0,
        byte_len: (MAP_VRAM_SIZE as u32) - 1,
    });

    render_state.mesh_buffers.push(MeshBuffer { 
        mesh_buffer, 
        free_mesh_buffer_ranges, 
        meshs: DashMap::new(),
        opaque_indirect_buffer,
        transparent_indirect_buffer,
        opaque_count_buffer,
        transparent_count_buffer,
        stat_percent_mesh_buffer_use: 1.0,
        stat_percent_mesh_buffer_usable: 1.0,
        stat_fragments_mesh_buffer: 0,
        stat_bad_fragments_mesh_buffer: 0,
        stat_buffer_defragmentation: false,
    });
}

pub fn create_gpu_mesh(render_state : &mut RenderState, mesh : &Mesh) -> Arc<GpuMeshReference> {
    let mut buffer_using = 0;
    let mut free_buffer_space = 0.0;
    for i in 0..(render_state.mesh_buffers.len()) {
        let buffer_data = render_state.mesh_buffers.get(i).unwrap();
        if free_buffer_space < buffer_data.stat_percent_mesh_buffer_use {
            free_buffer_space = buffer_data.stat_percent_mesh_buffer_use;
            buffer_using = i;
        }
    }
    if free_buffer_space < 0.15 { //below 15% free total.
        println!("new mesh needed so creating new one");
        create_new_mesh_buffer(render_state);
        buffer_using = render_state.mesh_buffers.len() - 1;
    }
    
    
    let mesh_buffer_space_bytes: u32 = (mesh.vertices.len() * mem::size_of::<Vertex>()) as u32;
    if mesh_buffer_space_bytes == 0 {
        return Arc::new(GpuMeshReference {id : 0, buffer_number: 0 });
    }
   
    let mut offset_bytes : Option<u32> = None;
    for space in &mut render_state.mesh_buffers[buffer_using].free_mesh_buffer_ranges {
        if space.byte_len >= mesh_buffer_space_bytes {
            offset_bytes = Some(space.byte_start);
            space.byte_start += mesh_buffer_space_bytes;
            space.byte_len -= mesh_buffer_space_bytes;
            break;
        }
    };

    render_state.mesh_buffers[buffer_using].free_mesh_buffer_ranges.retain(|x| x.byte_len != 0);
    
    match offset_bytes {
        Some(offset_bytes) => {
            render_state.queue.write_buffer(&render_state.mesh_buffers[buffer_using].mesh_buffer, offset_bytes.into(), bytemuck::cast_slice(&mesh.vertices));

            
            let alive_pointer = Arc::new(GpuMeshReference { id: render_state.mesh_id_upto, buffer_number: buffer_using });
            render_state.mesh_buffers[buffer_using].meshs.insert(render_state.mesh_id_upto,MeshBufferReference {
                vertex_position: offset_bytes / mem::size_of::<Vertex>() as u32,
                vertex_length: mesh.vertices.len() as u32,
                byte_vertex_position: offset_bytes,
                byte_vertex_length: mesh_buffer_space_bytes,
                alive_pointer: Arc::downgrade(&alive_pointer),
                id: render_state.mesh_id_upto,
                buffer_number: buffer_using,
            });
            
            render_state.mesh_id_upto += 1;


            return alive_pointer;
        },
        None => panic!("failed to load mesh to buffer. Likely out of vram"),
    };
}

const MIN_FREE_SPACE_SIZE: u32 =  (0.5 * 1024.0 * 1024.0) as u32;


pub fn mesh_buffer_cleanup(render_state : &mut RenderState, buffer_number : usize) {
    let chunk_cleanup_started = Instant::now();
    let mesh_buffer = render_state.mesh_buffers.get_mut(buffer_number).unwrap();
    //delete dead meshs
    mesh_buffer.meshs.retain(|_key, mesh| {
        let alive = mesh.alive_pointer.strong_count() > 0;
        if alive == false {
            //mesh has been dropped
            mesh_buffer.free_mesh_buffer_ranges.push(FreeBufferSpace {
                byte_start: mesh.byte_vertex_position,
                byte_len: mesh.byte_vertex_length,
            });
        }
        alive
    });

    //delete from actual free spaces anything that has no size
    mesh_buffer.free_mesh_buffer_ranges.retain(|fs| fs.byte_len > 0);
    
    //setup free space info
    let mut free_spaces: Vec<_> = mesh_buffer.free_mesh_buffer_ranges.iter_mut().collect();
    free_spaces.sort_by(|a, b|  a.byte_start.cmp(&b.byte_start));
    
    //delete next to each other and merge into one
    let mut i = 0;
    while i + 1 < free_spaces.len() {
        if free_spaces[i].byte_start + free_spaces[i].byte_len == free_spaces[i + 1].byte_start {
            free_spaces[i].byte_len += free_spaces[i + 1].byte_len;
            free_spaces[i + 1].byte_len = 0;
            free_spaces[i + 1].byte_start = 0;
            free_spaces.remove(i + 1);
        } else {
            i += 1;
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

    //update specs
    mesh_buffer.stat_percent_mesh_buffer_use = free_space as f32 / MAP_VRAM_SIZE as f32;
    mesh_buffer.stat_percent_mesh_buffer_usable = real_free_space as f32 / MAP_VRAM_SIZE as f32;
    mesh_buffer.stat_fragments_mesh_buffer = fragments;
    mesh_buffer.stat_bad_fragments_mesh_buffer = need_resizing_fragments;
    mesh_buffer.stat_buffer_defragmentation = false;

    //not critical so don't bother
    //leaving till later lets huge areas build up as well which can be skipped
    if mesh_buffer.stat_percent_mesh_buffer_usable > 0.10 {
        return;
    }
    mesh_buffer.stat_buffer_defragmentation = true;

    //move items to clean up gaps
    let mut mesh_list: Vec<_> = mesh_buffer.meshs.iter_mut().collect();
    mesh_list.retain(|mesh| mesh.value().byte_vertex_length != 0);
    mesh_list.sort_by_key(|mesh| mesh.value().byte_vertex_position);

    let mut command_encoder = render_state.device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Chunk buffer defrag"),
    });



    let mut next_mesh_pos = 0;
    for mut mesh in mesh_list {
        //println!("{} {}", next_mesh_pos, mesh.1.byte_vertex_position);
        if next_mesh_pos == mesh.value().byte_vertex_position || mesh.value().byte_vertex_position - next_mesh_pos > MIN_FREE_SPACE_SIZE {
            next_mesh_pos = mesh.value().byte_vertex_length + mesh.value().byte_vertex_position;
            continue;
        }

        //find the free spot for this
        'space_test : for free_space in &mut free_spaces {
            if free_space.byte_start != next_mesh_pos {
                continue;
            }
            //write data to temp buffer
            command_encoder.copy_buffer_to_buffer(
                &mesh_buffer.mesh_buffer, 
                    mesh.value().byte_vertex_position as u64, 
                &render_state.temporary_move_buffer, 
                0, 
                Some(mesh.value().byte_vertex_length as u64),
            );
            //write data back in new place
            command_encoder.copy_buffer_to_buffer(
                &render_state.temporary_move_buffer, 
                    0, 
                &mesh_buffer.mesh_buffer, 
                free_space.byte_start as u64, 
                Some(mesh.value().byte_vertex_length as u64),
            );


            //save changes to mesh info
            mesh.value_mut().byte_vertex_position = free_space.byte_start;
            mesh.value_mut().vertex_position = free_space.byte_start / mem::size_of::<Vertex>() as u32;

            //save changes to byte info
            free_space.byte_start += mesh.value().byte_vertex_length;

            break 'space_test;
        }

        //merge all free spaces
        let mut i = 0;
        while i + 1 < free_spaces.len() {
            if free_spaces[i].byte_start + free_spaces[i].byte_len == free_spaces[i + 1].byte_start {
                free_spaces[i].byte_len += free_spaces[i + 1].byte_len;
                free_spaces[i + 1].byte_len = 0;
                free_spaces[i + 1].byte_start = 0;
                free_spaces.remove(i + 1);
            } else {
                i += 1;
            }
        }

        next_mesh_pos = mesh.value().byte_vertex_length + mesh.value().byte_vertex_position; //will go 1 larger then te amount which is expected. As it is 0 based

        //base the limit by how much free room is left in vram
        if chunk_cleanup_started.elapsed().as_secs_f32() > 0.03 / ((real_free_space as f32 / MAP_VRAM_SIZE as f32) * 10.0) {
            break;
        }
    }

    let command_buffer = command_encoder.finish();
    render_state.queue.submit(Some(command_buffer));



    //remove buffers that are empty
    //render_state.free_mesh_buffer_ranges.retain(|x| x.byte_len != 0);
}