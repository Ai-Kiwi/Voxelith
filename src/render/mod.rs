use std::{collections::HashMap, sync::mpsc::{Receiver, Sender}, time::Instant};

use winit::event_loop::EventLoop;

use crate::{game::InputEvent, render::{app::App, handle_input::handle_user_input, mesh::{chunk_buffer_cleanup, update_meshs}, types::{ChunkMeshUpdate, EntityRenderData}, wgpu::RenderState}, utils::{Mesh, Vec2, Vec3}};


//mod entities;
mod mesh;
mod handle_input;
//mod gui;
pub mod types;
mod camera;
mod wgpu;
mod app;
mod init;
mod render_frame;

//pub const LEVEL_3_LOD_DISTANCE: f32 = 2560.0;
//pub const LEVEL_2_LOD_DISTANCE: f32 = 1280.0;
//pub const LEVEL_1_LOD_DISTANCE: f32 = 640.0;

pub const LEVEL_1_LOD_DISTANCE: f32 = 480.0;
pub const LEVEL_2_LOD_DISTANCE: f32 = LEVEL_1_LOD_DISTANCE * 2.0;
pub const LEVEL_3_LOD_DISTANCE: f32 = LEVEL_2_LOD_DISTANCE * 2.0;
pub const LEVEL_4_LOD_DISTANCE: f32 = LEVEL_3_LOD_DISTANCE * 2.0;
pub const MAP_VRAM_SIZE: u64 = 3 * 1024 * 1024 * 1024;

pub struct RenderChunkMeshBufferReference {
    pub byte_vertex_position : u32,
    pub byte_vertex_length : u32,
    pub vertex_position : u32,
    pub vertex_length : u32,
    pub lod : u8,
}

pub struct FreeBufferSpace {
    byte_start: u32,
    byte_len: u32,
}

impl RenderData {
    pub fn new() -> RenderData {
        RenderData {
            camera_yaw: 0.0,
            camera_pitch: 0.0,
            camera_position: Vec3::new(0.0, 0.0, 0.0),
            chunk_meshs: HashMap::new(),
            chunk_mesh_data: HashMap::new(),
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
    chunk_meshs : HashMap<(i32,i32,i32),RenderChunkMeshBufferReference>,
    chunk_mesh_data : HashMap<(i32,i32,i32),ChunkMeshUpdate>,
    //let mut entities_to_render: HashMap<u64,EntityRenderData> = HashMap::new();
}


pub fn init_frame_render(render_state : &mut RenderState) {
    #[cfg(feature = "perf_logs")]
    let init_frame_start_time = Instant::now();
    
    handle_user_input(render_state);

    update_meshs(render_state);

    chunk_buffer_cleanup(render_state);

    //update player pos
    let _ = render_state.render_channels.input_event_tx.send(InputEvent::CameraPositionUpdate(Vec3::new(
        render_state.camera.eye.x, 
        render_state.camera.eye.y, 
        render_state.camera.eye.z
    )));

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