use crate::{entity, game::world::{PixelTypes, WorldData}};


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





pub fn tick_physics(world : &mut WorldData) {

    let mut entities_x_col: Vec<u64> = Vec::new();
    let mut entities_y_col: Vec<u64> = Vec::new();
    let mut entities_z_col: Vec<u64> = Vec::new();

    for entity in &world.entities {
        
        //x
        let mut collision = false;
        for block in get_block_locations_at_position((entity.1.position.0 + entity.1.physics.velocity.0,entity.1.position.1 ,entity.1.position.2 ), entity.1.physics.hitbox) {
            if world.get_pixel_data(block.0,block.1,block.2) != PixelTypes::Air {
                collision = true;
            }
        }
        if collision == true {
            entities_x_col.push(*entity.0);
        }


        //y
        let mut collision = false;
        for block in get_block_locations_at_position((entity.1.position.0,entity.1.position.1 + entity.1.physics.velocity.1,entity.1.position.2 ), entity.1.physics.hitbox) {
            if world.get_pixel_data(block.0,block.1,block.2) != PixelTypes::Air {
                collision = true;
            }
        }
        if collision == true {
            entities_y_col.push(*entity.0);
        }

        //z
        let mut collision = false;
        for block in get_block_locations_at_position((entity.1.position.0,entity.1.position.1,entity.1.position.2 + entity.1.physics.velocity.2 ), entity.1.physics.hitbox) {
            if world.get_pixel_data(block.0,block.1,block.2) != PixelTypes::Air {
                collision = true;
            }
        }
        if collision == true {
            entities_z_col.push(*entity.0);
        }
    }
    for entity in &mut world.entities {
        entity.1.physics.grounded = false;
        if entities_x_col.contains(entity.0) {
            entity.1.physics.velocity.0 = 0.0;
        }else{
            entity.1.position.0 += entity.1.physics.velocity.0;
            entity.1.updated = true;
        }
        if entities_y_col.contains(entity.0) {
            if entity.1.physics.velocity.1 < 0.0 {
                entity.1.physics.grounded = true;
            }
            entity.1.physics.velocity.1 = 0.0;
        }else{
            entity.1.position.1 += entity.1.physics.velocity.1;
            entity.1.updated = true;
        }
        if entities_z_col.contains(entity.0) {
            entity.1.physics.velocity.2 = 0.0;
        }else{
            entity.1.position.2 += entity.1.physics.velocity.2;
            entity.1.updated = true;
        }

        if entity.1.physics.gravity == true {
            entity.1.physics.velocity.1 -= 0.25
        }

        entity.1.physics.velocity.0 *= 0.8;
        entity.1.physics.velocity.1 *= 0.8;
        entity.1.physics.velocity.2 *= 0.8; 
    }

}