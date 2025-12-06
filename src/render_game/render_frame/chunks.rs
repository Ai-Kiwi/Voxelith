use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use wgpu::{CommandEncoder, RenderPass, wgt::DrawIndirectArgs};

use crate::{render::wgpu::RenderState, render_game::GameData};


pub fn render_chunks(render_state : &RenderState, game_data : &mut GameData, view: &wgpu::TextureView, encoder : &mut CommandEncoder) {
    let mut gbuffer_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[
            Some(wgpu::RenderPassColorAttachment {
                view: &render_state.base_color_gbuffer_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    store: wgpu::StoreOp::Store,
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                },
                depth_slice: None,
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &render_state.lighting_gbuffer_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    store: wgpu::StoreOp::Store,
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                },
                depth_slice: None,
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &render_state.normal_gbuffer_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    store: wgpu::StoreOp::Store,
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                },
                depth_slice: None,
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &render_state.material_gbuffer_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    store: wgpu::StoreOp::Store,
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                },
                depth_slice: None,
            })
        ],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { 
            view: &render_state.depth_view, 
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }), 
            stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
    });

    gbuffer_render_pass.set_pipeline(&render_state.gbuffer_render_pipeline);
    
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
        gbuffer_render_pass.set_vertex_buffer(0, buffer.mesh_buffer.slice(..));
        gbuffer_render_pass.set_bind_group(0, &render_state.camera_bind_group, &[]);
        gbuffer_render_pass.set_bind_group(1, &render_state.voxel_buffer_info_bind_group, &[]);

        
        gbuffer_render_pass.multi_draw_indirect_count(
            &buffer.opaque_indirect_buffer, 
            0, 
            &buffer.opaque_count_buffer,
            0,
            1000000
        );
    }

    drop(gbuffer_render_pass);

    let mut composition_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[
            Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    store: wgpu::StoreOp::Store,
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                },
                depth_slice: None,
            }),
        ],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { 
            view: &render_state.depth_view, 
            depth_ops: None,
            stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
    });


    composition_render_pass.set_pipeline(&render_state.composition_render_pipeline);
    composition_render_pass.set_bind_group(0, &render_state.gbuffers_bind_group, &[]);
    composition_render_pass.draw(0..3, 0..1);

    drop(composition_render_pass);

    //transparent
    //for (i, buffer) in render_state.mesh_buffers.iter().enumerate() {
    //    let meshs = &buffer.meshs;
//
    //    //setup transparent
    //    let transparent_indirect_draw_calls: Vec<DrawIndirectArgs> = chunks
    //    .par_iter()
    //    .filter(|mesh| mesh.0.3 == true && mesh.1.size > 0 && mesh.1.buffer_number == i)
    //    .map(|chunk| {
    //        let id = chunk.1.pointer.id;
    //        let mesh_info = meshs.get(&id).unwrap();
    //        DrawIndirectArgs {
    //            vertex_count: mesh_info.vertex_length,
    //            instance_count: 1,
    //            first_vertex: mesh_info.vertex_position,
    //            first_instance: 0,
    //        }
    //    })
    //    .collect();
    //
    //    //render transparent
    //    render_pass.set_pipeline(&render_state.basic_mesh_render_pipeline);
    //    render_state.queue.write_buffer(&buffer.transparent_indirect_buffer, 0, bytemuck::cast_slice(&transparent_indirect_draw_calls));
    //    render_state.queue.write_buffer(&buffer.transparent_count_buffer, 0, bytemuck::cast_slice(&[transparent_indirect_draw_calls.len() as u32]));
    //    render_pass.set_vertex_buffer(0, buffer.mesh_buffer.slice(..));
    //    render_pass.set_bind_group(0, &render_state.camera_bind_group, &[]);
    //    
    //    render_pass.multi_draw_indirect_count(
    //        &buffer.transparent_indirect_buffer, 
    //        0, 
    //        &buffer.transparent_count_buffer,
    //        0,
    //        1000000
    //    );
    //}

}

    