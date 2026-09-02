use crate::{game::InputEvent, render::wgpu::RenderState, utils::Vec3};

pub fn handle_user_input(render_state: &mut RenderState) {
    //move player
    let forward_dir = Vec3::new(
        render_state.camera.yaw.cos(),
        0.0,
        render_state.camera.yaw.sin(),
    );

    let backwards_dir = forward_dir * -1.0;

    let right_dir = Vec3::new(
        -render_state.camera.yaw.sin(),
        0.0,
        render_state.camera.yaw.cos(),
    );

    let left_dir = right_dir * -1.0;

    let movement_speed = if render_state
        .keys_down
        .contains_key(&winit::keyboard::KeyCode::ShiftLeft)
    {
        500.0
    } else {
        100.0
    };

    if render_state
        .keys_down
        .contains_key(&winit::keyboard::KeyCode::KeyW)
    {
        render_state.camera.position += forward_dir * movement_speed * render_state.delta_time;
    }
    if render_state
        .keys_down
        .contains_key(&winit::keyboard::KeyCode::KeyS)
    {
        render_state.camera.position += backwards_dir * movement_speed * render_state.delta_time;
    }

    if render_state
        .keys_down
        .contains_key(&winit::keyboard::KeyCode::KeyA)
    {
        render_state.camera.position += left_dir * movement_speed * render_state.delta_time;
    }
    if render_state
        .keys_down
        .contains_key(&winit::keyboard::KeyCode::KeyD)
    {
        render_state.camera.position += right_dir * movement_speed * render_state.delta_time;
    }

    if render_state
        .keys_down
        .contains_key(&winit::keyboard::KeyCode::Space)
    {
        render_state.camera.position.y += movement_speed * render_state.delta_time;
    }
    if render_state
        .keys_down
        .contains_key(&winit::keyboard::KeyCode::ControlLeft)
    {
        render_state.camera.position.y -= movement_speed * render_state.delta_time;
    }

    let front = Vec3::new(
        render_state.camera.yaw.cos() * render_state.camera.pitch.cos(),
        render_state.camera.pitch.sin(),
        render_state.camera.yaw.sin() * render_state.camera.pitch.cos(),
    )
    .normalize();

    if render_state
        .keys_pressed
        .contains_key(&winit::keyboard::KeyCode::KeyE)
    {
        render_state
            .render_channels
            .input_event_tx
            .send(InputEvent::Click(
                Vec3 {
                    x: render_state.camera.position.x,
                    y: render_state.camera.position.y,
                    z: render_state.camera.position.z,
                },
                front,
            ))
            .unwrap();
    }
    if render_state
        .keys_pressed
        .contains_key(&winit::keyboard::KeyCode::KeyQ)
    {
        render_state
            .render_channels
            .input_event_tx
            .send(InputEvent::PlaceClick(
                Vec3 {
                    x: render_state.camera.position.x,
                    y: render_state.camera.position.y,
                    z: render_state.camera.position.z,
                },
                front,
            ))
            .unwrap();
    }

    //handle camera turning
    render_state.camera.yaw += render_state.mouse_position_delta.x;
    render_state.camera.pitch -= render_state.mouse_position_delta.y;

    render_state.camera.pitch = if render_state.camera.pitch > 1.5 {
        1.5
    } else {
        render_state.camera.pitch
    };
    render_state.camera.pitch = if render_state.camera.pitch < -1.5 {
        -1.5
    } else {
        render_state.camera.pitch
    };

    render_state.camera_uniform.update_view_proj_prespec(
        &mut render_state.camera,
        render_state.config.width,
        render_state.config.height,
    );
    render_state.queue.write_buffer(
        &render_state.camera_buffer,
        0,
        bytemuck::cast_slice(&[render_state.camera_uniform]),
    );
}
