use std::collections::HashMap;
use egui::Color32;
use wgpu::{Buffer, RenderPass};
use crate::{render::{camera::Camera, mesh, wgpu::RenderState}, utils::{Color, Vec3}};

mod create_mesh;

pub struct MeshCreator {
    pub mesh_voxels : HashMap<(i32,i32,i32),Color>,
    pub mesh_buffer : Option<Buffer>,
    pub mesh_buffer_size : u32,
    pub camera : Camera,
    pub camera_distance : f32,
    pub selected_color : [u8; 3]
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

    mesh_creator.mesh_voxels.insert((0,0,0), Color ::new(255, 255, 255, 255));

    mesh_creator.update_mesh(render_state);


}

pub fn render_mesh_creator(render_state : &mut RenderState, render_pass : &mut RenderPass<'_>, mesh_creator : &mut MeshCreator) {
    render_pass.set_pipeline(&render_state.opaque_render_pipeline);
    render_pass.set_bind_group(0, &render_state.camera_bind_group, &[]);

    if let Some(mesh_buffer) = &mesh_creator.mesh_buffer {
        render_pass.set_vertex_buffer(0, mesh_buffer.slice(..));
        render_pass.draw(0..mesh_creator.mesh_buffer_size, 0..1);
    }
}
