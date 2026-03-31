use crate::{
    entity::{Entity, phase::EntityPhase, controller::EntityController},
    map::movement::Position,
};

/// Canlının yönetim biçimi ve temel verileri
pub struct EntitySlot {
    /// Canlının benzersiz kimlik numarası
    pub id: usize,
    /// Canlının konumu
    pub pos: Position,
    /// Canlının bulunduğu durum (aktif, uykuda, ölü, silinecek)
    pub phase: EntityPhase,
    /// Canlının yönetim biçimi (AI veya Oyuncu)
    pub controller: EntityController,
    /// Canlının verisi (türüne göre değişen somut nesne)
    pub base: Box<dyn Entity>,
}

impl EntitySlot {
    /// Yeni canlı oluştur
    pub fn new(id: usize, pos: Position, phase: EntityPhase, controller: EntityController, base: Box<dyn Entity>) -> EntitySlot {
        Self {
            id,
            pos,
            phase,
            controller,
            base,
        }
    }

    /// Canlının bulunduğu konumu döndürür (okumak için)
    pub fn position(&self) -> &Position {
        &self.pos
    }

    /// Canlının bulunduğu konumu döndürür (değiştirmek için)
    pub fn position_mut(&mut self) -> &mut Position {
        &mut self.pos
    }

    /// Canlı verisini döndürür (okumak için)
    pub fn entity(&self) -> &dyn Entity {
        self.base.as_ref()
    }

    /// Canlı verisini döndürür (yazmak için)
    pub fn entity_mut(&mut self) -> &mut dyn Entity {
        self.base.as_mut()
    }

    /// Canlı durumunu (fazını) döndürür
    pub fn phase(&self) -> &EntityPhase {
        &self.phase
    }

    /// Canlı durumunu değiştirmek için döndürür
    pub fn phase_mut(&mut self) -> &mut EntityPhase {
        &mut self.phase
    }
}
