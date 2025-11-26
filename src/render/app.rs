use std::sync::Arc;

use egui::ViewportId;
use winit::{application::ApplicationHandler, event::{DeviceEvent, DeviceId, KeyEvent, WindowEvent}, event_loop::ActiveEventLoop, keyboard::PhysicalKey, platform, window::{Theme, Window}};

use crate::render::{GameData, init_frame_render, wgpu::RenderState};


pub struct App {
    pub state: Option<RenderState>,
    pub game_data: Option<GameData>,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: None,
            game_data: None
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
                match &mut self.game_data {
                    Some(game_data) => {
                        init_frame_render(state, Some(game_data));
                    },
                    None => {
                        init_frame_render(state, None);
                    },
                }
                
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
            } => state.handle_key(event_loop, code, key_state.is_pressed()),
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

