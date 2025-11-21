use std::collections::HashMap;

use crate::entity::Entity;

pub struct Entities {
    pub entities: HashMap<u64, Entity>,
    pub entities_count: u64,
}


pub fn handle_entity_update() {
    //send updates for entities
    //for entity in &mut world.entities {
    //    if entity.1.updated == true {
    //        let _ = entity_render_tx.send(EntityRenderData {
    //            id: *entity.0,
    //            alive: true,
    //            position: entity.1.position,
    //            entity_type: crate::entity::EntityType::Player,
    //        });
    //        entity.1.updated = false;
    //    }
    //}
}