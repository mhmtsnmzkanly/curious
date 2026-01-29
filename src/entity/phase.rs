#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityPhase {
    /// Aktif, karar alabilir
    Active,

    /// Uyuyor, N tick boyunca aksiyon yok
    Sleeping { remaining: usize },

    /// Ölü ama henüz temizlenmedi
    Corpse,

    /// World tarafından kaldırılacak
    Removed,
}

impl EntityPhase {
    pub fn is_active(&self) -> bool {
        matches!(self, EntityPhase::Active)
    }

    pub fn is_corpse(&self) -> bool {
        matches!(self, EntityPhase::Corpse)
    }
}
