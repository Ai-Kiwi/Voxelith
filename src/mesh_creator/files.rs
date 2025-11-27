use std::{collections::HashMap, fs};

use egui_wgpu::RenderState;
use wgpu::{Buffer, util::DeviceExt};

use crate::{mesh_creator::MeshCreator, utils::Color};

impl MeshCreator {
    pub fn load_mesh_file_to_hashmap(path : &str) -> Option<HashMap<(i32,i32,i32),Color>> {
        let file = fs::read(path);
        match file {
            Ok(value) => {
                let data = value.as_slice();

                //handle data
                let decode_data: HashMap<(i32,i32,i32),Color> = match bincode::decode_from_slice(data, bincode::config::standard()) {
                    Ok((map, _)) => map,
                    Err(_) => return None,
                };

                return Some(decode_data);
            },
            Err(_) => {
                return None;
            },
        }
    }

    pub fn load_mesh_file_to_buffer(path : &str, render_state : RenderState) -> Option<Buffer> {
        let hashmap_data = MeshCreator::load_mesh_file_to_hashmap(path);
        match hashmap_data {
            Some(data) => {
                let vertices = MeshCreator::vertices_from_voxel_color_hashmap(&data);
                let mesh_buffer = render_state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Loaded Mesh Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                return Some(mesh_buffer)
            },
            None => {
                return None
            },
        }
    }

    pub fn save_mesh_hashmap_to_file(path : &str, voxel_hashmap : &HashMap<(i32,i32,i32),Color> ) -> Result<(), std::io::Error> {
        // serialize to a Vec<u8> first, then write to the file
        let bytes = match bincode::encode_to_vec(voxel_hashmap, bincode::config::standard()) {
            Ok(v) => v,
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("bincode encode error: {}", e))),
        };
        
        fs::write(path, bytes)
    }
}