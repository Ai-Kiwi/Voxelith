use crate::{game::{entity::Entities, pixel::PixelTypes, world::WorldData}};


pub struct PhysicsObject {
    pub hitbox : (f32,f32,f32),
    pub moveable : bool,
    pub velocity : (f32,f32,f32),
    pub gravity : bool,
    pub grounded : bool,
}

fn get_block_locations_at_position(position: (f32,f32,f32), hitbox : (f32,f32,f32)) -> Vec<(i32,i32,i32)> {
    let mut locations: Vec<(i32,i32,i32)> = Vec::new();


    for x in ((position.0 - (hitbox.0 / 2.0)).floor() as i32)..=((position.0 + (hitbox.0 / 2.0)).ceil() as i32) {
        for y in ((position.1 - (hitbox.1 / 2.0)).floor() as i32)..=((position.1 + (hitbox.1 / 2.0)).ceil() as i32) {
            for z in ((position.2 - (hitbox.2 / 2.0)).floor() as i32)..=((position.2 + (hitbox.2 / 2.0)).ceil() as i32) {
                locations.push((x,y,z));
                
            }   
        }
    }

    return locations
}





pub fn tick_physics(world : &mut WorldData, entities : &mut Entities) {
    //for now just assuming velocity as 1 unit per game tick. Will change to m/s later


    //run for all entities;
    for (_, entity) in &mut entities.entities {
        if entity.physics.velocity.0 != 0.0 {
            let start_testing_x = entity.position.0 + (entity.physics.hitbox.0 * entity.physics.velocity.0.signum());
            let end_testing_x = start_testing_x + entity.physics.velocity.0;

            let start_testing_x = start_testing_x.trunc() as i32;
            let end_testing_x = end_testing_x.trunc() as i32;

            'x_test : for x_pos in start_testing_x..=end_testing_x {
                let blocks = world.get_area_block_data_unordered((x_pos, (entity.position.0 - entity.physics.hitbox.0).ceil() as i32, (entity.position.2 - entity.physics.hitbox.2).ceil() as i32), (x_pos,(entity.position.0 - entity.physics.hitbox.0).floor() as i32, (entity.position.2 - entity.physics.hitbox.2).floor() as i32));
                for block in blocks {
                    if block != PixelTypes::Air {
                        entity.position.0 = (x_pos - 1) as f32;
                        entity.physics.velocity.0 = 0.0;
                        break 'x_test;
                    }
                }
            }


        }
    }   

}