use std::sync::Arc;

use wgpu::TexelCopyTextureInfoBase;

use crate::{game::{chunk::Chunk, pixel::PixelTypes}, render::{mesh::create_gpu_mesh, wgpu::RenderState}, render_game::{ChunkInfo, GameData}, utils::Mesh};

pub struct ChunkMeshUpdate {
    pub chunk_pos : (i32,i32,i32),
    pub transparent : bool,
    pub mesh : Option<Mesh>,
    pub mesh_l2 : Option<Mesh>,
    pub mesh_l4 : Option<Mesh>,
    pub mesh_l8 : Option<Mesh>,
    pub data : Arc<Chunk>,
}

pub struct EntityRenderData {
    //    pub id : u64,
    //    pub alive : bool,
    //    pub position : (f32, f32, f32),
    //    pub entity_type : EntityType,
}

pub fn update_render_chunk_mesh(render_state : &mut RenderState, game_data : &mut GameData, chunk_pos : (i32,i32,i32), mesh : Option<&Mesh>, transparent : bool, chunk_data : Arc<Chunk>) {
    let key = (chunk_pos.0,chunk_pos.1,chunk_pos.2,transparent);
    if let Some(mesh_data) = mesh {
        let reference = create_gpu_mesh(render_state, &mesh_data);
        game_data.chunk_meshs.insert(key,ChunkInfo {
            buffer_number: reference.buffer_number.clone(),
            pointer: reference.clone(),
            size : mesh_data.vertices.len(),
        });
        if mesh_data.vertices.len() > 0 {
            game_data.cache_chunk_meshs.insert(key,ChunkInfo {
                buffer_number: reference.buffer_number.clone(),
                pointer: reference,
                size : mesh_data.vertices.len(),
            });        
        }else{
            game_data.cache_chunk_meshs.remove(&key);
        }
    }else{
        game_data.chunk_meshs.remove(&key);
        game_data.cache_chunk_meshs.remove(&key);

    }

    //update gpu reference data
    //x,z,y for order

    let mut texture_chunk_data : Vec<u8> = Vec::with_capacity(16 * 16 * 16);
    for z in 0..16{
        for y in 0..16{
            for x in 0..16{
                let pixel_value = chunk_data.data[x + (z * 16) + (y * 16 * 16)];
                let value = if pixel_value.is_transparent() || pixel_value == PixelTypes::Air {0} else {255};
                texture_chunk_data.push(value);
            }
        }
    };





    if chunk_pos.0 >= 0 && chunk_pos.0 <= 7 && chunk_pos.1 >= 0 && chunk_pos.1 <= 7 && chunk_pos.2 >= 0 && chunk_pos.2 <= 7 {
        render_state.queue.write_texture(
            wgpu::TexelCopyTextureInfoBase {
                texture: &render_state.voxel_map_texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: chunk_pos.0 as u32 * 16,
                    y: chunk_pos.1 as u32 * 16,
                    z: chunk_pos.2 as u32 * 16,
                },
                aspect: wgpu::TextureAspect::All,
            },
            texture_chunk_data.as_slice(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(16),
                rows_per_image: Some(16),
            },
            wgpu::Extent3d {
                width: 16,
                height: 16,
                depth_or_array_layers: 16,
            },
        );
    }
}


pub fn update_chunk_meshs(render_state : &mut RenderState, game_data : &mut GameData ) {
    loop {
        let mesh_update = game_data.render_channels.chunk_mesh_update_rx.try_recv();
        match mesh_update {
            Ok(mesh_update) => {
                update_render_chunk_mesh(render_state, game_data, mesh_update.chunk_pos, mesh_update.mesh.as_ref(), mesh_update.transparent, mesh_update.data.clone());

                game_data.chunk_mesh_data.insert((mesh_update.chunk_pos.0,mesh_update.chunk_pos.1,mesh_update.chunk_pos.2,mesh_update.transparent), mesh_update);
            },
            Err(_) => {
                break
            },
        }
    }
}
