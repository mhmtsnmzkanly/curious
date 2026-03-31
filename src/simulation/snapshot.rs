use crate::{
    map::movement::Position,
    entity::{species::Species, phase::EntityPhase, lifestate::LifeState},
};

/// Bir varlığın (EntitySlot) serileştirilebilir durum özeti.
/// dyn Entity trait nesnelerinin zorluklarını aşmak için kullanılır.
#[derive(Debug, Clone)]
pub struct EntitySnapshot {
    pub id: usize,
    pub pos: Position,
    pub species: Species,
    pub phase: EntityPhase,
    pub life: LifeState,
}

/// Tüm simülasyonun anlık durum (snapshot) özeti.
/// Bu veri ağ üzerinden yeni bağlanan istemcilere "Initial State" olarak gönderilebilir.
#[derive(Debug, Clone)]
pub struct WorldSnapshot {
    pub tick: usize,
    pub entities: Vec<EntitySnapshot>,
    pub map_resources: Vec<(Position, usize)>, // Sadece yemek/su olan hücreler
}
