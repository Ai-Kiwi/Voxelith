use crate::{physics::PhysicsObject, utils::Vec3};



pub struct Entity {
    pub id : u64,
    pub position : Vec3,
    pub physics : PhysicsObject,
    pub entity_class : EntityClass,
    pub render_component : Option<EntityRenderComponent>


    //health
    //mob ai
    //type
    //position
    //rotation
    //physics
    //inventory
}

pub struct EntityRenderComponent {
    pub entity_meshs : Vec<EntityRenderMeshInstance>,
}

pub struct EntityRenderMeshInstance {
    pub mesh_id : u64,
    pub offset : Vec3,
    pub rotation : Vec3 
}

pub enum EntityClass {
    Player,
}