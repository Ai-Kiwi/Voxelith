use std::{collections::HashMap, f32::consts::PI};
use wgpu::{Buffer, CommandEncoder, RenderPass};
use crate::{render::{camera::Camera, wgpu::RenderState}, utils::{Color, Vec3, raycast_test, voxel_raycast_test}};

mod create_mesh;
mod files;

pub struct MeshCreator {
    pub mesh_voxels : HashMap<(i32,i32,i32),Color>,
    pub mesh_buffer : Option<Buffer>,
    pub mesh_buffer_size : u32,
    pub camera : Camera,
    pub camera_distance : f32,
    pub selected_color : [u8; 3],
    pub file_editing : String,
    pub update_due : bool
}

impl MeshCreator {
    pub fn new() -> MeshCreator {
        MeshCreator {
            mesh_voxels: HashMap::new(),
            mesh_buffer: None,
            mesh_buffer_size: 0,
            camera: Camera::new(),
            camera_distance: 10.0,
            selected_color: [255,255,255],
            file_editing: "basic".to_string(),
            update_due: true,
        }
    }
}

pub fn tick_mesh_creator(render_state : &mut RenderState, mesh_creator : &mut MeshCreator, page_open : bool) {
    if page_open == false {
        return;
    }

    //handle camera turning
    mesh_creator.camera.yaw += render_state.mouse_position_delta.x;
    mesh_creator.camera.pitch -= render_state.mouse_position_delta.y;

    mesh_creator.camera.pitch = if mesh_creator.camera.pitch > 1.5 { 1.5 } else { mesh_creator.camera.pitch };
    mesh_creator.camera.pitch = if mesh_creator.camera.pitch < -1.5 { -1.5 } else { mesh_creator.camera.pitch };


    let camera_front_norm = Vec3::new(
        mesh_creator.camera.yaw.cos() * mesh_creator.camera.pitch.cos(),
        mesh_creator.camera.pitch.sin(),
        mesh_creator.camera.yaw.sin() * mesh_creator.camera.pitch.cos()
    ).normalize();

    let camera_position = Vec3::new(0.0, 0.0, 0.0) - (camera_front_norm * mesh_creator.camera_distance);
    mesh_creator.camera.position = camera_position;

    render_state.camera_uniform.update_view_proj(&mut mesh_creator.camera, render_state.config.width, render_state.config.height);


    if render_state.keys_pressed.contains_key(&winit::keyboard::KeyCode::KeyE) {
        let screen_height = render_state.config.height as f32;
        let screen_width = render_state.config.width as f32;
        let fov_y: f32 = mesh_creator.camera.fovy.to_radians();
        let fov_x = 2.0 * ((screen_width / screen_height) * (fov_y / 2.0).tan()).atan();

        let normalized_mouse_x = (render_state.mouse_position.x - (screen_width / 2.0)) / (screen_width / 2.0);
        let normalized_mouse_y = -1.0 * (render_state.mouse_position.y - (screen_height / 2.0)) / (screen_height / 2.0);

        let half_fov_y_tan = (fov_y * 0.5).tan();
        let half_fov_x_tan = (fov_x * 0.5).tan();

        let angle_offset_y = (normalized_mouse_y * half_fov_y_tan).atan();
        let angle_offset_x = (normalized_mouse_x * half_fov_x_tan).atan();

        println!("{} {}",normalized_mouse_x, normalized_mouse_y);
        println!("{} {}",angle_offset_x, angle_offset_y);

        let place_pitch = mesh_creator.camera.pitch + angle_offset_y;
        let place_yaw = mesh_creator.camera.yaw + angle_offset_x;

        let ray_direction = Vec3::new(
            place_yaw.cos() * place_pitch.cos(),
            place_pitch.sin(),
            place_yaw.sin() * place_pitch.cos()
        ).normalize();

        let mut last_ray_postion = mesh_creator.camera.position;
        let mut i = 0;
        for ray in voxel_raycast_test(mesh_creator.camera.position, ray_direction) {
            i+=1;
            if ray.y < 0.0 {
                break;
            }
            if i > 100 {
                break;
            }
            if mesh_creator.mesh_voxels.get(&(ray.x as i32,ray.y as i32,ray.z as i32)).is_some() {
                break;
            }
            last_ray_postion = ray
        }

        mesh_creator.mesh_voxels.insert((last_ray_postion.x as i32,last_ray_postion.y as i32,last_ray_postion.z as i32), Color::new(mesh_creator.selected_color[0], mesh_creator.selected_color[1], mesh_creator.selected_color[2], 255));
        mesh_creator.update_due = true;
    }


    if mesh_creator.update_due {
        mesh_creator.update_mesh(render_state);
        mesh_creator.update_due = false;
    }
}

pub fn render_mesh_creator(render_state : &mut RenderState, mesh_creator : &mut MeshCreator, view: &wgpu::TextureView, encoder : &mut CommandEncoder) {
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
                store: wgpu::StoreOp::Store,
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                }),
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { 
            view: &render_state.depth_view, 
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }), 
            stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
    });


    render_pass.set_pipeline(&render_state.basic_mesh_render_pipeline);
    render_pass.set_bind_group(0, &render_state.camera_bind_group, &[]);

    if mesh_creator.mesh_buffer_size > 0 {
        if let Some(mesh_buffer) = &mesh_creator.mesh_buffer {
            render_pass.set_vertex_buffer(0, mesh_buffer.slice(..));
            render_pass.draw(0..mesh_creator.mesh_buffer_size, 0..1);
        }
    }
}
