use crate::{entity::Entity, game::chunk::Chunk};

pub struct DebugGuiData {
    pub selected_info : SelectedGuiDebugItem,
}

pub enum SelectedGuiDebugItem {
    None,
    Chunk(Chunk),
    Entity(Entity)
}