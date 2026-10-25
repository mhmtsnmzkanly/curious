use crate::entity::species::Species;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorpseStage {
    Fresh,
    Aging,
    Putrid,
}

impl CorpseStage {
    pub fn scavenging_toxicity(self, species: Species) -> u16 {
        match species {
            Species::Omnivore => 0,
            Species::Carnivore => match self {
                CorpseStage::Fresh | CorpseStage::Aging => 0,
                CorpseStage::Putrid => 4,
            },
            Species::Herbivore => 4,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
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

    /// Cesedin çürüme aşaması
    pub fn corpse_stage(&self) -> Option<CorpseStage> {
        match self {
            EntityPhase::Corpse { remaining } => {
                if *remaining >= 4 {
                    Some(CorpseStage::Fresh)
                } else if *remaining >= 2 {
                    Some(CorpseStage::Aging)
                } else {
                    Some(CorpseStage::Putrid)
                }
            }
            _ => None,
        }
    }

    /// World için tick kolaylığı ve otomatik durum güncellemesi
    pub fn tick(&mut self) {
        match self {
            // Uyuyorsa zamanı düşür, dolduysa sonra ki aşamaya geçir
            EntityPhase::Sleeping { remaining } => {
                if *remaining > 0 {
                    *remaining -= 1;
                } else {
                    *self = EntityPhase::Active;
                }
            }
            // Cesedin ortadan kalkması gereken süreyi düşür,
            // Bittiyse sisteme kaldırası gerektiğini bildir
            EntityPhase::Corpse { remaining } => {
                if *remaining > 0 {
                    *remaining -= 1;
                } else {
                    *self = EntityPhase::Removed;
                }
            }
            _ => {}
        }
    }
}
