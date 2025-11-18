use std::{collections::HashMap, sync::mpsc::{Receiver, Sender}, time::Instant};

use winit::event_loop::EventLoop;

use crate::{game::InputEvent, render::{app::App, handle_input::handle_user_input, mesh::update_meshs, types::{ChunkMeshUpdate, EntityRenderData}, wgpu::RenderState}, utils::{Mesh, Vec2, Vec3}};
use ::wgpu::Buffer;


//mod entities;
mod mesh;
mod handle_input;
//mod gui;
pub mod types;
mod camera;
mod wgpu;
mod app;

//pub const LEVEL_3_LOD_DISTANCE: f32 = 2560.0;
//pub const LEVEL_2_LOD_DISTANCE: f32 = 1280.0;
//pub const LEVEL_1_LOD_DISTANCE: f32 = 640.0;

pub const LEVEL_1_LOD_DISTANCE: f32 = 480.0;
pub const LEVEL_2_LOD_DISTANCE: f32 = LEVEL_1_LOD_DISTANCE * 2.0;
pub const LEVEL_3_LOD_DISTANCE: f32 = LEVEL_2_LOD_DISTANCE * 2.0;
pub const LEVEL_4_LOD_DISTANCE: f32 = LEVEL_3_LOD_DISTANCE * 2.0;


pub struct RenderChunkMesh {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub num_indices: u32,
    pub mesh_data : Mesh,
}

pub struct FullRenderChunkInfo {
    pub main_lod : RenderChunkMesh,
    pub lod_2 : RenderChunkMesh,
    pub lod_4 : RenderChunkMesh,
    pub lod_8 : RenderChunkMesh,
}

impl RenderData {
    pub fn new() -> RenderData {
        RenderData {
            camera_yaw: 0.0,
            camera_pitch: 0.0,
            camera_position: Vec3::new(0.0, 0.0, 0.0),
            chunk_meshs: HashMap::new(),
            chunk_batch_meshs: HashMap::new(),
            dirty_chunk_mesh_batches: HashMap::new(),
        }
    }
}

pub struct RenderThreadChannels {
    chunk_mesh_update_rx : Receiver<ChunkMeshUpdate>, 
    entity_render_rx : Receiver<EntityRenderData>, 
    input_event_tx: Sender<InputEvent>
}

pub struct RenderData {
    camera_yaw : f32,
    camera_pitch : f32,
    camera_position : Vec3,
    chunk_meshs : HashMap<(i32,i32,i32),FullRenderChunkInfo>,
    chunk_batch_meshs : HashMap<(i32,i32,i32,u8),RenderChunkMesh>,
    dirty_chunk_mesh_batches : HashMap<(i32,i32,i32,u8),()>,
    //let mut entities_to_render: HashMap<u64,EntityRenderData> = HashMap::new();
}


pub fn init_frame_render(render_state : &mut RenderState) {
    #[cfg(feature = "perf_logs")]
    let init_frame_start_time = Instant::now();
    
    handle_user_input(render_state);

    update_meshs(render_state);


    //cleanup data now that frame has happened
    render_state.keys_released.clear();
    render_state.keys_pressed.clear();
    render_state.mouse_position_delta = Vec2::new(0.0,0.0);
    let now = Instant::now();
    render_state.delta_time = (now - render_state.last_frame_time).as_secs_f32();
    render_state.last_frame_time = now;
    #[cfg(feature = "perf_logs")]
    println!("init frame time {}ms",init_frame_start_time.elapsed().as_millis())
}







pub async fn render_thread(chunk_mesh_update_rx : Receiver<ChunkMeshUpdate>, entity_render_rx : Receiver<EntityRenderData>, input_event_tx: Sender<InputEvent>) {
    let thread_chennels = RenderThreadChannels {
        chunk_mesh_update_rx,
        entity_render_rx,
        input_event_tx
    };

    let event_loop = EventLoop::with_user_event().build().expect("failed to create event loop");
    let mut app: App = App::new(thread_chennels);
    event_loop.run_app(&mut app).expect("failed to run app");
}