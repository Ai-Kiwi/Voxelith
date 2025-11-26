use std::{sync::Arc, time::Instant};

use egui::ViewportId;
use winit::{application::ApplicationHandler, event::{DeviceEvent, DeviceId, KeyEvent, WindowEvent}, event_loop::ActiveEventLoop, keyboard::{KeyCode, PhysicalKey}, platform, window::{Theme, Window}};

use crate::{mesh_creator::{MeshCreator, init_mesh_creator}, render::{GameData, mesh::mesh_buffer_cleanup, wgpu::RenderState}, render_game::tick_game_render_logic, utils::Vec2};

enum PageOpen {
    Game,
    TitleScreen,
}

pub struct App {
    pub state: Option<RenderState>,
    pub game_data: Option<GameData>,
    pub mesh_creator : Option<MeshCreator>,
    pub page_open: PageOpen,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: None,
            game_data: None,
            page_open : PageOpen::Game,
            mesh_creator: None,
        }
    }

    fn update_cursor_lock(render_state : &mut RenderState, locked : bool) {
        if locked {
            let _ = render_state.window.set_cursor_grab(winit::window::CursorGrabMode::Confined).or_else(|_e| render_state.window.set_cursor_grab(winit::window::CursorGrabMode::Locked));
            render_state.window.set_cursor_visible(false);
        }else{
            let _ = render_state.window.set_cursor_grab(winit::window::CursorGrabMode::None);
            render_state.window.set_cursor_visible(true);
        }
    }
        
    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            (KeyCode::F11, true) => {
                state.fullscreen = !state.fullscreen;
                if state.fullscreen {
                    state.window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(state.window.current_monitor())));
                }else{
                    state.window.set_fullscreen(None);
                }
            },
            (KeyCode::Tab, true) => {
                state.game_selected = !state.game_selected;
                match self.page_open {
                    PageOpen::Game => {App::update_cursor_lock(state, state.game_selected)},
                    PageOpen::TitleScreen => {App::update_cursor_lock(state, false);},
                }
            },
            //update the keys that are are pressed in the render state
            _ => {
                if is_pressed {
                    if !state.keys_down.contains_key(&code) {
                        state.keys_down.insert(code, ());
                        state.keys_pressed.insert(code, ());
                    }
                }else{
                    state.keys_down.remove(&code);
                    state.keys_released.insert(code, ());
                }
            }
        }
    }
}

impl ApplicationHandler<RenderState> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let _ = window.set_cursor_grab(winit::window::CursorGrabMode::Confined).or_else(|_e| window.set_cursor_grab(winit::window::CursorGrabMode::Locked));
        window.set_cursor_visible(false);
        window.set_title("Voxelith");


        self.state = Some(pollster::block_on(RenderState::new(window)).unwrap());

    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: RenderState) {
        self.state = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        //update egui update
        if state.game_selected == false {
            let _ = state.egui_winit.on_window_event(&state.window, &event);
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                //tick game render logic
                if let Some(game_data) = &mut self.game_data {
                    tick_game_render_logic(state, game_data);
                }

                //tick mesh creator
                if let Some(mesh_creator) = &mut self.mesh_creator {
                    init_mesh_creator(state, mesh_creator);
                }

                //clean up mesh buffers
                mesh_buffer_cleanup(state);

                //cleanup data now that frame info has been processed
                state.keys_released.clear();
                state.keys_pressed.clear();
                state.mouse_position_delta = Vec2::new(0.0,0.0);
                let now = Instant::now();
                state.delta_time = (now - state.last_frame_time).as_secs_f32();
                state.last_frame_time = now;

                
                match state.render(&mut self.game_data) {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = state.window.inner_size();
                        state.resize(size.width, size.height);
                    }
                    Err(e) => {
                        log::error!("Unable to render {}", e);
                    }
                }
            },
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => self.handle_key(event_loop, code, key_state.is_pressed()),
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            DeviceEvent::MouseMotion { delta } => {
                if state.game_selected {
                    let screen_size = state.window.inner_size();
                    let mouse_sensitivity = 10.0 / (screen_size.width as f32);
    
                    state.mouse_position_delta.x += delta.0 as f32 * mouse_sensitivity;
                    state.mouse_position_delta.y += delta.1 as f32 * mouse_sensitivity; 
                }
            },
            _ => {}
        }
    }
}

