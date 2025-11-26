use std::{collections::HashMap, hash::Hash, rc::Weak, sync::{Arc, mpsc::{Receiver, Sender, channel}}, thread, time::Instant};

use pollster::block_on;
use ::wgpu::Buffer;
use winit::event_loop::EventLoop;

use crate::{game::{GameSnapshot, InputEvent, game_thread}, render::{app::App, handle_input::handle_user_input, mesh::{GpuMeshReference, mesh_buffer_cleanup, update_chunk_meshs}, types::{ChunkMeshUpdate, EntityRenderData}, wgpu::RenderState}, utils::{Mesh, Vec2, Vec3}};


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
mod debug_gui;

//pub const LEVEL_3_LOD_DISTANCE: f32 = 2560.0;
//pub const LEVEL_2_LOD_DISTANCE: f32 = 1280.0;
//pub const LEVEL_1_LOD_DISTANCE: f32 = 640.0;

pub const LEVEL_1_LOD_DISTANCE: f32 = 480.0;
pub const LEVEL_2_LOD_DISTANCE: f32 = LEVEL_1_LOD_DISTANCE * 2.0;
pub const LEVEL_3_LOD_DISTANCE: f32 = LEVEL_2_LOD_DISTANCE * 2.0;
pub const LEVEL_4_LOD_DISTANCE: f32 = LEVEL_3_LOD_DISTANCE * 2.0;
pub const MAP_VRAM_SIZE: u64 = 3 * 1024 * 1024 * 1024;

impl GameData {
    pub fn new() -> GameData {
        GameData {
            camera_yaw: 0.0,
            camera_pitch: 0.0,
            camera_position: Vec3::new(0.0, 0.0, 0.0),
            chunk_meshs: HashMap::new(),
            chunk_mesh_data: HashMap::new(),
            render_channels: todo!(),
        }
    }
}

pub struct RenderThreadChannels {
    chunk_mesh_update_rx : Receiver<ChunkMeshUpdate>, 
    entity_render_rx : Receiver<EntityRenderData>, 
    input_event_tx: Sender<InputEvent>,
}

pub struct ChunkInfo {
    pointer : Arc<GpuMeshReference>,
    lod : u8,
    size : usize
}

pub struct GameData {
    pub camera_yaw : f32,
    pub camera_pitch : f32,
    pub camera_position : Vec3,
    pub chunk_meshs : HashMap<(i32,i32,i32,bool),ChunkInfo>,
    pub chunk_mesh_data : HashMap<(i32,i32,i32,bool),ChunkMeshUpdate>,
    pub render_channels : RenderThreadChannels,
    //let mut entities_to_render: HashMap<u64,EntityRenderData> = HashMap::new();
}


pub fn init_frame_render(render_state : &mut RenderState, game_data : Option<&mut GameData>) {
    if let Some(game_data) = game_data {
        handle_user_input(render_state, game_data);
        update_chunk_meshs(render_state, game_data);

        //update player pos
        let _ = game_data.render_channels.input_event_tx.send(InputEvent::CameraPositionUpdate(Vec3::new(
            render_state.camera.eye.x, 
            render_state.camera.eye.y, 
            render_state.camera.eye.z
        )));
    }

    mesh_buffer_cleanup(render_state);



    //cleanup data now that frame has happened
    render_state.keys_released.clear();
    render_state.keys_pressed.clear();
    render_state.mouse_position_delta = Vec2::new(0.0,0.0);
    let now = Instant::now();
    render_state.delta_time = (now - render_state.last_frame_time).as_secs_f32();
    render_state.last_frame_time = now;
}







pub async fn render_thread() {
    let event_loop = EventLoop::with_user_event().build().expect("failed to create event loop");
    let mut app: App = App::new();

    //start up game state
    let (chunk_mesh_update_tx, chunk_mesh_update_rx) = channel::<ChunkMeshUpdate>();
    let (entity_render_tx, entity_render_rx) = channel::<EntityRenderData>();
    let (input_event_tx, mut input_event_rx) = channel::<InputEvent>();
    
    //game loop thread start
    let _ = thread::spawn(move || {
        block_on(game_thread(chunk_mesh_update_tx, entity_render_tx, &mut input_event_rx));
    });

    let game_state = GameData {
        camera_yaw: 0.0,
        camera_pitch: 0.0,
        camera_position: Vec3::new(0.0, 0.0, 0.0),
        chunk_meshs: HashMap::new(),
        chunk_mesh_data: HashMap::new(),
        render_channels: RenderThreadChannels {
            chunk_mesh_update_rx,
            entity_render_rx,
            input_event_tx,
        },
    };

    app.game_data = Some(game_state);

    event_loop.run_app(&mut app).expect("failed to run app");
    
    //drop everything. This allows for a clean shutdown
    app.game_data = None;
    app.state = None;
}