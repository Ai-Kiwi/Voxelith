use crate::{physics::PhysicsObject, utils::Vec3};



pub struct Entity {
    pub id : u64,
    pub position : Vec3,
    pub physics : PhysicsObject,
    pub entity_type : EntityType,
    //health
    //mob ai
    //type
    //position
    //rotation
    //physics
    //inventory
}

pub enum EntityType {
    Player
}