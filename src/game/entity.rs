use std::{collections::HashMap, sync::mpsc::Sender};

use crate::{entity::{Entity, EntityClass}, game::Game, utils::Vec3};

pub struct Entities {
    pub entities: HashMap<u64, Entity>,
    pub entities_count: u64,
}


pub struct EntityRenderData {
    pub id : u64,
    pub position : Vec3,
    pub entity_class : EntityClass,
    pub alive : bool,
}


pub fn handle_entity_update(game : &Game, entity_render_tx: &Sender<EntityRenderData>,) {
    //send updates for entities
    for entity in &game.entities.entities {
        let _ = entity_render_tx.send(EntityRenderData {
            id: *entity.0,
            position: entity.1.position,
            entity_class: crate::entity::EntityClass::Player,
            alive: true,
        });
    }
    
}