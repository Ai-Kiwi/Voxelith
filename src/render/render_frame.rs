use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use wgpu::wgt::DrawIndirectArgs;

use crate::{render::{LEVEL_1_LOD_DISTANCE, wgpu::RenderState}, utils::Vec3};

impl RenderState {
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        #[cfg(feature = "perf_logs")]
        let full_screen_draw_start_time = Instant::now();
        self.window.request_redraw();

        // We can't render unless the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }
                
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Far Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
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
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { 
                view: &self.depth_view, 
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }), 
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        
        render_pass.set_pipeline(&self.render_pipeline);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        
        #[cfg(feature = "perf_logs")]
        let mut stats = RenderStats::new();

        let camera_direction_normal = Vec3::new(
            self.camera.target.x - self.camera.eye.x, 
            self.camera.target.y - self.camera.eye.y, 
            self.camera.target.z - self.camera.eye.z
        ).normalize();

        #[cfg(feature = "perf_logs")]
        let setup_start = Instant::now();

        let mut indirect_draw_calls = Vec::new();


        for mesh in &self.data.chunk_meshs {
            if mesh.1.vertex_length > 0 {
                let camera_chunk_normal = Vec3::new(
                    (mesh.0.0 * 16 + 8) as f32 - self.camera.eye.x,
                    (mesh.0.1 * 16 + 8) as f32 - self.camera.eye.y, 
                    (mesh.0.2 * 16 + 8) as f32 - self.camera.eye.z
                ).normalize();
            
                let cos_angle = camera_chunk_normal.dot(&camera_direction_normal);
                if cos_angle < 0.707 { //around 45 degrees
                    continue;
                }

                indirect_draw_calls.push(DrawIndirectArgs {
                        vertex_count: mesh.1.vertex_length,
                        instance_count: 1,
                        first_vertex: mesh.1.vertex_position,
                        first_instance: 0,
                });
            }
        }

        self.queue.write_buffer(&self.indirect_buffer, 0, bytemuck::cast_slice(&indirect_draw_calls));
        self.queue.write_buffer(&self.count_buffer, 0, bytemuck::cast_slice(&[indirect_draw_calls.len() as u32]));

        render_pass.set_vertex_buffer(0, self.chunk_mesh_buffer.slice(..));
        render_pass.multi_draw_indirect_count(
            &self.indirect_buffer, 
            0, 
            &self.count_buffer,
            0,
            1000000
        );


        #[cfg(feature = "perf_logs")]
        println!("Started setup {}ms",setup_start.elapsed().as_millis());
        #[cfg(feature = "perf_logs")]
        let render_start = Instant::now();
        drop(render_pass);

        #[cfg(feature = "perf_logs")]
        println!("renderer game {}ms",render_start.elapsed().as_millis());
 

        #[cfg(feature = "perf_logs")]
        println!(
            "Draw calls: {}, Triangles: {}",
            stats.draw_calls,
            stats.triangles
        );

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        #[cfg(feature = "perf_logs")]
        println!("full screen draw {}ms",full_screen_draw_start_time.elapsed().as_millis());

        Ok(())
    }
}