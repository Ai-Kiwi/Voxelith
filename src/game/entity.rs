use std::{collections::HashMap, sync::mpsc::Sender};

use crate::{entity::{Entity, EntityClass}, game::Game, render_game::entities::{EntityRenderData, EntityRenderDataUpdate}, utils::Vec3};

pub struct Entities {
    pub entities: HashMap<u64, Entity>,
    pub entities_count: u64,
}


pub fn handle_entity_update(game : &Game, entity_render_tx: &Sender<EntityRenderDataUpdate>,) {
    //send updates for entities
    for entity in &game.entities.entities {
        let _ = entity_render_tx.send(EntityRenderDataUpdate {
            id: *entity.0,
            position: entity.1.position,
            entity_class: crate::entity::EntityClass::Player,
        });
    }
    
}