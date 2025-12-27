use crate::{game::{Game, entity::Entities, pixel::PixelTypes, world::WorldData}, utils::Vec3};


pub struct PhysicsObject {
    pub hitbox : Vec3,
    pub moveable : bool,
    pub velocity : Vec3,
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





pub fn tick_physics(game : &mut Game) {
    //for now just assuming velocity as 1 unit per game tick. Will change to m/s later


    //run for all entities;
    for (_, entity) in &mut game.entities.entities {
        if entity.physics.velocity.x != 0.0 {
            let start_testing_x = entity.position.x + (entity.physics.hitbox.x * entity.physics.velocity.x.signum());
            let end_testing_x = start_testing_x + entity.physics.velocity.x;

            let start_testing_x = start_testing_x.trunc() as i32;
            let end_testing_x = end_testing_x.trunc() as i32;

            //'x_test : for x_pos in start_testing_x..=end_testing_x {
            //    let blocks = game.world.get_area_block_data_unordered((x_pos, (entity.position.0 - entity.physics.hitbox.0).ceil() as i32, (entity.position.2 - entity.physics.hitbox.2).ceil() as i32), (x_pos,(entity.position.0 - entity.physics.hitbox.0).floor() as i32, (entity.position.2 - entity.physics.hitbox.2).floor() as i32));
            //    for block in blocks {
            //        if block != Some(PixelTypes::Air) {
            //            entity.position.0 = (x_pos - 1) as f32;
            //            entity.physics.velocity.0 = 0.0;
            //            break 'x_test;
            //        }
            //    }
            //}

        }

        if entity.physics.velocity.y != 0.0 {
            let start_testing_y = entity.position.y + (entity.physics.hitbox.y * entity.physics.velocity.y.signum());
            let end_testing_y = start_testing_y + entity.physics.velocity.y;

            let start_testing_y = start_testing_y.trunc() as i32;
            let end_testing_y = end_testing_y.trunc() as i32;

            if entity.physics.velocity.y < 0.0 {
                'y_test : for y_pos in start_testing_y..=end_testing_y {
                    let first_corner = Vec3::new((entity.position.x + entity.physics.hitbox.x).ceil(), y_pos as f32, (entity.position.z + entity.physics.hitbox.z).ceil());
                    let second_corner: Vec3 = Vec3::new((entity.position.x - entity.physics.hitbox.x).floor(), y_pos as f32, (entity.position.z - entity.physics.hitbox.z).floor());
                    let collided = game.world.test_collision_area((first_corner.x as i32,first_corner.y as i32,first_corner.z as i32),(second_corner.x as i32,second_corner.y as i32,second_corner.z as i32));
                    if collided {
                        entity.position.y = (y_pos - 1) as f32;
                        entity.physics.velocity.y = 0.0;
                        break 'y_test;
                    }
                }
            }else{

            }

        }
    }   

}