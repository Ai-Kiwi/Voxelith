use std::collections::HashMap;

use wgpu::Buffer;

use crate::{render::wgpu::RenderState, utils::Color};

pub struct MeshCreator {
    mesh_voxels : HashMap<(i32,i32,i32),Color>,
    mesh_buffer : Buffer,
}

pub fn init_mesh_creator(render_state : &mut RenderState, mesh_creator : &mut MeshCreator) {

}

pub fn render_mesh_creator(render_state : &mut RenderState, mesh_creator : &mut MeshCreator) {

}
