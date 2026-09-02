use std::{
    collections::HashMap,
    sync::{
        Arc,
        mpsc::{Receiver, Sender},
    },
    time::Instant,
};

use cgmath::Point3;
use egui::ahash::HashSet;
use egui_wgpu::Renderer;
use wgpu::{BindGroupLayout, Buffer, Device, Texture, TextureView, util::DeviceExt};
use winit::{keyboard::KeyCode, window::Window};

use crate::{
    entity::EntityClass,
    game::{InputEvent, entity::EntityId, lose_terrain::LoseTerrainId},
    render::{
        RenderFrameThreadPerformanceInfo,
        camera::{CameraUniform, OrthographicCamera, PerspectiveCamera},
        entity_meshs::{MeshEntityLocationReference, MeshId, MeshInstance, MeshInstanceId},
        init::{
            entity_meshs::MeshInstancesBufferInfo, gbuffer::update_render_state_gbuffer,
            init_render_state,
        },
        mesh::{GpuMeshReference, MeshBuffer},
        render_frame::gui::GuiInfo,
        update_state::{ChunkMeshUpdate, EntityRenderDataUpdate, LoseObjectRenderDataUpdate},
    },
    utils::{Vec2, Vec3},
};

pub fn get_distance_to_camera_unsquared(camera: &PerspectiveCamera, x: f32, y: f32, z: f32) -> f32 {
    let dx = camera.position.x - x;
    let dy = camera.position.y - y;
    let dz = camera.position.z - z;

    return dx * dx + dy * dy + dz * dz;
}

pub struct RenderState {
    //gpu related data
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub is_surface_configured: bool,
    pub gbuffer_render_pipeline: wgpu::RenderPipeline,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub window: Arc<Window>,
    pub depth_texture: Texture,
    pub depth_view: TextureView,
    pub depth_sampler: wgpu::Sampler,

    //other stuff that is just helped for engine itself
    pub last_frame_time: Instant,
    pub start_time: Instant,
    pub delta_time: f32,
    pub keys_down: HashMap<KeyCode, ()>,
    pub keys_pressed: HashMap<KeyCode, ()>,
    pub keys_released: HashMap<KeyCode, ()>,
    pub mouse_position_delta: Vec2,
    pub mouse_position: Vec2,
    //buffers used, although store game data they are overridden and temporary.
    pub temporary_move_buffer: Buffer,
    pub mesh_id_upto: u64,
    pub mesh_buffers: Vec<MeshBuffer>,

    //entity instances
    pub mesh_id_reference: HashMap<MeshId, MeshEntityLocationReference>,
    pub entity_meshs_buffer: Buffer,
    pub mesh_instances: HashMap<MeshId, MeshInstancesBufferInfo>,
    pub static_and_lose_chunk_instance_info: Buffer,

    //gui related stuff. Also engine
    pub egui_renderer: Renderer,
    pub egui_context: egui::Context,
    pub egui_winit: egui_winit::State,
    pub performance_info: RenderFrameThreadPerformanceInfo,

    //window state
    pub game_selected: bool,
    pub fullscreen: bool,
    pub gui_info: GuiInfo,

    //rendering buffers
    pub base_color_gbuffer_view: TextureView,
    pub lighting_gbuffer_view: TextureView,
    pub base_color_gbuffer_sampler: wgpu::Sampler,
    pub lighting_gbuffer_sampler: wgpu::Sampler,
    pub gbuffers_bind_group: wgpu::BindGroup,
    pub gbuffers_bind_group_layout: wgpu::BindGroupLayout,
    pub composition_pipeline_layout: wgpu::PipelineLayout,
    pub composition_render_pipeline: wgpu::RenderPipeline,
    pub transparent_pipeline_layout: wgpu::PipelineLayout,
    pub transparent_render_pipeline: wgpu::RenderPipeline,
    pub material_gbuffer_sampler: wgpu::Sampler,
    pub normal_gbuffer_sampler: wgpu::Sampler,
    pub normal_gbuffer_view: TextureView,
    pub material_gbuffer_view: TextureView,

    //shadow related
    pub sun_shadow_lod_0: SunShadow,
    pub sun_shadow_lod_1: SunShadow,
    pub sun_shadow_lod_2: SunShadow,
    pub sun_shadow_lod_3: SunShadow,
    pub sun_shadow_render_pipeline: wgpu::RenderPipeline,
    pub sun_shadow_textures_bind_group: wgpu::BindGroup,

    //volumetric lighting related
    pub volumetric_lighting_gbuffer_sampler: wgpu::Sampler,
    pub volumetric_lighting_gbuffer_view: TextureView,
    pub volumetric_lighting_render_pipeline: wgpu::RenderPipeline,
    pub volumetric_lighting_bind_group: wgpu::BindGroup,
    pub volumetric_lighting_bind_group_layout: wgpu::BindGroupLayout,

    //game state related stuff
    pub camera: PerspectiveCamera,
    pub chunks: ChunkListInfo,
    pub render_channels: RenderThreadChannels,
    pub entities: Vec<EntityRenderData>,
    pub entities_loc: HashMap<EntityId, usize>,
    pub lose_objects: Vec<LoseObjectInfo>,
    pub lose_objects_loc: HashMap<LoseTerrainId, usize>,
}

pub struct RenderThreadChannels {
    pub chunk_mesh_update_rx: Receiver<ChunkMeshUpdate>,
    pub entity_render_rx: Receiver<EntityRenderDataUpdate>,
    pub input_event_tx: Sender<InputEvent>,
    pub lose_object_update_rx: Receiver<LoseObjectRenderDataUpdate>,
}

pub struct ChunkListInfo {
    pub chunk_meshs: Vec<ChunkInfo>,
    pub chunk_meshs_loc: HashMap<ChunkIndexInfo, usize>,
}

pub struct ChunkInfo {
    pub pointer: Arc<GpuMeshReference>,
    pub buffer_number: usize,
    pub size: usize,
    pub index_info: ChunkIndexInfo,
    pub lose_object_id: Option<LoseTerrainId>,
}

pub struct LoseObjectInfo {
    pub position: Vec3,
    pub id: LoseTerrainId,
    pub chunks: ChunkListInfo,
}

#[derive(Eq, Hash, PartialEq, Copy, Clone)]
pub struct ChunkIndexInfo {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub transparent: bool,
}

pub struct EntityRenderData {
    pub id: EntityId,
    pub position: Vec3,
    pub entity_class: EntityClass,
    pub render_mesh_id: MeshId, //THIS IS VERY TEMP, WILL CHANGE TO REAL SYSTEM LATER BUT FOR NOW WILL WORK
    pub instance_id: MeshInstanceId,
}

pub struct SunShadow {
    pub camera: OrthographicCamera,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: Buffer,
    pub texture_view: TextureView,
    pub texture_sampler: wgpu::Sampler,
    pub texture_distance_sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
}

impl<'a> RenderState {
    pub async fn new(
        window: Arc<Window>,
        render_thread_channels: RenderThreadChannels,
    ) -> anyhow::Result<RenderState> {
        return init_render_state(window, render_thread_channels).await;
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
