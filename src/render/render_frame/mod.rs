use std::time::Instant;

use crate::{mesh_creator::{MeshCreator, render_mesh_creator}, render::{app::PageOpen, wgpu::RenderState}, render_game::{GameData, render_frame::chunks::render_chunks}};

pub mod gui;

impl RenderState {
    pub fn render(&mut self, page_open : &PageOpen, game_data : &mut Option<GameData>, mesh_creator : &mut Option<MeshCreator>) -> Result<(), wgpu::SurfaceError> {
        let start_render_time = Instant::now();
        self.window.request_redraw();

        // We can't render unless the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }
                
        let output = self.surface.get_current_texture()?;
        let mut view: wgpu::TextureView = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
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
        
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));

        self.performance_info.start_render_time = start_render_time.elapsed().as_secs_f32();
        let main_content_render_start = Instant::now();

        //render chunks
        match page_open {
            PageOpen::Game => {
                if let Some(game_data) = game_data {
                    render_chunks(self, &mut render_pass, game_data);
                }
            },
            PageOpen::TitleScreen => {

            },
            PageOpen::MeshCreator => {
                if let Some(mesh_creator) = mesh_creator {
                    render_mesh_creator(self,&mut render_pass, mesh_creator);
                }
            },
        }

        self.performance_info.main_content_render_time = main_content_render_start.elapsed().as_secs_f32();


        //render the entities


        drop(render_pass);

        //render gui
        let render_gui_start = Instant::now();
        self.render_gui(&mut encoder, &mut view, page_open,  game_data, mesh_creator);
        self.performance_info.render_gui_time = render_gui_start.elapsed().as_secs_f32();

        // submit will accept anything that implements IntoIter
        let render_render_start = Instant::now();
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        self.performance_info.finish_render_time = render_render_start.elapsed().as_secs_f32();

        Ok(())
    }
}