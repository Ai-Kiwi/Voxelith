use std::{collections::HashMap, sync::Arc, time::Instant};

use egui_wgpu::Renderer;
use wgpu::{Buffer, Texture, TextureView};
use winit::{keyboard::KeyCode, window::Window};

use crate::{render::{RenderFrameThreadPerformanceInfo, camera::{Camera, CameraUniform}, mesh::MeshBuffer, render_frame::gui::GuiInfo}, utils::Vec2};

pub fn get_distance_to_camera_unsquared(camera : &Camera, x : f32, y : f32, z : f32) -> f32 {
    let dx = camera.position.x - x;
    let dy = camera.position.y - y;
    let dz = camera.position.z - z;

    return dx*dx + dy*dy + dz*dz;
}


pub fn create_depth_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth Texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
             | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[wgpu::TextureFormat::Depth32Float],
    })
}


pub struct RenderState {
    //gpu related data
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub is_surface_configured: bool,
    pub opaque_render_pipeline: wgpu::RenderPipeline,
    pub transparent_render_pipeline: wgpu::RenderPipeline,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub window: Arc<Window>,
    pub depth_texture: Texture,
    pub depth_view: TextureView,
    
    //other stuff that is just helped for engine itself
    pub last_frame_time : Instant,
    pub start_time : Instant,
    pub delta_time : f32,
    pub keys_down : HashMap<KeyCode,()>,
    pub keys_pressed : HashMap<KeyCode,()>,
    pub keys_released : HashMap<KeyCode,()>,
    pub mouse_position_delta: Vec2,
    pub mouse_position: Vec2,
    //buffers used, although store game data they are overridden and temporary.
    pub temporary_move_buffer : Buffer, 
    pub mesh_id_upto : u64,
    pub mesh_buffers : Vec<MeshBuffer>,


    //gui related stuff. Also engine
    pub egui_renderer : Renderer,
    pub egui_context : egui::Context,
    pub egui_winit : egui_winit::State,

    //window state
    pub game_selected : bool,
    pub fullscreen : bool,
    pub gui_info : GuiInfo,

    pub performance_info : RenderFrameThreadPerformanceInfo,
}

impl<'a> RenderState {

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            let depth_texture = create_depth_texture(&self.device,width,height);
            let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());


            self.depth_texture = depth_texture;
            self.depth_view = depth_view;
            self.is_surface_configured = true;
        }    
    }

}

