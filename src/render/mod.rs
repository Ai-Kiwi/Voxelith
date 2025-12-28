use std::{collections::{BTreeMap, HashMap}, sync::mpsc::channel, thread};

use pollster::block_on;
use winit::event_loop::EventLoop;

use crate::{game::{InputEvent, entity::EntityRenderData, game_thread}, mesh_creator::MeshCreator, render::{app::App, camera::PerspectiveCamera}, render_game::{GameData, RenderThreadChannels, chunk::ChunkMeshUpdate}};


pub mod camera;
pub mod wgpu;
mod app;
mod init;
mod render_frame;
pub mod mesh;

//pub const LEVEL_3_LOD_DISTANCE: f32 = 2560.0;
//pub const LEVEL_2_LOD_DISTANCE: f32 = 1280.0;
//pub const LEVEL_1_LOD_DISTANCE: f32 = 640.0;
pub const MAP_VRAM_SIZE: u64 = 256 * 1024 * 1024; //256mb

pub struct RenderFrameThreadPerformanceInfo {
    total_tick_time : f32, 
    main_game_tick : f32, 
    mesh_creator_tick : f32, 
    update_mesh_buffer : f32, 
    total_render_time : f32, 
    start_render_time : f32, 
    main_content_render_time : f32, 
    render_gui_time : f32,
    finish_render_time : f32,
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
        camera: PerspectiveCamera::new(),
        cache_chunk_meshs: BTreeMap::new(),
    };

    app.game_data = Some(game_state);

    app.page_open = app::PageOpen::Game;
    app.mesh_creator = Some(MeshCreator::new());
    

    event_loop.run_app(&mut app).expect("failed to run app");
    
    //drop everything. This allows for a clean shutdown
    app.game_data = None;
    app.state = None;
}

// https://acko.net/blog/teardown-frame-teardown/