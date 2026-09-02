use cgmath::{Quaternion, Vector3};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use wgpu::{CommandEncoder, wgt::DrawIndirectArgs};

use crate::render::{
    entity_meshs::{MeshInstance, MeshInstanceRaw},
    render_frame::render_world::RenderFrameObjects,
    wgpu::{LoseObjectInfo, RenderState},
};

pub fn create_chunk_draw_calls(
    render_state: &mut RenderState,
) -> (Vec<Vec<DrawIndirectArgs>>, Vec<Vec<DrawIndirectArgs>>) {
    let chunks = &render_state.chunks.chunk_meshs;

    let mut terrain_buffer_draw_calls = Vec::new();
    for (i, buffer) in render_state.mesh_buffers.iter().enumerate() {
        let meshs = &buffer.meshs;

        let opaque_indirect_draw_calls: Vec<DrawIndirectArgs> = chunks
            .par_iter()
            .filter(|mesh| {
                mesh.index_info.transparent == false
                    && mesh.size > 0
                    && mesh.buffer_number == i
                    && mesh.lose_object_id == None
            })
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
    //add lose objects
    let mut current_lose_chunk_object: u32 = 0;
    for object in &render_state.lose_objects {
        current_lose_chunk_object += 1;
        let instance_contents_raw = MeshInstance {
            position: Vector3::new(object.position.x, object.position.y, object.position.z),
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
        }
        .to_raw();
        render_state.queue.write_buffer(
            &render_state.static_and_lose_chunk_instance_info,
            (std::mem::size_of::<MeshInstanceRaw>() as u64) * current_lose_chunk_object as u64,
            bytemuck::bytes_of(&instance_contents_raw),
        );
        for (i, buffer) in render_state.mesh_buffers.iter().enumerate() {
            let meshs = &buffer.meshs;

            let opaque_indirect_draw_calls: Vec<DrawIndirectArgs> = object
                .chunks
                .chunk_meshs
                .par_iter()
                .filter(|mesh| {
                    mesh.index_info.transparent == false
                        && mesh.size > 0
                        && mesh.buffer_number == i
                        && mesh.lose_object_id == Some(object.id)
                })
                .map(|chunk| {
                    let id = chunk.pointer.id;
                    let mesh_info = meshs.get(&id).unwrap();
                    DrawIndirectArgs {
                        vertex_count: mesh_info.vertex_length,
                        instance_count: 1,
                        first_vertex: mesh_info.vertex_position,
                        first_instance: current_lose_chunk_object,
                    }
                })
                .collect();

            terrain_buffer_draw_calls[i].extend(opaque_indirect_draw_calls);
        }
    }
    for (i, draw_call) in terrain_buffer_draw_calls.iter().enumerate() {
        render_state.queue.write_buffer(
            &render_state.mesh_buffers[i].opaque_indirect_buffer,
            0,
            bytemuck::cast_slice(&draw_call),
        );
        render_state.queue.write_buffer(
            &render_state.mesh_buffers[i].opaque_count_buffer,
            0,
            bytemuck::cast_slice(&[draw_call.len() as u32]),
        );
    }

    let mut transparent_terrain_buffer_draw_calls: Vec<Vec<DrawIndirectArgs>> = Vec::new();
    for (i, buffer) in render_state.mesh_buffers.iter().enumerate() {
        let meshs = &buffer.meshs;

        let mut transparent_indirect_draw_calls: Vec<(DrawIndirectArgs, f32)> = chunks
            .par_iter()
            .filter(|mesh| {
                mesh.index_info.transparent == true
                    && mesh.size > 0
                    && mesh.buffer_number == i
                    && mesh.lose_object_id == None
            })
            .map(|chunk| {
                let id = chunk.pointer.id;
                let mesh_info = meshs.get(&id).unwrap();

                let dx = (chunk.index_info.x as f32 * 16.0) - render_state.camera.position.x;
                let dy = (chunk.index_info.y as f32 * 16.0) - render_state.camera.position.y;
                let dz = (chunk.index_info.z as f32 * 16.0) - render_state.camera.position.z;
                let dist_sq = dx * dx + dy * dy + dz * dz;

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

        transparent_terrain_buffer_draw_calls.push(
            transparent_indirect_draw_calls
                .into_iter()
                .map(|(draw, _)| draw)
                .collect(),
        );
    }
    for (i, draw_call) in transparent_terrain_buffer_draw_calls.iter().enumerate() {
        render_state.queue.write_buffer(
            &render_state.mesh_buffers[i].transparent_indirect_buffer,
            0,
            bytemuck::cast_slice(&draw_call),
        );
        render_state.queue.write_buffer(
            &render_state.mesh_buffers[i].transparent_count_buffer,
            0,
            bytemuck::cast_slice(&[draw_call.len() as u32]),
        );
    }

    return (
        terrain_buffer_draw_calls,
        transparent_terrain_buffer_draw_calls,
    );
}

pub fn render_opaque_chunks(
    render_state: &RenderState,
    render_frame_objects: &RenderFrameObjects,
    render_pass: &mut wgpu::RenderPass<'_>,
) {
    for (i, draw_call) in render_frame_objects.terrain.iter().enumerate() {
        render_state.queue.write_buffer(
            &render_state.mesh_buffers[i].opaque_indirect_buffer,
            0,
            bytemuck::cast_slice(&draw_call),
        );
        render_state.queue.write_buffer(
            &render_state.mesh_buffers[i].opaque_count_buffer,
            0,
            bytemuck::cast_slice(&[draw_call.len() as u32]),
        );
        render_pass.set_vertex_buffer(0, render_state.mesh_buffers[i].mesh_buffer.slice(..));
        render_pass.set_vertex_buffer(
            1,
            render_state.static_and_lose_chunk_instance_info.slice(..),
        );

        render_pass.multi_draw_indirect_count(
            &render_state.mesh_buffers[i].opaque_indirect_buffer,
            0,
            &render_state.mesh_buffers[i].opaque_count_buffer,
            0,
            1000000,
        );
    }
}

pub fn render_transparent_chunks(
    render_state: &mut RenderState,
    render_frame_objects: &RenderFrameObjects,
    render_pass: &mut wgpu::RenderPass<'_>,
) {
    for (i, draw_call) in render_frame_objects.transparent_terrain.iter().enumerate() {
        render_pass.set_vertex_buffer(0, render_state.mesh_buffers[i].mesh_buffer.slice(..));
        render_pass.set_vertex_buffer(
            1,
            render_state.static_and_lose_chunk_instance_info.slice(..),
        );

        render_pass.multi_draw_indirect_count(
            &render_state.mesh_buffers[i].transparent_indirect_buffer,
            0,
            &render_state.mesh_buffers[i].transparent_count_buffer,
            0,
            1000000,
        );
    }
}
