use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, mpsc::Sender},
};

use crate::{
    game::{
        chunk::Chunk,
        world::{self, WorldData},
    },
    render::update_state::LoseObjectRenderDataUpdate,
    utils::Vec3,
};

pub struct LoseTerrain {
    pub objects: HashMap<LoseTerrainId, LoseObject>,
    pub object_count: u64,
    pub position_updated: HashSet<LoseTerrainId>,
    pub mesh_updated: HashSet<LoseTerrainId>,
}

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub struct LoseTerrainId(pub u64);

pub struct LoseObject {
    pub position: Vec3,
    pub velocity: Vec3,
    pub id: LoseTerrainId,
    pub min_max_x: (i32, i32),
    pub min_max_y: (i32, i32),
    pub min_max_z: (i32, i32),
    pub chunks: Vec<Arc<Chunk>>,
    pub chunks_loc: HashMap<(i32, i32, i32), usize>,
    pub chunks_updated: HashSet<(i32, i32, i32)>, //center_of_mass : (f32,f32,f32)
                                                  //rotation
                                                  //bounding_box
}

impl LoseObject {
    pub fn find_lose_terrain_collision(
        &mut self,
        world_data: &WorldData,
    ) -> Vec<((i32, i32, i32), (i32, i32, i32))> {
        let rigidbody_chunk_list: Vec<(&(i32, i32, i32), &Arc<Chunk>)> = self
            .chunks_loc
            .iter()
            .filter(|chunk| {
                let mut valid = false;

                let l_chunk_x = chunk.0.0;
                let l_chunk_y = chunk.0.1;
                let l_chunk_z = chunk.0.2;

                valid = (l_chunk_x as f32 * 16.0) - self.velocity.x
                    < ((self.min_max_x.0 as f32) * 16.0)
                    || valid;
                valid = (l_chunk_x as f32 * 16.0) - self.velocity.x
                    > ((self.min_max_x.1 as f32) * 16.0)
                    || valid;

                valid = (l_chunk_y as f32 * 16.0) - self.velocity.y
                    < ((self.min_max_y.0 as f32) * 16.0)
                    || valid;
                valid = (l_chunk_y as f32 * 16.0) - self.velocity.y
                    > ((self.min_max_y.1 as f32) * 16.0)
                    || valid;

                valid = (l_chunk_z as f32 * 16.0) - self.velocity.z
                    < ((self.min_max_z.0 as f32) * 16.0)
                    || valid;
                valid = (l_chunk_z as f32 * 16.0) - self.velocity.z
                    > ((self.min_max_z.1 as f32) * 16.0)
                    || valid;

                //println!("{} {} {}", l_chunk_x, l_chunk_y, l_chunk_z);
                //println!("{}", valid);

                valid
            })
            .map(|chunk| {
                let chunk_data = self.chunks.get(*chunk.1).unwrap();

                (chunk.0, chunk_data)
            })
            .filter(|chunk| chunk.1.solid_items > 0)
            .collect();

        let mut touching_world_chunks: HashSet<(i32, i32, i32)> = HashSet::new();

        for l_chunk in rigidbody_chunk_list {
            let w_chunk_x = (l_chunk.0.0 as f32) + (self.position.x / 16.0);
            let w_chunk_y = (l_chunk.0.1 as f32) + (self.position.y / 16.0);
            let w_chunk_z = (l_chunk.0.2 as f32) + (self.position.z / 16.0);

            //keeps in mind if it is same it duplicates. This lowers amount to test
            let w_chunk_x_min = w_chunk_x.floor() as i32;
            let w_chunk_x_max = w_chunk_x.ceil() as i32;

            let w_chunk_y_min = w_chunk_y.floor() as i32;
            let w_chunk_y_max = w_chunk_x.ceil() as i32;

            let w_chunk_z_min = w_chunk_z.floor() as i32;
            let w_chunk_z_max = w_chunk_x.ceil() as i32;

            touching_world_chunks.insert((w_chunk_x_min, w_chunk_y_min, w_chunk_z_min));
            touching_world_chunks.insert((w_chunk_x_max, w_chunk_y_min, w_chunk_z_min));
            touching_world_chunks.insert((w_chunk_x_min, w_chunk_y_max, w_chunk_z_min));
            touching_world_chunks.insert((w_chunk_x_max, w_chunk_y_max, w_chunk_z_min));
            touching_world_chunks.insert((w_chunk_x_min, w_chunk_y_min, w_chunk_z_max));
            touching_world_chunks.insert((w_chunk_x_max, w_chunk_y_min, w_chunk_z_max));
            touching_world_chunks.insert((w_chunk_x_min, w_chunk_y_max, w_chunk_z_max));
            touching_world_chunks.insert((w_chunk_x_max, w_chunk_y_max, w_chunk_z_max));
        }

        //println!("touching world chunk count : {}", touching_world_chunks.len());
        //for item in &touching_world_chunks {
        //    println!("{} {} {}", item.0, item.1, item.2);
        //}

        let world_chunk_list: Vec<(&(i32, i32, i32), &Arc<Chunk>)> = touching_world_chunks
            .iter()
            .filter_map(|chunk_pos| {
                let chunk_data = world_data.chunks.get(chunk_pos)?;

                Some((chunk_pos, chunk_data))
            })
            .filter(|chunk| chunk.1.solid_items > 0)
            .collect();

        //println!("testing world chunks with data : {}", world_chunk_list.len());

        let world_chunk_references_to_lose_object: HashMap<
            &(i32, i32, i32),
            Vec<Option<&Arc<Chunk>>>,
        > = world_chunk_list
            .iter()
            .map(|w_chunk| {
                let mut reference_chunks: Vec<Option<&Arc<Chunk>>> = Vec::new();

                let l_chunk_x = (w_chunk.0.0 as f32) - (self.position.x / 16.0);
                let l_chunk_y = (w_chunk.0.1 as f32) - (self.position.y / 16.0);
                let l_chunk_z = (w_chunk.0.2 as f32) - (self.position.z / 16.0);

                let l_chunk_x_min = l_chunk_x.floor() as i32;
                let l_chunk_x_max = l_chunk_x_min + 1;

                let l_chunk_y_min = l_chunk_y.floor() as i32;
                let l_chunk_y_max = l_chunk_y_min + 1;

                let l_chunk_z_min = l_chunk_z.floor() as i32;
                let l_chunk_z_max = l_chunk_z_min + 1;

                reference_chunks.push(
                    self.chunks_loc
                        .get(&(l_chunk_x_min, l_chunk_y_min, l_chunk_z_min))
                        .and_then(|index| self.chunks.get(*index)),
                );
                reference_chunks.push(
                    self.chunks_loc
                        .get(&(l_chunk_x_max, l_chunk_y_min, l_chunk_z_min))
                        .and_then(|index| self.chunks.get(*index)),
                );
                reference_chunks.push(
                    self.chunks_loc
                        .get(&(l_chunk_x_min, l_chunk_y_max, l_chunk_z_min))
                        .and_then(|index| self.chunks.get(*index)),
                );
                reference_chunks.push(
                    self.chunks_loc
                        .get(&(l_chunk_x_max, l_chunk_y_max, l_chunk_z_min))
                        .and_then(|index| self.chunks.get(*index)),
                );
                reference_chunks.push(
                    self.chunks_loc
                        .get(&(l_chunk_x_min, l_chunk_y_min, l_chunk_z_max))
                        .and_then(|index| self.chunks.get(*index)),
                );
                reference_chunks.push(
                    self.chunks_loc
                        .get(&(l_chunk_x_max, l_chunk_y_min, l_chunk_z_max))
                        .and_then(|index| self.chunks.get(*index)),
                );
                reference_chunks.push(
                    self.chunks_loc
                        .get(&(l_chunk_x_min, l_chunk_y_max, l_chunk_z_max))
                        .and_then(|index| self.chunks.get(*index)),
                );
                reference_chunks.push(
                    self.chunks_loc
                        .get(&(l_chunk_x_max, l_chunk_y_max, l_chunk_z_max))
                        .and_then(|index| self.chunks.get(*index)),
                );

                (w_chunk.0, reference_chunks)
            })
            .collect();

        //println!("reference chunks : {}", world_chunk_references_to_lose_object.len());
        //for item in &world_chunk_references_to_lose_object {
        //    println!("pos : {} {} {}", item.0.0, item.0.1, item.0.2);
        //    for item_child in item.1 {
        //        match item_child {
        //            Some(value) => println!("{}", value.solid_items),
        //            None => println!("-"),
        //        }
        //    }
        //}

        let mut collision_points: Vec<((i32, i32, i32), (i32, i32, i32))> = Vec::new();

        for chunk in world_chunk_list {
            let w_chunk_references_l_chunk =
                world_chunk_references_to_lose_object.get(&chunk.0).unwrap();
            let l_chunk_min_x = ((chunk.0.0 as f32) - (self.position.x / 16.0)).floor() as i32;
            let l_chunk_min_y = ((chunk.0.1 as f32) - (self.position.y / 16.0)).floor() as i32;
            let l_chunk_min_z = ((chunk.0.2 as f32) - (self.position.z / 16.0)).floor() as i32;

            let l_base_voxel_x = ((chunk.0.0 * 16) as f32 - self.position.x).floor() as i32;
            let l_base_voxel_y = ((chunk.0.1 * 16) as f32 - self.position.y).floor() as i32;
            let l_base_voxel_z = ((chunk.0.2 * 16) as f32 - self.position.z).floor() as i32;

            for y in 0..16 as i32 {
                for z in 0..16 as i32 {
                    for x in 0..16 as i32 {
                        let l_chunk_x = (l_base_voxel_x + x).div_euclid(16);
                        let l_chunk_y = (l_base_voxel_y + y).div_euclid(16);
                        let l_chunk_z = (l_base_voxel_z + z).div_euclid(16);

                        let l_chunk_local_voxel_x = (l_base_voxel_x + x).rem_euclid(16);
                        let l_chunk_local_voxel_y = (l_base_voxel_y + y).rem_euclid(16);
                        let l_chunk_local_voxel_z = (l_base_voxel_z + z).rem_euclid(16);

                        //

                        let rigidbody_location = (
                            (l_base_voxel_x + x),
                            (l_base_voxel_y + y),
                            (l_base_voxel_z + z),
                        );
                        let world_location = (
                            ((chunk.0.0 * 16) + x),
                            ((chunk.0.1 * 16) + y),
                            ((chunk.0.2 * 16) + z),
                        );

                        let mut offset: usize = 0;
                        offset += (l_chunk_x != l_chunk_min_x) as usize;
                        offset += (l_chunk_y != l_chunk_min_y) as usize * 2;
                        offset += (l_chunk_z != l_chunk_min_z) as usize * 4;

                        let self_solid = chunk.1.get_relative_pixel(x, y, z).is_solid();

                        if let Some(chunk) = w_chunk_references_l_chunk.get(offset).unwrap() {
                            //println!("{}", chunk.solid_items);
                            if chunk
                                .get_relative_pixel(
                                    l_chunk_local_voxel_x,
                                    l_chunk_local_voxel_y,
                                    l_chunk_local_voxel_z,
                                )
                                .is_solid()
                                == true
                                && self_solid == true
                            {
                                //println!("MOD {} {} {}", l_chunk_local_voxel_x, l_chunk_local_voxel_y, l_chunk_local_voxel_z);
                                //println!("[[[{}",chunk.get_relative_pixel(l_chunk_local_voxel_x, l_chunk_local_voxel_y, l_chunk_local_voxel_z) as usize);
                                collision_points.push((rigidbody_location, world_location));
                            }
                        }
                    }
                }
            }
        }

        //get list of world chunks touching those chunks
        //filter world chunks to only solid ones.
        //for all voxels in world chunks see if its touching one on the rigidbody.
        //keep a list of all places a collision occurred.
        return collision_points;
    }

    pub fn tick_physics(&mut self, world_data: &mut WorldData) -> bool {
        let contacts = self.find_lose_terrain_collision(world_data);
        for contact in &contacts {
            world_data.set_pixel_data(contact.1, super::pixel::PixelTypes::Sand);
        }
        let if_len = (contacts.len() == 0) as u32 as f32;
        self.position.x += self.velocity.x * if_len;
        self.position.y += self.velocity.y * if_len;
        self.position.z += self.velocity.z * if_len;
        return true;
    }
}
//current limits
//only does by corner
//no rotation support
//only chunk level for testing, no chunks chunks.

//find touching world chunks from bounding box
//find which of these chunks have something solid
//find which chunks of fallen terrain have something solid

pub fn handle_lose_object_update(
    lose_objects: &mut LoseTerrain,
    entity_render_tx: &Sender<LoseObjectRenderDataUpdate>,
) {
    //send updates for entities
    for object_id in &lose_objects.position_updated {
        if let Some(entity_data) = lose_objects.objects.get(object_id) {
            let _ = entity_render_tx.send(LoseObjectRenderDataUpdate {
                id: *object_id,
                position: entity_data.position,
            });
        }
    }
    lose_objects.position_updated.clear();
}
