use std::collections::HashMap;

use wgpu::util::DeviceExt;

use crate::{mesh_creation::create_triangles::{TriangleSide, add_triangle}, mesh_creator::MeshCreator, render::wgpu::RenderState, utils::{Color, Vertex}};

//TODO: merge this code with other code. As it does same as mesh creation thread so could be one function. Although it does complex it so might leave as 2

impl MeshCreator {
    pub fn vertices_from_voxel_color_hashmap(voxel_data : &HashMap<(i32,i32,i32),Color>) -> Vec<Vertex> {
        let mut vertices : Vec<Vertex> = Vec::new();


        for voxel in voxel_data {
            let voxel_x = voxel.0.0;
            let voxel_y = voxel.0.1;
            let voxel_z  = voxel.0.2;

            if voxel_data.get(&(voxel_x - 1, voxel_y, voxel_z)).is_none() {
                add_triangle(&mut vertices, TriangleSide::Left, voxel_x, voxel_y, voxel_z, 1, *voxel.1);
            }
            
            if voxel_data.get(&(voxel_x + 1, voxel_y, voxel_z)).is_none() {
                add_triangle(&mut vertices, TriangleSide::Right, voxel_x, voxel_y, voxel_z, 1, *voxel.1);
            }

            if voxel_data.get(&(voxel_x, voxel_y - 1, voxel_z)).is_none() {
                add_triangle(&mut vertices, TriangleSide::Bottom, voxel_x, voxel_y, voxel_z, 1, *voxel.1);
            }

            if voxel_data.get(&(voxel_x, voxel_y + 1, voxel_z)).is_none() {
                add_triangle(&mut vertices, TriangleSide::Top, voxel_x, voxel_y, voxel_z, 1, *voxel.1);
            }
            
            if voxel_data.get(&(voxel_x, voxel_y, voxel_z - 1)).is_none() {
                add_triangle(&mut vertices, TriangleSide::Back, voxel_x, voxel_y, voxel_z, 1, *voxel.1);
            }

            if voxel_data.get(&(voxel_x, voxel_y, voxel_z + 1)).is_none() {
                add_triangle(&mut vertices, TriangleSide::Front, voxel_x, voxel_y, voxel_z, 1, *voxel.1);
            }
        }
        return vertices
    }

    pub fn update_mesh(&mut self, render_state : &mut RenderState) {
        let vertices = MeshCreator::vertices_from_voxel_color_hashmap(&self.mesh_voxels);

        self.mesh_buffer_size = vertices.len() as u32;

        let mesh_buffer = render_state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Creator Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        self.mesh_buffer = Some(mesh_buffer);

    }
}

