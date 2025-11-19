use std::sync::Arc;

use winit::{application::ApplicationHandler, event::{DeviceEvent, DeviceId, KeyEvent, WindowEvent}, event_loop::ActiveEventLoop, keyboard::PhysicalKey, window::Window};

use crate::render::{RenderThreadChannels, wgpu::RenderState};


pub struct App {
    state: Option<RenderState>,
    render_channels : Option<RenderThreadChannels>
}

impl App {
    pub fn new(render_channel : RenderThreadChannels) -> Self {
        Self {
            state: None,
            render_channels : Some(render_channel)
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
        // If we are not on web we can use pollster to
        // await the 
        match self.render_channels.take() {
            Some(channels) => self.state = Some(pollster::block_on(RenderState::new(window,channels)).unwrap()),
            None => panic!("Channels has already been sent to render state so can't resend"),
        }

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

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                state.update();
                match state.render() {
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
                let screen_size = state.window.inner_size();
                let mouse_sensitivity = 10.0 / (screen_size.width as f32);

                state.mouse_position_delta.x += delta.0 as f32 * mouse_sensitivity;
                state.mouse_position_delta.y += delta.1 as f32 * mouse_sensitivity; 
            },
            _ => {}
        }
    }

    

}

