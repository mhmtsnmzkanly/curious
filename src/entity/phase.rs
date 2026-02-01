#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityPhase {
    /// Aktif, karar alabilir
    Active,

    /// Uyuyor, "remaining" tick boyunca aksiyon yok
    Sleeping { remaining: usize },

    /// Ölü, "remaining" tick sonra kaldırılacak
    Corpse { remaining: usize },

    /// World tarafından kaldırılacak
    Removed,
}

impl EntityPhase {
    /// Canlı karar verebilir mi?
    pub fn is_active(&self) -> bool {
        matches!(self, EntityPhase::Active)
    }

    /// Yaşıyor mu? Ölü mü?
    pub fn is_corpse(&self) -> bool {
        matches!(self, EntityPhase::Corpse { .. })
    }

    /// Uyuyor mu?
    pub fn is_sleeping(&self) -> bool {
        matches!(self, EntityPhase::Sleeping { .. })
    }

    /// Kaldırılmasına gerek var mı?
    pub fn need_remove(&self) -> bool {
        matches!(self, EntityPhase::Removed)
    }


    /// World için tick kolaylığı ve otomatik durum güncellemesi
    pub fn tick(&mut self) {
        match self {
            EntityPhase::Sleeping { remaining } => {
                if *remaining > 0 {
                    *remaining -= 1;
                } else {
                    *self = EntityPhase::Active;
                }
            },
            EntityPhase::Corpse { remaining } => {
                if *remaining > 0 {
                    *remaining -= 1;
                } else {
                    *self = EntityPhase::Removed;
                }
            },
            _ => {}
        }
    }
}
