use std::collections::HashMap;

use cgmath::{Point3, Quaternion, Vector3};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use wgpu::{CommandEncoder, RenderPass, wgc::device::{self, queue}, wgt::DrawIndirectArgs};

use crate::{entity, render::{self, entity_meshs::{MeshId, MeshInstance, MeshInstanceId, MeshInstanceRaw}, mesh, wgpu::RenderState}, render_game::GameData, utils::Vec3};


pub fn render_chunks(render_state : &mut RenderState, game_data : &mut GameData, view: &wgpu::TextureView, encoder : &mut CommandEncoder) {
    let chunks = &game_data.cache_chunk_meshs;
    
    //create draw calls.
    let mut terrain_buffer_draw_calls = Vec::new();
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

        terrain_buffer_draw_calls.push(opaque_indirect_draw_calls);
    }

    //collect entity instances which we want to render
    let mut entity_instances_to_render: HashMap<MeshId,Vec<MeshInstanceId>> = HashMap::new();
    for (i, entity) in &game_data.entities {
        let mesh_id: MeshId = entity.render_mesh_id;
        let instance_id = entity.instance_id;
        match entity_instances_to_render.get_mut(&mesh_id) {
            Some(draw_calls) => {
                draw_calls.push(instance_id);
            },
            None => {
                let mut buffer_calls: Vec<MeshInstanceId> = Vec::new();
                buffer_calls.push(instance_id);
                entity_instances_to_render.insert(mesh_id, buffer_calls);
            },
        }
    }
    //put the updated data into buffers
    for (mesh_id, instance_buffer) in &render_state.mesh_instances {
        if let Some(entity_list) = entity_instances_to_render.get(&mesh_id) {
            let mut buffer_contents: Vec<u8> = Vec::with_capacity(entity_list.len() * size_of::<MeshInstanceRaw>());
            for entity_instance_id in entity_list {
                let instance_data = instance_buffer.mesh_instances.get(entity_instance_id).unwrap();
                buffer_contents.extend_from_slice(bytemuck::bytes_of(&instance_data.to_raw()));
            }
            render_state.queue.write_buffer(&instance_buffer.instances_buffer, 0, &buffer_contents);
        }
    }
    
    //sun shadows textures
    let sun_shadow_items = Vec::from([
        &mut render_state.sun_shadow_lod_0,
        &mut render_state.sun_shadow_lod_1,
        &mut render_state.sun_shadow_lod_2,
        &mut render_state.sun_shadow_lod_3
    ]);

    for sun_shadow in sun_shadow_items {
        sun_shadow.camera.target = Point3::new(render_state.camera_uniform.position[0], render_state.camera_uniform.position[1], render_state.camera_uniform.position[2]);
        sun_shadow.camera.position = Vec3::new(50.0 + render_state.camera_uniform.position[0], 500.0 + render_state.camera_uniform.position[1], 150.0 + render_state.camera_uniform.position[2]);
        sun_shadow.camera_uniform.update_view_proj_ortho(&mut sun_shadow.camera);
        render_state.queue.write_buffer(&sun_shadow.camera_buffer, 0, bytemuck::cast_slice(&[sun_shadow.camera_uniform]));
        
        let mut sun_shadow_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Sun Shadow Render Pass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { 
                view: &sun_shadow.texture_view, 
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }), 
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        sun_shadow_render_pass.set_pipeline(&render_state.sun_shadow_render_pipeline);
        sun_shadow_render_pass.set_bind_group(0, &sun_shadow.bind_group, &[]);
        
        
        for (i, draw_call) in terrain_buffer_draw_calls.iter().enumerate() {
            render_state.queue.write_buffer(&render_state.mesh_buffers[i].opaque_indirect_buffer, 0, bytemuck::cast_slice(&draw_call));
            render_state.queue.write_buffer(&render_state.mesh_buffers[i].opaque_count_buffer, 0, bytemuck::cast_slice(&[draw_call.len() as u32]));
            sun_shadow_render_pass.set_vertex_buffer(0, render_state.mesh_buffers[i].mesh_buffer.slice(..));
            sun_shadow_render_pass.set_vertex_buffer(1, render_state.blank_instance_info.slice(..));
        
            sun_shadow_render_pass.multi_draw_indirect_count(
                &render_state.mesh_buffers[i].opaque_indirect_buffer, 
                0, 
                &render_state.mesh_buffers[i].opaque_count_buffer,
                0,
                1000000
            );
        }
        drop(sun_shadow_render_pass)
    }
    
    
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
    gbuffer_render_pass.set_bind_group(0, &render_state.camera_bind_group, &[]);
    gbuffer_render_pass.set_bind_group(1, &render_state.sun_shadow_textures_bind_group, &[]);

    //let camera_direction_normal = Vec3::new(
    //    game_data.camera.target.x - game_data.camera.position.x, 
    //    game_data.camera.target.y - game_data.camera.position.y, 
    //    game_data.camera.target.z - game_data.camera.position.z
    //).normalize();

    //render the terrain.

    //render opaque
    for (i, draw_call) in terrain_buffer_draw_calls.iter().enumerate() {
        render_state.queue.write_buffer(&render_state.mesh_buffers[i].opaque_indirect_buffer, 0, bytemuck::cast_slice(&draw_call));
        render_state.queue.write_buffer(&render_state.mesh_buffers[i].opaque_count_buffer, 0, bytemuck::cast_slice(&[draw_call.len() as u32]));
        gbuffer_render_pass.set_vertex_buffer(0, render_state.mesh_buffers[i].mesh_buffer.slice(..));
        gbuffer_render_pass.set_vertex_buffer(1, render_state.blank_instance_info.slice(..));
    
        gbuffer_render_pass.multi_draw_indirect_count(
            &render_state.mesh_buffers[i].opaque_indirect_buffer, 
            0, 
            &render_state.mesh_buffers[i].opaque_count_buffer,
            0,
            1000000
        );
    }

    //render entities
    for mesh in entity_instances_to_render {
        let buffer_info = render_state.mesh_instances.get(&mesh.0).unwrap();
        let vertex_info = render_state.mesh_id_reference.get(&mesh.0).expect(format!("Failed to render entity as texture id {} is not loaded", mesh.0.0).as_str());

        gbuffer_render_pass.set_vertex_buffer(0, render_state.entity_meshs_buffer.slice(..));
        gbuffer_render_pass.set_vertex_buffer(1, buffer_info.instances_buffer.slice(..));

        gbuffer_render_pass.draw(vertex_info.start..(vertex_info.start+vertex_info.length), 0..(mesh.1.len() as u32));
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
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
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
    composition_render_pass.set_bind_group(1, &render_state.camera_bind_group, &[]);
    composition_render_pass.draw(0..3, 0..1);

    drop(composition_render_pass);

}

    