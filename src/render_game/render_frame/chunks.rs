use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use wgpu::{RenderPass, wgt::DrawIndirectArgs};

use crate::{render::{camera, mesh, wgpu::RenderState}, render_game::GameData, utils::Vec3};


pub fn render_chunks(render_state : &RenderState, render_pass : &mut RenderPass<'_>, game_data : &mut GameData) {
    render_pass.set_pipeline(&render_state.opaque_render_pipeline);
    

    //let camera_direction_normal = Vec3::new(
    //    game_data.camera.target.x - game_data.camera.position.x, 
    //    game_data.camera.target.y - game_data.camera.position.y, 
    //    game_data.camera.target.z - game_data.camera.position.z
    //).normalize();

    //render the terrain.

    let meshs = &render_state.meshs;

    //setup opaque
    let opaque_indirect_draw_calls: Vec<DrawIndirectArgs> = game_data.chunk_meshs
    .par_iter()
    .filter(|mesh| mesh.0.3 == false && mesh.1.size > 0)
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

    //setup transparent
    let transparent_indirect_draw_calls: Vec<DrawIndirectArgs> = game_data.chunk_meshs
    .par_iter()
    .filter(|mesh| mesh.0.3 == true && mesh.1.size > 0)
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
    render_state.queue.write_buffer(&render_state.opaque_indirect_buffer, 0, bytemuck::cast_slice(&opaque_indirect_draw_calls));
    render_state.queue.write_buffer(&render_state.opaque_count_buffer, 0, bytemuck::cast_slice(&[opaque_indirect_draw_calls.len() as u32]));
    render_pass.set_vertex_buffer(0, render_state.mesh_buffer.slice(..));
    render_pass.set_bind_group(0, &render_state.camera_bind_group, &[]);

    render_pass.multi_draw_indirect_count(
        &render_state.opaque_indirect_buffer, 
        0, 
        &render_state.opaque_count_buffer,
        0,
        1000000
    );

    //render transparent
    render_pass.set_pipeline(&render_state.transparent_render_pipeline);
    render_state.queue.write_buffer(&render_state.transparent_indirect_buffer, 0, bytemuck::cast_slice(&transparent_indirect_draw_calls));
    render_state.queue.write_buffer(&render_state.transparent_count_buffer, 0, bytemuck::cast_slice(&[transparent_indirect_draw_calls.len() as u32]));
    render_pass.set_vertex_buffer(0, render_state.mesh_buffer.slice(..));
    render_pass.set_bind_group(0, &render_state.camera_bind_group, &[]);
    
    render_pass.multi_draw_indirect_count(
        &render_state.transparent_indirect_buffer, 
        0, 
        &render_state.transparent_count_buffer,
        0,
        1000000
    );
}

    