use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use wgpu::{RenderPass, wgt::DrawIndirectArgs};

use crate::{render::{camera, mesh, wgpu::RenderState}, render_game::GameData, utils::Vec3};


pub fn render_chunks(render_state : &RenderState, render_pass : &mut RenderPass<'_>, game_data : &mut GameData) {
    render_pass.set_pipeline(&render_state.opaque_render_pipeline);
    
    let chunks = &game_data.cache_chunk_meshs;

    //let camera_direction_normal = Vec3::new(
    //    game_data.camera.target.x - game_data.camera.position.x, 
    //    game_data.camera.target.y - game_data.camera.position.y, 
    //    game_data.camera.target.z - game_data.camera.position.z
    //).normalize();

    //render the terrain.

    //opaque
    for (i, buffer) in render_state.mesh_buffers.iter().enumerate() {
        let meshs = &buffer.meshs;

        let opaque_indirect_draw_calls: Vec<DrawIndirectArgs> = chunks
        .par_iter()
        .filter(|mesh| mesh.0.3 == false && mesh.1.size > 0 && mesh.1.buffer_number == i)
        .map(|chunk| {
            let id = chunk.1.pointer.id;
            let mesh_info = meshs.get(&id).unwrap();
            DrawIndirectArgs {
                vertex_count: mesh_info.vertex_length,
                instance_count: 1,
                first_vertex: mesh_info.vertex_position,
                first_instance: 0,
            }
        })
        .collect();
    
        //render opaque
        render_state.queue.write_buffer(&buffer.opaque_indirect_buffer, 0, bytemuck::cast_slice(&opaque_indirect_draw_calls));
        render_state.queue.write_buffer(&buffer.opaque_count_buffer, 0, bytemuck::cast_slice(&[opaque_indirect_draw_calls.len() as u32]));
        render_pass.set_vertex_buffer(0, buffer.mesh_buffer.slice(..));
        render_pass.set_bind_group(0, &render_state.camera_bind_group, &[]);
        
        render_pass.multi_draw_indirect_count(
            &buffer.opaque_indirect_buffer, 
            0, 
            &buffer.opaque_count_buffer,
            0,
            1000000
        );
    }

    //transparent
    for (i, buffer) in render_state.mesh_buffers.iter().enumerate() {
        let meshs = &buffer.meshs;

        //setup transparent
        let transparent_indirect_draw_calls: Vec<DrawIndirectArgs> = chunks
        .par_iter()
        .filter(|mesh| mesh.0.3 == true && mesh.1.size > 0 && mesh.1.buffer_number == i)
        .map(|chunk| {
            let id = chunk.1.pointer.id;
            let mesh_info = meshs.get(&id).unwrap();
            DrawIndirectArgs {
                vertex_count: mesh_info.vertex_length,
                instance_count: 1,
                first_vertex: mesh_info.vertex_position,
                first_instance: 0,
            }
        })
        .collect();
    
        //render transparent
        render_pass.set_pipeline(&render_state.transparent_render_pipeline);
        render_state.queue.write_buffer(&buffer.transparent_indirect_buffer, 0, bytemuck::cast_slice(&transparent_indirect_draw_calls));
        render_state.queue.write_buffer(&buffer.transparent_count_buffer, 0, bytemuck::cast_slice(&[transparent_indirect_draw_calls.len() as u32]));
        render_pass.set_vertex_buffer(0, buffer.mesh_buffer.slice(..));
        render_pass.set_bind_group(0, &render_state.camera_bind_group, &[]);
        
        render_pass.multi_draw_indirect_count(
            &buffer.transparent_indirect_buffer, 
            0, 
            &buffer.transparent_count_buffer,
            0,
            1000000
        );
    }

}

    