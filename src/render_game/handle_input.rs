use crate::{game::InputEvent, render::wgpu::RenderState, render_game::GameData, utils::Vec3};

pub fn handle_user_input(render_state : &mut RenderState, game_data : &mut GameData) {
    //move player
    let forward_dir = Vec3::new(
        game_data.camera.yaw.cos(),
        0.0,
        game_data.camera.yaw.sin()
    );

    let backwards_dir = forward_dir * -1.0;

    let right_dir = Vec3::new(
        -game_data.camera.yaw.sin(),
        0.0,
        game_data.camera.yaw.cos()
    );

    let left_dir = right_dir  * -1.0;

    let movement_speed = if render_state.keys_down.contains_key(&winit::keyboard::KeyCode::ShiftLeft) {
        500.0
    }else {
        100.0
    };


    if render_state.keys_down.contains_key(&winit::keyboard::KeyCode::KeyW) {
        game_data.camera.position += forward_dir * movement_speed * render_state.delta_time;
    }
    if render_state.keys_down.contains_key(&winit::keyboard::KeyCode::KeyS) {
        game_data.camera.position +=  backwards_dir * movement_speed * render_state.delta_time;
    }

    if render_state.keys_down.contains_key(&winit::keyboard::KeyCode::KeyA) {
        game_data.camera.position += left_dir * movement_speed * render_state.delta_time;
    }
    if render_state.keys_down.contains_key(&winit::keyboard::KeyCode::KeyD) {
        game_data.camera.position += right_dir * movement_speed * render_state.delta_time;
    }

    if render_state.keys_down.contains_key(&winit::keyboard::KeyCode::Space) {
        game_data.camera.position.y += movement_speed * render_state.delta_time;
    }
    if render_state.keys_down.contains_key(&winit::keyboard::KeyCode::ControlLeft) {
        game_data.camera.position.y -= movement_speed * render_state.delta_time;
    }

    let front = Vec3::new(
        game_data.camera.yaw.cos() * game_data.camera.pitch.cos(),
        game_data.camera.pitch.sin(),
        game_data.camera.yaw.sin() * game_data.camera.pitch.cos()
    ).normalize();


    if render_state.keys_pressed.contains_key(&winit::keyboard::KeyCode::KeyE) {
        game_data.render_channels.input_event_tx.send(InputEvent::Click(Vec3 { 
            x: game_data.camera.position.x, 
            y: game_data.camera.position.y, 
            z: game_data.camera.position.z 
        },front)).unwrap();
    }
    if render_state.keys_pressed.contains_key(&winit::keyboard::KeyCode::KeyQ) {
        game_data.render_channels.input_event_tx.send(InputEvent::PlaceClick(Vec3 { 
            x: game_data.camera.position.x, 
            y: game_data.camera.position.y, 
            z: game_data.camera.position.z 
        },front)).unwrap();
    }


    //handle camera turning
    game_data.camera.yaw += render_state.mouse_position_delta.x;
    game_data.camera.pitch -= render_state.mouse_position_delta.y;

    game_data.camera.pitch = if game_data.camera.pitch > 1.5 { 1.5 } else { game_data.camera.pitch };
    game_data.camera.pitch = if game_data.camera.pitch < -1.5 { -1.5 } else { game_data.camera.pitch };

    render_state.camera_uniform.update_view_proj_prespec(&mut game_data.camera, render_state.config.width, render_state.config.height);
    render_state.queue.write_buffer(&render_state.camera_buffer, 0, bytemuck::cast_slice(&[render_state.camera_uniform]));
}