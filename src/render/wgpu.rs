use std::{collections::HashMap, sync::Arc, time::Instant};

use bytemuck::{Pod, Zeroable};
use wgpu::{Texture, TextureView};
use winit::{event_loop::ActiveEventLoop, keyboard::KeyCode, window::Window};

use crate::{render::{FreeBufferSpace, RenderData, RenderThreadChannels, camera::{Camera, CameraUniform}, init_frame_render}, utils::Vec2};

pub fn get_distance_to_camera_unsquared(render_state : &RenderState, x : f32, y : f32, z : f32) -> f32 {
    let dx = render_state.camera.eye.x - x;
    let dy = render_state.camera.eye.y - y;
    let dz = render_state.camera.eye.z - z;

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

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct DrawIndirectArgs {
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
}

struct RenderStats {
    draw_calls: u32,
    triangles: u32,
}

impl RenderStats {
    fn new() -> Self {
        Self { draw_calls: 0, triangles: 0 }
    }
}

pub struct RenderState {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub is_surface_configured: bool,
    pub render_pipeline: wgpu::RenderPipeline,
    pub camera: Camera,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub window: Arc<Window>,
    pub data: RenderData,
    pub render_channels : RenderThreadChannels,
    pub depth_texture: Texture,
    pub depth_view: TextureView,

    pub last_frame_time : Instant,
    pub delta_time : f32,

    pub keys_down : HashMap<KeyCode,()>,
    pub keys_pressed : HashMap<KeyCode,()>,
    pub keys_released : HashMap<KeyCode,()>,

    pub chunk_mesh_buffer: wgpu::Buffer,
    pub free_mesh_buffer_ranges : Vec<FreeBufferSpace>,
    pub indirect_buffer: wgpu::Buffer,
    pub count_buffer: wgpu::Buffer,
    pub temporary_move_buffer: wgpu::Buffer,

    pub mouse_position_delta: Vec2,
}

impl<'a> RenderState {

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            let depth_texture = create_depth_texture(&self.device,width,height);
            let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
            self.camera.aspect = self.config.width as f32 / self.config.height as f32;


            self.depth_texture = depth_texture;
            self.depth_view = depth_view;
            self.is_surface_configured = true;
        }    
    }
    
    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {
                if is_pressed {
                    if !self.keys_down.contains_key(&code) {
                        self.keys_down.insert(code, ());
                        self.keys_pressed.insert(code, ());
                    }
                }else{
                    self.keys_down.remove(&code);
                    self.keys_released.insert(code, ());
                }
            }
        }
    }

    pub fn update(&mut self) {
        init_frame_render(self);
    }

}

