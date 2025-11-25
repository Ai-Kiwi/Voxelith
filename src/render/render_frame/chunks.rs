use wgpu::{RenderPass, wgt::DrawIndirectArgs};

use crate::{render::wgpu::RenderState, utils::Vec3};

impl RenderState {
    pub fn render_chunks(&mut self, render_pass : &mut RenderPass<'_>) {
        render_pass.set_pipeline(&self.opaque_render_pipeline);
        

        let camera_direction_normal = Vec3::new(
            self.camera.target.x - self.camera.eye.x, 
            self.camera.target.y - self.camera.eye.y, 
            self.camera.target.z - self.camera.eye.z
        ).normalize();

        #[cfg(feature = "perf_logs")]
        let setup_start = Instant::now();

        //render the terrain.

        //setup opaque
        let mut opaque_indirect_draw_calls = Vec::new();

        for mesh in &self.data.chunk_meshs {
            if mesh.1.vertex_length > 0 && mesh.0.3 == false {
                //let camera_chunk_normal = Vec3::new(
                //    (mesh.0.0 * 16 + 8) as f32 - self.camera.eye.x,
                //    (mesh.0.1 * 16 + 8) as f32 - self.camera.eye.y, 
                //    (mesh.0.2 * 16 + 8) as f32 - self.camera.eye.z
                //).normalize();
            
                //let cos_angle = camera_chunk_normal.dot(&camera_direction_normal);
                //if cos_angle < 0.5 { //around 60 degrees
                //    continue;
                //}

                opaque_indirect_draw_calls.push(DrawIndirectArgs {
                        vertex_count: mesh.1.vertex_length,
                        instance_count: 1,
                        first_vertex: mesh.1.vertex_position,
                        first_instance: 0,
                });
            }
        }

        //setup transparent
        let mut transparent_indirect_draw_calls = Vec::new();

        for mesh in &self.data.chunk_meshs {
            if mesh.1.vertex_length > 0 && mesh.0.3 == true {
                //let camera_chunk_normal = Vec3::new(
                //    (mesh.0.0 * 16 + 8) as f32 - self.camera.eye.x,
                //    (mesh.0.1 * 16 + 8) as f32 - self.camera.eye.y, 
                //    (mesh.0.2 * 16 + 8) as f32 - self.camera.eye.z
                //).normalize();
            
                //let cos_angle = camera_chunk_normal.dot(&camera_direction_normal);
                //if cos_angle < 0.5 { //around 60 degrees
                //    continue;
                //}

                transparent_indirect_draw_calls.push(DrawIndirectArgs {
                        vertex_count: mesh.1.vertex_length,
                        instance_count: 1,
                        first_vertex: mesh.1.vertex_position,
                        first_instance: 0,
                });
            }
        }

        //render opaque
        self.queue.write_buffer(&self.opaque_indirect_buffer, 0, bytemuck::cast_slice(&opaque_indirect_draw_calls));
        self.queue.write_buffer(&self.opaque_count_buffer, 0, bytemuck::cast_slice(&[opaque_indirect_draw_calls.len() as u32]));
        render_pass.set_vertex_buffer(0, self.chunk_mesh_buffer.slice(..));
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

        render_pass.multi_draw_indirect_count(
            &self.opaque_indirect_buffer, 
            0, 
            &self.opaque_count_buffer,
            0,
            1000000
        );

        //render transparent
        render_pass.set_pipeline(&self.transparent_render_pipeline);
        self.queue.write_buffer(&self.transparent_indirect_buffer, 0, bytemuck::cast_slice(&transparent_indirect_draw_calls));
        self.queue.write_buffer(&self.transparent_count_buffer, 0, bytemuck::cast_slice(&[transparent_indirect_draw_calls.len() as u32]));
        render_pass.set_vertex_buffer(0, self.chunk_mesh_buffer.slice(..));
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        
        render_pass.multi_draw_indirect_count(
            &self.transparent_indirect_buffer, 
            0, 
            &self.transparent_count_buffer,
            0,
            1000000
        );
    }
}
    