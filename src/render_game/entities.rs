use crate::{entity::EntityClass, utils::Vec3};

pub struct EntityRenderData {
    pub id : u64,
    pub position : Vec3,
    pub entity_class : EntityClass,
    pub alive : bool,
    pub render_mesh_id : u64, //THIS IS VERY TEMP, WILL CHANGE TO REAL SYSTEM LATER BUT FOR NOW WILL WORK
}



pub fn update_entites() {
    loop {
        //let entity_render_update = entity_render_rx.try_recv();
        //match entity_render_update {
        //    Ok(entity_update) => {
        //        entities_to_render.insert(entity_update.id,entity_update);
        //    },
        //    Err(_) => {
        //        break
        //    },
        //}
    }
}

pub fn render_entites() {
    //for entity in &entities_to_render {
    //    match entity.1.entity_type {
    //        EntityType::Player => {
    //            if first_person == false {
    //                draw_cube(vec3(entity.1.position.0,entity.1.position.1,entity.1.position.2), vec3(5.0,12.0,5.0), None, RED);
    //            }
    //        },
    //    }
    //}
}