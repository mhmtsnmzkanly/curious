use crate::{
    map::{cell::Cell, position::Position},
    world::WorldView,
};

/// Algılanan tekil hedef
#[derive(Debug, Clone)]
pub struct PerceivedEntity {
    pub id: usize,
    pub position: Position,
}

/// Algılanan yiyecek
#[derive(Debug, Clone)]
pub struct PerceivedFood {
    pub position: Position,
    pub amount: usize,
    pub is_corpse: bool, // etçil için önemli
}

/// Entity'nin bir tick boyunca algıladığı dünya kesiti
#[derive(Debug, Clone)]
pub struct Perception {
    /// Görülen yiyecekler
    pub foods: Vec<PerceivedFood>,

    /// Tehdit olarak algılanan canlılar
    pub enemies: Vec<PerceivedEntity>,

    /// Çiftleşme için uygun görülen canlılar
    pub mates: Vec<PerceivedEntity>,
}

impl Perception {
    pub fn empty() -> Self {
        Self {
            foods: Vec::new(),
            enemies: Vec::new(),
            mates: Vec::new(),
        }
    }

    pub fn has_food(&self) -> bool {
        !self.foods.is_empty()
    }

    pub fn has_enemy(&self) -> bool {
        !self.enemies.is_empty()
    }

    pub fn has_mate(&self) -> bool {
        !self.mates.is_empty()
    }
}

/// ===============================
/// PERCEPTION BUILDER
/// ===============================
///
/// Bu modülün tek görevi:
/// - WorldView'dan OKUMA yapmak
/// - Entity için anlamlı algı (Perception) üretmek
///
/// Entity:
/// - Haritayı bilmez
/// - EntityPos bilmez
/// - Faz bilgisi bilmez
///
/// Sadece "algıladıklarını" bilir.
pub struct PerceptionBuilder;

impl PerceptionBuilder {
    /// ===============================
    /// ALGILAMA OLUŞTUR
    /// ===============================
    ///
    /// center  : Entity'nin pozisyonu
    /// radius  : Algılama menzili
    pub fn build(view: &WorldView, center: Position, radius: usize, self_id: usize) -> Perception {
        let mut perception = Perception::empty();

        // ===============================
        // 1. YAKIN ENTITY'LER
        // ===============================
        for (pos, id) in view.nearby_entities(center, radius) {
            // Kendini algılamasın
            if id == self_id {
                continue;
            }

            // Ceset mi?
            if view.entity_phase.get(&id).is_some_and(|p| p.is_corpse()) {
                // Cesetler etçil için "yiyecek"tir
                perception.foods.push(PerceivedFood {
                    position: pos,
                    amount: 10, // sabit veya ileride hesaplanabilir
                    is_corpse: true,
                });
                continue;
            }

            // Canlı entity
            if view.entity_phase.get(&id).is_some_and(|p| p.is_active()) {
                perception
                    .enemies
                    .push(PerceivedEntity { id, position: pos });

                // Aynı zamanda potansiyel eş olabilir
                perception.mates.push(PerceivedEntity { id, position: pos });
            }
        }

        // ===============================
        // 2. YAKIN ÇEVRE HÜCRELERİ
        // ===============================
        //
        // Kare alan taraması (Manhattan)
        for dx in 0..=radius {
            for dy in 0..=radius {
                let positions = [
                    (center.x + dx, center.y + dy),
                    (center.x + dx, center.y.saturating_sub(dy)),
                    (center.x.saturating_sub(dx), center.y + dy),
                    (center.x.saturating_sub(dx), center.y.saturating_sub(dy)),
                ];

                for (x, y) in positions {
                    let pos = Position { x, y };

                    if !view.in_bounds(pos) {
                        continue;
                    }

                    // Hücre içeriği
                    match view.cell(pos) {
                        Some(Cell::Food { amount }) => {
                            perception.foods.push(PerceivedFood {
                                position: pos,
                                amount: *amount,
                                is_corpse: false,
                            });
                        }
                        Some(Cell::Water { amount }) => {
                            perception.foods.push(PerceivedFood {
                                position: pos,
                                amount: *amount,
                                is_corpse: false,
                            });
                        }
                        _ => {}
                    }
                }
            }
        }

        perception
    }
}
