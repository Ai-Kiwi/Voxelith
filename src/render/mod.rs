use std::{collections::HashMap, hash::Hash, rc::Weak, sync::{Arc, mpsc::{Receiver, Sender, channel}}, thread, time::Instant};

use pollster::block_on;
use ::wgpu::Buffer;
use winit::event_loop::EventLoop;

use crate::{game::{GameSnapshot, InputEvent, game_thread}, render::{app::App, camera::Camera, mesh::{GpuMeshReference, mesh_buffer_cleanup}, wgpu::RenderState}, render_game::{GameData, RenderThreadChannels, chunk::{ChunkMeshUpdate, EntityRenderData}, tick_game_render_logic}, utils::{Mesh, Vec2, Vec3}};


pub mod camera;
pub mod wgpu;
mod app;
mod init;
mod render_frame;
pub mod mesh;

//pub const LEVEL_3_LOD_DISTANCE: f32 = 2560.0;
//pub const LEVEL_2_LOD_DISTANCE: f32 = 1280.0;
//pub const LEVEL_1_LOD_DISTANCE: f32 = 640.0;


pub fn init_frame_render(render_state : &mut RenderState, game_data : Option<&mut GameData>) {
    //game logic stuff
    if let Some(game_data) = game_data {
        tick_game_render_logic(render_state, game_data);
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
        chunk_meshs: HashMap::new(),
        chunk_mesh_data: HashMap::new(),
        render_channels: RenderThreadChannels {
            chunk_mesh_update_rx,
            entity_render_rx,
            input_event_tx,
        },
        camera: Camera::new(),
    };

    app.game_data = Some(game_state);

    event_loop.run_app(&mut app).expect("failed to run app");
    
    //drop everything. This allows for a clean shutdown
    app.game_data = None;
    app.state = None;
}