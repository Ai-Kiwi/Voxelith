use cgmath::Point3;
use wgpu::CommandEncoder;

use crate::{render::{render_frame::render_world::RenderFrameObjects, wgpu::RenderState}, utils::Vec3};

pub fn render_sun_shadows(render_state : &mut RenderState, encoder : &mut CommandEncoder, render_frame_objects : &RenderFrameObjects) {
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
    
        //draw terrain
        for (i, _draw_call) in render_frame_objects.terrain.iter().enumerate() {
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
        //draw entities
        for mesh in &render_frame_objects.entity_instances {
            let buffer_info = render_state.mesh_instances.get(&mesh.0).unwrap();
            let vertex_info = render_state.mesh_id_reference.get(&mesh.0).expect(format!("Failed to render entity as texture id {} is not loaded", mesh.0.0).as_str());

            sun_shadow_render_pass.set_vertex_buffer(0, render_state.entity_meshs_buffer.slice(..));
            sun_shadow_render_pass.set_vertex_buffer(1, buffer_info.instances_buffer.slice(..));

            sun_shadow_render_pass.draw(vertex_info.start..(vertex_info.start+vertex_info.length), 0..(mesh.1.len() as u32));
        }
        drop(sun_shadow_render_pass)
    }
}