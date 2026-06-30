use std::sync::Arc;

use winit::dpi::Position;

use crate::{game::chunk::Chunk, render::{mesh::create_gpu_mesh, wgpu::{ChunkInfo, RenderState}}, utils::Mesh};

pub fn update_render_chunk_mesh(render_state : &mut RenderState, chunk_pos : (i32,i32,i32), mesh : Option<&Mesh>, transparent : bool) {
    let key = (chunk_pos.0,chunk_pos.1,chunk_pos.2,transparent);
    if let Some(mesh_data) = mesh && mesh_data.vertices.len() > 0 {
        let reference = create_gpu_mesh(render_state, &mesh_data);
        let chunk_info = ChunkInfo {
            buffer_number: reference.buffer_number.clone(),
            pointer: reference.clone(),
            size : mesh_data.vertices.len(),
            position: key,
        };

        let old_chunk_loc: Option<&usize> = render_state.chunk_meshs_loc.get(&key);
        match old_chunk_loc {
            Some(old_loc) => {
                render_state.chunk_meshs[*old_loc] = chunk_info;
            },
            None => {
                render_state.chunk_meshs_loc.insert(chunk_info.position, render_state.chunk_meshs.len());
                render_state.chunk_meshs.push(chunk_info);
            },
        }
    }else{
        let old_chunk_loc = render_state.chunk_meshs_loc.remove(&key);
        match old_chunk_loc {
            Some(old_loc) => {
                render_state.chunk_meshs.swap_remove(old_loc);
                let old_replaced = render_state.chunk_meshs.get_mut(old_loc);
                match old_replaced {
                    Some(replaced) => {
                        let position = replaced.position;
                        render_state.chunk_meshs_loc.insert(position, old_loc);
                    },
                    None => (),
                }
            },
            None => (),
        }
    }

    //update gpu reference data
    //x,z,y for order

    //let mut texture_chunk_data : Vec<u8> = Vec::with_capacity(16 * 16 * 16);
    //for z in 0..16{
    //    for y in 0..16{
    //        for x in 0..16{
    //            let pixel_value = chunk_data.data[x + (z * 16) + (y * 16 * 16)];
    //            let value = if pixel_value.is_transparent() || pixel_value == PixelTypes::Air {0} else {255};
    //            texture_chunk_data.push(value);
    //        }
    //    }
    //};
}


pub fn update_chunk_meshs(render_state : &mut RenderState ) {
    loop {
        let mesh_update = render_state.render_channels.chunk_mesh_update_rx.try_recv();
        match mesh_update {
            Ok(mesh_update) => {
                update_render_chunk_mesh(render_state, mesh_update.chunk_pos, mesh_update.mesh.as_ref(), mesh_update.transparent);

                render_state.chunk_mesh_data.insert((mesh_update.chunk_pos.0,mesh_update.chunk_pos.1,mesh_update.chunk_pos.2,mesh_update.transparent), mesh_update);
            },
            Err(_) => {
                break
            },
        }
    }
}
