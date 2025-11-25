use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use wgpu::{RenderPass, wgt::DrawIndirectArgs};

use crate::{render::{GameData, wgpu::RenderState}, utils::Vec3};

impl RenderState {
    pub fn render_chunks(&mut self, render_pass : &mut RenderPass<'_>, game_data : &mut GameData) {
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
        let opaque_indirect_draw_calls: Vec<DrawIndirectArgs> = game_data.chunk_meshs
        .par_iter()
        .filter(|mesh| mesh.0.3 == false && mesh.1.size > 0)
        .map(|chunk| {
            let id = chunk.1.pointer.id;
            let mesh_info = self.meshs.get(&id).unwrap();
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
            let mesh_info = self.meshs.get(&id).unwrap();
            DrawIndirectArgs {
                vertex_count: mesh_info.vertex_length,
                instance_count: 1,
                first_vertex: mesh_info.vertex_position,
                first_instance: 0,
            }
        })
        .collect();


        //render opaque
        self.queue.write_buffer(&self.opaque_indirect_buffer, 0, bytemuck::cast_slice(&opaque_indirect_draw_calls));
        self.queue.write_buffer(&self.opaque_count_buffer, 0, bytemuck::cast_slice(&[opaque_indirect_draw_calls.len() as u32]));
        render_pass.set_vertex_buffer(0, self.mesh_buffer.slice(..));
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
        render_pass.set_vertex_buffer(0, self.mesh_buffer.slice(..));
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
    