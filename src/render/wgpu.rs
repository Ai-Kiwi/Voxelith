use std::{collections::HashMap, rc::Weak, sync::Arc, time::Instant};

use bytemuck::{Pod, Zeroable};
use egui::Mesh;
use egui_wgpu::Renderer;
use wgpu::{Texture, TextureView};
use winit::{event_loop::ActiveEventLoop, keyboard::KeyCode, monitor, window::Window};

use crate::{render::{GameData, RenderThreadChannels, camera::{Camera, CameraUniform}, init_frame_render, mesh::{FreeBufferSpace, MeshBufferReference}, render_frame::gui::GuiInfo}, utils::Vec2};

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
    pub camera: Camera, //most data stored in game_data that is used
    
    //other stuff that is just helped for engine itself
    pub last_frame_time : Instant,
    pub start_time : Instant,
    pub delta_time : f32,
    pub keys_down : HashMap<KeyCode,()>,
    pub keys_pressed : HashMap<KeyCode,()>,
    pub keys_released : HashMap<KeyCode,()>,
    pub mouse_position_delta: Vec2,
    //buffers used, although store game data they are overridden and temporary.
    pub opaque_indirect_buffer: wgpu::Buffer,
    pub transparent_indirect_buffer: wgpu::Buffer,
    pub opaque_count_buffer: wgpu::Buffer,
    pub transparent_count_buffer: wgpu::Buffer,
    pub temporary_move_buffer: wgpu::Buffer,
    pub mesh_buffer: wgpu::Buffer,
    pub free_mesh_buffer_ranges : Vec<FreeBufferSpace>,
    pub meshs : HashMap<u64,MeshBufferReference>,
    pub mesh_id_upto : u64,


    //gui related stuff. Also engine
    pub egui_renderer : Renderer,
    pub egui_context : egui::Context,
    pub egui_winit : egui_winit::State,

    //window state
    pub game_selected : bool,
    pub fullscreen : bool,
    pub gui_info : GuiInfo,
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
            (KeyCode::F11, true) => {
                self.fullscreen = !self.fullscreen;
                if self.fullscreen {
                    self.window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(self.window.current_monitor())));
                }else{
                    self.window.set_fullscreen(None);
                }
            },
            (KeyCode::Tab, true) => {
                self.game_selected = !self.game_selected;
                if self.game_selected {
                    let _ = self.window.set_cursor_grab(winit::window::CursorGrabMode::Confined).or_else(|_e| self.window.set_cursor_grab(winit::window::CursorGrabMode::Locked));
                    self.window.set_cursor_visible(false);
                }else{
                    let _ = self.window.set_cursor_grab(winit::window::CursorGrabMode::None);
                    self.window.set_cursor_visible(true);
                }
            },
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

}

