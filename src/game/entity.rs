use std::{
    collections::HashMap,
    sync::mpsc::{Sender, channel},
};

use crate::{
    entity::{Entity, EntityRenderComponent},
    game::Game,
    physics::PhysicsObject,
    render::update_state::EntityRenderDataUpdate,
    utils::Vec3,
};

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub struct EntityId(pub u64);

pub struct Entities {
    pub entities: HashMap<EntityId, Entity>,
    pub entities_count: u64,
    pub updated: HashMap<EntityId, ()>,
}

impl Entities {
    pub fn new() -> Entities {
        Entities {
            entities: HashMap::new(),
            entities_count: 0,
            updated: HashMap::new(),
        }
    }
}

pub fn handle_entity_update(game: &mut Game, entity_render_tx: &Sender<EntityRenderDataUpdate>) {
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

#[test]
fn test_entity() {
    let mut game = Game::new();
    for i in 0..5 {
        game.entities.entities.insert(
            EntityId(i),
            Entity {
                id: EntityId(i),
                position: Vec3::new(0.0, 0.0, 0.0),
                physics: PhysicsObject {
                    hitbox: Vec3::new(2.5, 8.5, 2.5),
                    moveable: true,
                    velocity: Vec3::new(0.0, 0.0, 0.0),
                    gravity: true,
                    grounded: false,
                },
                entity_class: crate::entity::EntityClass::Player,
                render_component: Some(EntityRenderComponent {
                    entity_meshs: Vec::new(),
                }),
            },
        );
    }

    game.entities.updated.insert(EntityId(3), ());
    game.entities.updated.insert(EntityId(15), ());
    game.entities.updated.insert(EntityId(1), ());

    assert_eq!(game.entities.updated.len(), 3);
    assert!(game.entities.updated.contains_key(&EntityId(1)));
    assert!(game.entities.updated.contains_key(&EntityId(3)));
    assert!(game.entities.updated.contains_key(&EntityId(15)));

    let (entity_render_tx, entity_render_rx) = channel::<EntityRenderDataUpdate>();
    handle_entity_update(&mut game, &entity_render_tx);

    for _ in 0..2 {
        let result = entity_render_rx.try_recv().unwrap();
        assert!(result.id == EntityId(3) || result.id == EntityId(1));
    }

    assert!(entity_render_rx.try_recv().is_err());
}
