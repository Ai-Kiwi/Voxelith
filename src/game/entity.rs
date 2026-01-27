use std::{collections::HashMap, sync::mpsc::Sender};

use crate::{entity::{Entity, EntityClass}, game::Game, render_game::entities::{EntityRenderData, EntityRenderDataUpdate}, utils::Vec3};

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub struct EntityId (pub u64);

pub struct Entities {
    pub entities: HashMap<EntityId, Entity>,
    pub entities_count: u64,
    pub updated : HashMap<EntityId,()>
}


pub fn handle_entity_update(game : &mut Game, entity_render_tx: &Sender<EntityRenderDataUpdate>,) {
    //send updates for entities
    for entity in &game.entities.updated {
        if let Some(entity_data) = game.entities.entities.get(entity.0) {
            let _ = entity_render_tx.send(EntityRenderDataUpdate {
                id: *entity.0,
                position: entity_data.position,
                entity_class: crate::entity::EntityClass::Player,
            });
        }
    }
    game.entities.updated.clear();
}