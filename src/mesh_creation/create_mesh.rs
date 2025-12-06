use crate::{game::pixel::PixelTypes, mesh_creation::{ChunkMeshCreateRequest, create_triangles::{TriangleSide, add_triangle}, pix_colors::calculate_pix_color}, utils::{Material, Mesh, Vertex}};




pub fn create_chunk_mesh(request : &ChunkMeshCreateRequest, lod: i32, transparent : bool) -> Mesh {
    let mut vertices: Vec<Vertex> = Vec::new();
    let step = lod;

    for local_x in (0..16).step_by(lod as usize) {
        for local_y in (0..16).step_by(lod as usize) {
            for local_z in (0..16).step_by(lod as usize) {
                let pixel_value = request.chunk.get_relative_pixel(local_x,local_y,local_z);
                if transparent == true {
                    if pixel_value.is_transparent() == false || pixel_value == PixelTypes::Air {
                        continue;
                    }
                }else{
                    if pixel_value.is_transparent() == true || pixel_value == PixelTypes::Air {
                        continue;
                    }
                }

                let world_pixel_x = local_x + (request.position.0 * 16);
                let world_pixel_y = local_y + (request.position.1 * 16);
                let world_pixel_z = local_z + (request.position.2 * 16);
                //println!("{}",world_pixel_x);
                let (color, material) = calculate_pix_color(pixel_value, world_pixel_x as i32, world_pixel_y as i32, world_pixel_z as i32);
                

                if local_x == 0 {
                    if let Some(neighbor) = request.x_negative_chunk_neighbor.as_ref() {
                        if neighbor.get_relative_pixel(16 - step, local_y, local_z).should_be_face(transparent) {
                            add_triangle(&mut vertices, TriangleSide::Left, world_pixel_x, world_pixel_y, world_pixel_z, lod, &color, &material);
                        }
                    }
                }else{
                    if request.chunk.get_relative_pixel(local_x - step, local_y, local_z).should_be_face(transparent) {
                        add_triangle(&mut vertices, TriangleSide::Left, world_pixel_x, world_pixel_y, world_pixel_z, lod, &color, &material);
                    }
                }
                if local_x + step > 15 {
                    if let Some(neighbor) = request.x_positive_chunk_neighbor.as_ref() {
                        if neighbor.get_relative_pixel(-1 + step, local_y, local_z).should_be_face(transparent)  {
                            add_triangle(&mut vertices, TriangleSide::Right, world_pixel_x, world_pixel_y, world_pixel_z, lod, &color, &material);
                        }
                    }
                }else{
                    if request.chunk.get_relative_pixel(local_x + step, local_y, local_z).should_be_face(transparent)  {
                        add_triangle(&mut vertices, TriangleSide::Right, world_pixel_x, world_pixel_y, world_pixel_z, lod, &color, &material);
                    }
                }

                if local_y == 0 {
                    if let Some(neighbor) = request.y_negative_chunk_neighbor.as_ref() {
                        if neighbor.get_relative_pixel(local_x, 16 - step, local_z).should_be_face(transparent)  {
                            add_triangle(&mut vertices, TriangleSide::Bottom, world_pixel_x, world_pixel_y, world_pixel_z, lod, &color, &material);
                        }
                    }
                }else{
                    if request.chunk.get_relative_pixel(local_x, local_y - step, local_z).should_be_face(transparent)  {
                        add_triangle(&mut vertices, TriangleSide::Bottom, world_pixel_x, world_pixel_y, world_pixel_z, lod, &color, &material);
                    }
                }
                if local_y + step > 15 {
                    if let Some(neighbor) = request.y_positive_chunk_neighbor.as_ref() {
                        if neighbor.get_relative_pixel(local_x, -1 + step, local_z).should_be_face(transparent)  {
                            add_triangle(&mut vertices, TriangleSide::Top, world_pixel_x, world_pixel_y, world_pixel_z, lod, &color, &material);
                        }
                    }
                }else{
                    if request.chunk.get_relative_pixel(local_x, local_y + step, local_z).should_be_face(transparent)  {
                        add_triangle(&mut vertices, TriangleSide::Top, world_pixel_x, world_pixel_y, world_pixel_z, lod, &color, &material);
                    }
                }

                
                if local_z == 0 {
                    if let Some(neighbor) = request.z_negative_chunk_neighbor.as_ref() {
                        if neighbor.get_relative_pixel(local_x, local_y, 16 - step).should_be_face(transparent)  {
                            add_triangle(&mut vertices, TriangleSide::Back, world_pixel_x, world_pixel_y, world_pixel_z, lod, &color, &material);
                        }
                    }
                }else{
                    if request.chunk.get_relative_pixel(local_x, local_y, local_z - step).should_be_face(transparent)  {
                        add_triangle(&mut vertices, TriangleSide::Back, world_pixel_x, world_pixel_y, world_pixel_z, lod, &color, &material);
                    }
                }
                if local_z + step > 15 {
                    if let Some(neighbor) = request.z_positive_chunk_neighbor.as_ref() {
                        if neighbor.get_relative_pixel(local_x, local_y, -1 + step).should_be_face(transparent)  {
                            add_triangle(&mut vertices, TriangleSide::Front, world_pixel_x, world_pixel_y, world_pixel_z, lod, &color, &material);
                        }
                    }
                }else{
                    if request.chunk.get_relative_pixel(local_x, local_y, local_z + step).should_be_face(transparent)  {
                        add_triangle(&mut vertices, TriangleSide::Front, world_pixel_x, world_pixel_y, world_pixel_z, lod, &color, &material);
                    }
                }
            }
        }
    }





    let mesh = Mesh { vertices };
    return mesh
}