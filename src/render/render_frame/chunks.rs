use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use wgpu::{CommandEncoder, wgt::DrawIndirectArgs};

use crate::render::{render_frame::render_world::RenderFrameObjects, wgpu::RenderState};

pub fn create_chunk_draw_calls(render_state : &mut RenderState) -> (Vec<Vec<DrawIndirectArgs>>, Vec<Vec<DrawIndirectArgs>>) {
    let chunks = &render_state.chunk_meshs;

    let mut terrain_buffer_draw_calls = Vec::new();
    for (i, buffer) in render_state.mesh_buffers.iter().enumerate() {
        let meshs = &buffer.meshs;

        let opaque_indirect_draw_calls: Vec<DrawIndirectArgs> = chunks
        .par_iter()
        .filter(|mesh| mesh.position.3 == false && mesh.size > 0 && mesh.buffer_number == i)
        .map(|chunk| {
            let id = chunk.pointer.id;
            let mesh_info = meshs.get(&id).unwrap();
            DrawIndirectArgs {
                vertex_count: mesh_info.vertex_length,
                instance_count: 1,
                first_vertex: mesh_info.vertex_position,
                first_instance: 0,
            }
        })
        .collect();

        terrain_buffer_draw_calls.push(opaque_indirect_draw_calls);
    }
    for (i, draw_call) in terrain_buffer_draw_calls.iter().enumerate() {
        render_state.queue.write_buffer(&render_state.mesh_buffers[i].opaque_indirect_buffer, 0, bytemuck::cast_slice(&draw_call));
        render_state.queue.write_buffer(&render_state.mesh_buffers[i].opaque_count_buffer, 0, bytemuck::cast_slice(&[draw_call.len() as u32]));
    }
    let mut transparent_terrain_buffer_draw_calls: Vec<Vec<DrawIndirectArgs>> = Vec::new();
    for (i, buffer) in render_state.mesh_buffers.iter().enumerate() {
        let meshs = &buffer.meshs;

        let mut transparent_indirect_draw_calls: Vec<(DrawIndirectArgs, f32)> = chunks
        .par_iter()
        .filter(|mesh| mesh.position.3 == true && mesh.size > 0 && mesh.buffer_number == i)
        .map(|chunk| {
            let id = chunk.pointer.id;
            let mesh_info = meshs.get(&id).unwrap();

            let chunk_pos = chunk.position; // Assuming chunk.0.0 is position
            let dx = (chunk_pos.0 as f32 * 16.0) - render_state.camera.position.x;
            let dy = (chunk_pos.0 as f32 * 16.0) - render_state.camera.position.y;
            let dz = (chunk_pos.0 as f32 * 16.0) - render_state.camera.position.z;
            let dist_sq = dx*dx + dy*dy + dz*dz;

            let draw_call = DrawIndirectArgs {
                vertex_count: mesh_info.vertex_length,
                instance_count: 1,
                first_vertex: mesh_info.vertex_position,
                first_instance: 0,
            };

            (draw_call, dist_sq)
        })
        .collect();

        transparent_indirect_draw_calls.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        transparent_terrain_buffer_draw_calls.push(transparent_indirect_draw_calls.into_iter().map(|(draw, _)| draw).collect());
    }
    for (i, draw_call) in transparent_terrain_buffer_draw_calls.iter().enumerate() {
        render_state.queue.write_buffer(&render_state.mesh_buffers[i].transparent_indirect_buffer, 0, bytemuck::cast_slice(&draw_call));
        render_state.queue.write_buffer(&render_state.mesh_buffers[i].transparent_count_buffer, 0, bytemuck::cast_slice(&[draw_call.len() as u32]));
    }
    return (terrain_buffer_draw_calls, transparent_terrain_buffer_draw_calls);
}

pub fn render_opaque_chunks(render_state : &mut RenderState, render_frame_objects : &RenderFrameObjects, render_pass : &mut wgpu::RenderPass<'_>) {
    for (i, draw_call) in render_frame_objects.terrain.iter().enumerate() {
        render_state.queue.write_buffer(&render_state.mesh_buffers[i].opaque_indirect_buffer, 0, bytemuck::cast_slice(&draw_call));
        render_state.queue.write_buffer(&render_state.mesh_buffers[i].opaque_count_buffer, 0, bytemuck::cast_slice(&[draw_call.len() as u32]));
        render_pass.set_vertex_buffer(0, render_state.mesh_buffers[i].mesh_buffer.slice(..));
        render_pass.set_vertex_buffer(1, render_state.blank_instance_info.slice(..));
    
        render_pass.multi_draw_indirect_count(
            &render_state.mesh_buffers[i].opaque_indirect_buffer, 
            0, 
            &render_state.mesh_buffers[i].opaque_count_buffer,
            0,
            1000000
        );
    }
}

pub fn render_transparent_chunks(render_state : &mut RenderState, render_frame_objects : &RenderFrameObjects, render_pass : &mut wgpu::RenderPass<'_>) {
    for (i, draw_call) in render_frame_objects.transparent_terrain.iter().enumerate() {
        render_pass.set_vertex_buffer(0, render_state.mesh_buffers[i].mesh_buffer.slice(..));
        render_pass.set_vertex_buffer(1, render_state.blank_instance_info.slice(..));
    
        render_pass.multi_draw_indirect_count(
            &render_state.mesh_buffers[i].transparent_indirect_buffer, 
            0, 
            &render_state.mesh_buffers[i].transparent_count_buffer,
            0,
            1000000
        );
    }
}