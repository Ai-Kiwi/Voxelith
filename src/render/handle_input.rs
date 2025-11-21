use cgmath::Point3;

use crate::{game::InputEvent, render::wgpu::RenderState, utils::{Vec3}};


pub fn handle_user_input(render_state : &mut RenderState) {
    //move player
    let forward_dir = Vec3::new(
        render_state.data.camera_yaw.cos(),
        0.0,
        render_state.data.camera_yaw.sin()
    );

    let backwards_dir = forward_dir * -1.0;

    let right_dir = Vec3::new(
        -render_state.data.camera_yaw.sin(),
        0.0,
        render_state.data.camera_yaw.cos()
    );

    let left_dir = right_dir  * -1.0;

    let movement_speed = if render_state.keys_down.contains_key(&winit::keyboard::KeyCode::ShiftLeft) {
        500.0
    }else {
        100.0
    };


    if render_state.keys_down.contains_key(&winit::keyboard::KeyCode::KeyW) {
        render_state.data.camera_position += forward_dir * movement_speed * render_state.delta_time;
    }
    if render_state.keys_down.contains_key(&winit::keyboard::KeyCode::KeyS) {
        render_state.data.camera_position +=  backwards_dir * movement_speed * render_state.delta_time;
    }

    if render_state.keys_down.contains_key(&winit::keyboard::KeyCode::KeyA) {
        render_state.data.camera_position += left_dir * movement_speed * render_state.delta_time;
    }
    if render_state.keys_down.contains_key(&winit::keyboard::KeyCode::KeyD) {
        render_state.data.camera_position += right_dir * movement_speed * render_state.delta_time;
    }

    if render_state.keys_down.contains_key(&winit::keyboard::KeyCode::Space) {
        render_state.data.camera_position.y += movement_speed * render_state.delta_time;
    }
    if render_state.keys_down.contains_key(&winit::keyboard::KeyCode::ControlLeft) {
        render_state.data.camera_position.y -= movement_speed * render_state.delta_time;
    }

    let front = Vec3::new(
        render_state.data.camera_yaw.cos() * render_state.data.camera_pitch.cos(),
        render_state.data.camera_pitch.sin(),
        render_state.data.camera_yaw.sin() * render_state.data.camera_pitch.cos()
    ).normalize();


    if render_state.keys_pressed.contains_key(&winit::keyboard::KeyCode::KeyE) {
        render_state.render_channels.input_event_tx.send(InputEvent::Click(Vec3 { 
            x: render_state.camera.eye.x, 
            y: render_state.camera.eye.y, 
            z: render_state.camera.eye.z 
        },front)).unwrap();
    }


    //handle camera turning
    render_state.data.camera_yaw += render_state.mouse_position_delta.x;
    render_state.data.camera_pitch -= render_state.mouse_position_delta.y;

    render_state.data.camera_pitch = if render_state.data.camera_pitch > 1.5 { 1.5 } else { render_state.data.camera_pitch };
    render_state.data.camera_pitch = if render_state.data.camera_pitch < -1.5 { -1.5 } else { render_state.data.camera_pitch };


    let target_position =  render_state.data.camera_position + front;
    render_state.camera.eye = Point3::new(
        render_state.data.camera_position.x, 
        render_state.data.camera_position.y, 
        render_state.data.camera_position.z
    );
    render_state.camera.target = Point3::new(
        target_position.x, 
        target_position.y, 
        target_position.z
    );

    render_state.camera_uniform.update_view_proj(&render_state.camera);
    render_state.queue.write_buffer(&render_state.camera_buffer, 0, bytemuck::cast_slice(&[render_state.camera_uniform]));
}