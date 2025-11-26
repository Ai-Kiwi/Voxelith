use egui::color_picker::Alpha;
use egui_wgpu::ScreenDescriptor;
use wgpu::RenderPassColorAttachment;

use crate::{mesh_creator::MeshCreator, render::{app::PageOpen, wgpu::RenderState}, render_game::GameData};

impl RenderState {
    pub fn render_gui(&mut self, encoder : &mut wgpu::CommandEncoder, view : &mut wgpu::TextureView, page_open : &PageOpen, game_data : &mut Option<GameData>, mesh_creator : &mut Option<MeshCreator>) {
        let input = self.egui_winit.take_egui_input(&self.window);
        let egui_output = self.egui_context.run(input, |ctx| {
            match page_open {
                PageOpen::Game => {
                    
                },
                PageOpen::TitleScreen => {

                },
                PageOpen::MeshCreator => {
                    if let Some(mesh_creator) = mesh_creator {
                        egui::Window::new("Hierarchy").show(&ctx, |ui| {
                            ui.vertical(|ui| {
                                ui.color_edit_button_srgb(&mut mesh_creator.selected_color);
                                ui.label("text");
                                ui.label("text");
                                ui.label("text");
                                ui.label("text");
                            })
                        });
                    }
                },
            }
        });

        self.egui_winit.handle_platform_output(&self.window, egui_output.platform_output);

        let paint_jobs = self.egui_context.tessellate(egui_output.shapes, 1.0);
        let tdelta: egui::TexturesDelta = egui_output.textures_delta;
        // apply texture updates from egui (add/update and free)
        for (id, image_delta) in &tdelta.set {
            self.egui_renderer.update_texture(&self.device, &self.queue, *id, image_delta);
        }

        self.egui_renderer.update_buffers(&self.device, &self.queue, encoder, &paint_jobs, &ScreenDescriptor {
            size_in_pixels: [self.window.inner_size().width,self.window.inner_size().height],
            pixels_per_point: 1.0,
        });

        let mut ui_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render UI pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { 
                    load: wgpu::LoadOp::Load, 
                    store: wgpu::StoreOp::Store 
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        let ui_render_pass: &mut wgpu::RenderPass<'static> = unsafe { std::mem::transmute(&mut ui_render_pass) };



        self.egui_renderer.render(
            ui_render_pass,
            &paint_jobs,
            &ScreenDescriptor {
                size_in_pixels: [self.window.inner_size().width, self.window.inner_size().height],
                pixels_per_point: 1.0,
            },
        );

        //cleanup textures
        for id in &tdelta.free {
            self.egui_renderer.free_texture(id);
        }
    }
}

pub struct GuiInfo {
    
}

impl GuiInfo {
    pub fn new() -> GuiInfo {
        GuiInfo {

        }
    }
}