use crate::{game::{world::PixelTypes}, mesh_creation::{ChunkMeshCreateRequest, create_triangles::{TriangleSide, add_triangle}, pix_colors::calculate_pix_color}, utils::{Mesh, Vertex}};




pub fn create_chunk_mesh(request : &ChunkMeshCreateRequest, lod : f32) -> Mesh {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let step = lod as i32;

    for local_x in (0..16).step_by(lod as usize) {
        for local_y in (0..16).step_by(lod as usize) {
            for local_z in (0..16).step_by(lod as usize) {

                let pixel_value = request.chunk.get_relative_pixel(local_x,local_y,local_z);
                if pixel_value != PixelTypes::Air {
                    let world_pixel_x = local_x as f32 + (request.position.0 as f32 * 16.0);
                    let world_pixel_y = local_y as f32 + (request.position.1 as f32 * 16.0);
                    let world_pixel_z = local_z as f32 + (request.position.2 as f32 * 16.0);
                    //println!("{}",world_pixel_x);
                    let color = calculate_pix_color(pixel_value, world_pixel_x as i32, world_pixel_y as i32, world_pixel_z as i32);

                    if local_x == 0 {
                        if let Some(neighbor) = request.x_negative_chunk_neighbor.as_ref() {
                            if neighbor.get_relative_pixel(16 - step, local_y, local_z) == PixelTypes::Air {
                                add_triangle(&mut vertices, &mut indices, TriangleSide::Left, world_pixel_x, world_pixel_y, world_pixel_z, lod, color);
                            }
                        }
                    }else{
                        if request.chunk.get_relative_pixel(local_x - step, local_y, local_z) == PixelTypes::Air {
                            add_triangle(&mut vertices, &mut indices, TriangleSide::Left, world_pixel_x, world_pixel_y, world_pixel_z, lod, color);
                        }
                    }
                    if local_x + step > 15 {
                        if let Some(neighbor) = request.x_positive_chunk_neighbor.as_ref() {
                            if neighbor.get_relative_pixel(-1 + step, local_y, local_z) == PixelTypes::Air {
                                add_triangle(&mut vertices, &mut indices, TriangleSide::Right, world_pixel_x, world_pixel_y, world_pixel_z, lod, color);
                            }
                        }
                    }else{
                        if request.chunk.get_relative_pixel(local_x + step, local_y, local_z) == PixelTypes::Air {
                            add_triangle(&mut vertices, &mut indices, TriangleSide::Right, world_pixel_x, world_pixel_y, world_pixel_z, lod, color);
                        }
                    }

                    if local_y == 0 {
                        if let Some(neighbor) = request.y_negative_chunk_neighbor.as_ref() {
                            if neighbor.get_relative_pixel(local_x, 16 - step, local_z) == PixelTypes::Air {
                                add_triangle(&mut vertices, &mut indices, TriangleSide::Bottom, world_pixel_x, world_pixel_y, world_pixel_z, lod, color);
                            }
                        }
                    }else{
                        if request.chunk.get_relative_pixel(local_x, local_y - step, local_z) == PixelTypes::Air {
                            add_triangle(&mut vertices, &mut indices, TriangleSide::Bottom, world_pixel_x, world_pixel_y, world_pixel_z, lod, color);
                        }
                    }
                    if local_y + step > 15 {
                        if let Some(neighbor) = request.y_positive_chunk_neighbor.as_ref() {
                            if neighbor.get_relative_pixel(local_x, -1 + step, local_z) == PixelTypes::Air {
                                add_triangle(&mut vertices, &mut indices, TriangleSide::Top, world_pixel_x, world_pixel_y, world_pixel_z, lod, color);
                            }
                        }
                    }else{
                        if request.chunk.get_relative_pixel(local_x, local_y + step, local_z) == PixelTypes::Air {
                            add_triangle(&mut vertices, &mut indices, TriangleSide::Top, world_pixel_x, world_pixel_y, world_pixel_z, lod, color);
                        }
                    }

                    
                    if local_z == 0 {
                        if let Some(neighbor) = request.z_negative_chunk_neighbor.as_ref() {
                            if neighbor.get_relative_pixel(local_x, local_y, 16 - step) == PixelTypes::Air {
                                add_triangle(&mut vertices, &mut indices, TriangleSide::Back, world_pixel_x, world_pixel_y, world_pixel_z, lod, color);
                            }
                        }
                    }else{
                        if request.chunk.get_relative_pixel(local_x, local_y, local_z - step) == PixelTypes::Air {
                            add_triangle(&mut vertices, &mut indices, TriangleSide::Back, world_pixel_x, world_pixel_y, world_pixel_z, lod, color);
                        }
                    }
                    if local_z + step > 15 {
                        if let Some(neighbor) = request.z_positive_chunk_neighbor.as_ref() {
                            if neighbor.get_relative_pixel(local_x, local_y, -1 + step) == PixelTypes::Air {
                                add_triangle(&mut vertices, &mut indices, TriangleSide::Front, world_pixel_x, world_pixel_y, world_pixel_z, lod, color);
                            }
                        }
                    }else{
                        if request.chunk.get_relative_pixel(local_x, local_y, local_z + step) == PixelTypes::Air {
                            add_triangle(&mut vertices, &mut indices, TriangleSide::Front, world_pixel_x, world_pixel_y, world_pixel_z, lod, color);
                        }
                    }

                }
            }
        }
    }

    //handle shadows for mesh
    for i in (0..vertices.len()).step_by(4) {
        //let bottom_left = mesh.vertices[i];
        let bottom_right = vertices[i + 1];
        let top_right = vertices[i + 2];
        let top_left = vertices[i + 3];

        let edge_1 = top_right.position - top_left.position;
        let edge_2 = bottom_right.position - top_left.position;
        let cross = edge_2.cross(&edge_1).normalize();
        //println!("{}, {}, {}",edge_1,edge_2,cross);

        for j in i..(i+4) {
            let brightness = (cross.y + 1.0) / 2.0;
            //mesh.vertices[j].normal[0] = brightness;

            //let color: [u8; 4] = if brightness < 0.1 {
            //    [255,0,0,255]
            //}else if brightness > 0.1 {
            //    [255,255,255,255]
            //}else {
            //    [255,128,128,255]
            //};

            //mesh.vertices[j].color = [255,255,255,255];
            vertices[j].extra[0] = (brightness * 255.0) as u8;

            //println!("{}", brightness)
        }


        
    }





    let mesh = Mesh { vertices, indices: indices };
    return mesh
}