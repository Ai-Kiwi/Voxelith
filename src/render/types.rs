use crate::utils::Mesh;

pub struct ChunkMeshUpdate {
    pub chunk_pos : (i32,i32,i32),
    pub transparent : bool,
    pub mesh : Option<Mesh>,
    pub mesh_l2 : Option<Mesh>,
    pub mesh_l4 : Option<Mesh>,
    pub mesh_l8 : Option<Mesh>
}

pub struct EntityRenderData {
    //    pub id : u64,
    //    pub alive : bool,
    //    pub position : (f32, f32, f32),
    //    pub entity_type : EntityType,
}