use crate::physics::PhysicsObject;



pub struct Entity {
    pub id : u64,
    pub position : (f32,f32,f32),
    pub physics : PhysicsObject,
    pub entity_type : EntityType,
    //health
    //mob ai
    //type
    //position
    //rotation
    //physics
    //inventory
    pub updated : bool
}

pub enum EntityType {
    Player
}