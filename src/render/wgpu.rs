use std::{collections::HashMap, sync::Arc, time::Instant};

use cgmath::Point3;
use egui_wgpu::Renderer;
use wgpu::{BindGroupLayout, Buffer, Device, Texture, TextureView, util::DeviceExt};
use winit::{keyboard::KeyCode, window::Window};

use crate::{render::{RenderFrameThreadPerformanceInfo, camera::{CameraUniform, OrthographicCamera, PerspectiveCamera}, entity_meshs::{MeshEntityLocationReference, MeshId, MeshInstance, MeshInstanceId}, init::{gbuffer::update_render_state_gbuffer, init_render_state}, mesh::MeshBuffer, render_frame::gui::GuiInfo}, utils::Vec2};

pub fn get_distance_to_camera_unsquared(camera : &PerspectiveCamera, x : f32, y : f32, z : f32) -> f32 {
    let dx = camera.position.x - x;
    let dy = camera.position.y - y;
    let dz = camera.position.z - z;

    return dx*dx + dy*dy + dz*dz;
}

pub struct RenderState {
    //gpu related data
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub is_surface_configured: bool,
    pub gbuffer_render_pipeline : wgpu::RenderPipeline,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub window: Arc<Window>,
    pub depth_texture: Texture,
    pub depth_view: TextureView,
    pub depth_sampler: wgpu::Sampler,
    
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

    //entity instances
    pub free_mesh_instance :  Vec<MeshInstanceId>,
    pub mesh_id_reference : HashMap<MeshId,MeshEntityLocationReference>,
    pub mesh_instance_buffer : wgpu::Buffer,
    pub entity_mesh_buffer : wgpu::Buffer,
    pub mesh_instances: HashMap<MeshInstanceId, MeshInstance>,

    //gui related stuff. Also engine
    pub egui_renderer : Renderer,
    pub egui_context : egui::Context,
    pub egui_winit : egui_winit::State,
    pub performance_info : RenderFrameThreadPerformanceInfo,

    //window state
    pub game_selected : bool,
    pub fullscreen : bool,
    pub gui_info : GuiInfo,

    //rendering buffers
    pub base_color_gbuffer_view: TextureView,
    pub lighting_gbuffer_view: TextureView,
    pub base_color_gbuffer_sampler: wgpu::Sampler,
    pub lighting_gbuffer_sampler: wgpu::Sampler,
    pub gbuffers_bind_group: wgpu::BindGroup,
    pub gbuffers_bind_group_layout: wgpu::BindGroupLayout,
    pub composition_pipeline_layout: wgpu::PipelineLayout,
    pub composition_render_pipeline: wgpu::RenderPipeline,
    pub material_gbuffer_sampler: wgpu::Sampler,
    pub normal_gbuffer_sampler: wgpu::Sampler,
    pub normal_gbuffer_view: TextureView,
    pub material_gbuffer_view: TextureView,

    //shadow related
    pub sun_shadow_lod_0 : SunShadow,
    pub sun_shadow_lod_1 : SunShadow,
    pub sun_shadow_lod_2 : SunShadow,
    pub sun_shadow_lod_3 : SunShadow,
    pub sun_shadow_render_pipeline: wgpu::RenderPipeline,
    pub sun_shadow_textures_bind_group: wgpu::BindGroup,
}

pub struct SunShadow {
    pub camera : OrthographicCamera,
    pub camera_uniform : CameraUniform,
    pub camera_buffer: Buffer,
    pub texture_view: TextureView,
    pub texture_sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
}

impl<'a> RenderState {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<RenderState> {
        return init_render_state(window).await;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            update_render_state_gbuffer(self);
        }    
    }
}

