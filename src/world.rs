use std::collections::HashMap;

use crate::{
    entity::{Entity, phase::EntityPhase},
    map::{Map, position::Position},
};

/// ===============================
/// WORLD VIEW
/// ===============================
///
/// WorldView:
/// - Entity'lerin dünyayı OKUMASI için vardır
/// - Dünya DEĞİŞTİRMEZ
/// - Entity iç durumlarını AÇMAZ
///
/// Ama şunları söyler:
/// - Bu pozisyonda entity var mı?
/// - Kaç tane var?
/// - Canlı mı / ceset mi?
/// - Yakın çevrede kimler var?
pub struct WorldView<'a> {
    pub map: &'a Map,

    /// Pozisyon -> entity id listesi
    /// Aynı hücrede birden fazla entity olabilir
    pub entity_pos: &'a HashMap<Position, Vec<usize>>,

    /// Entity id -> faz bilgisi (Active / Corpse vs.)
    pub entity_phase: &'a HashMap<usize, EntityPhase>,
}

impl<'a> WorldView<'a> {
    /// World tarafından oluşturulur
    pub fn new(
        map: &'a Map,
        entity_pos: &'a HashMap<Position, Vec<usize>>,
        entity_phase: &'a HashMap<usize, EntityPhase>,
    ) -> Self {
        Self {
            map,
            entity_pos,
            entity_phase,
        }
    }

    // ===============================
    // MAP OKUMA
    // ===============================

    /// Harita sınırları içinde mi?
    pub fn in_bounds(&self, pos: Position) -> bool {
        self.map.in_bounds(pos)
    }

    /// Bu hücreye hareket edilebilir mi?
    pub fn is_walkable(&self, pos: Position) -> bool {
        self.map.is_walkable(pos)
    }

    /// Hücre bilgisi
    pub fn cell(&self, pos: Position) -> Option<&crate::map::cell::Cell> {
        self.map.cell(pos)
    }

    // ===============================
    // ENTITY ALGILAMA
    // ===============================

    /// Bu pozisyonda entity var mı?
    pub fn has_entity(&self, pos: Position) -> bool {
        self.entity_pos.contains_key(&pos)
    }

    /// Bu pozisyondaki entity id'leri
    pub fn entities_at(&self, pos: Position) -> &[usize] {
        self.entity_pos
            .get(&pos)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Bu pozisyonda canlı entity var mı?
    pub fn has_alive_entity(&self, pos: Position) -> bool {
        self.entities_at(pos)
            .iter()
            .any(|id| self.entity_phase.get(id).is_some_and(|p| p.is_active()))
    }

    /// Bu pozisyonda ceset var mı?
    pub fn has_corpse(&self, pos: Position) -> bool {
        self.entities_at(pos)
            .iter()
            .any(|id| self.entity_phase.get(id).is_some_and(|p| p.is_corpse()))
    }

    // ===============================
    // YAKIN ÇEVRE (ALGILAMA TEMELİ)
    // ===============================

    /// Belirli bir merkez etrafında (Manhattan mesafe)
    /// entity olan pozisyonları döner
    pub fn nearby_entities(&self, center: Position, radius: usize) -> Vec<(Position, usize)> {
        let mut result = Vec::new();

        for (pos, ids) in self.entity_pos.iter() {
            let dx = pos.x.abs_diff(center.x);
            let dy = pos.y.abs_diff(center.y);

            if dx + dy <= radius {
                for id in ids {
                    result.push((*pos, *id));
                }
            }
        }

        result
    }
}

/// ===============================
/// WORLD
/// ===============================
///
/// World:
/// - Gerçek değişiklikler burada yapılır
/// - Entity konumları burada tutulur
/// - İki fazlı tick burada yönetilir
pub struct World {
    pub map: Map,

    /// Tüm entity'ler
    pub entities: Vec<Box<dyn Entity>>,

    /// Pozisyon -> entity id listesi
    entity_pos: HashMap<Position, Vec<usize>>,

    /// Entity id -> faz
    entity_phase: HashMap<usize, EntityPhase>,
}

impl World {
    /// World oluşturulurken çağrılır
    pub fn new(map: Map, entities: Vec<Box<dyn Entity>>) -> Self {
        let mut world = Self {
            map,
            entities,
            entity_pos: HashMap::new(),
            entity_phase: HashMap::new(),
        };

        world.rebuild_entity_maps();
        world
    }

    /// ===============================
    /// ENTITY HARİTALARINI YENİDEN KUR
    /// ===============================
    ///
    /// Bu fonksiyon:
    /// - Başlangıçta
    /// - Büyük temizliklerden sonra
    /// çağrılır
    fn rebuild_entity_maps(&mut self) {
        self.entity_pos.clear();
        self.entity_phase.clear();

        for e in self.entities.iter() {
            let id = e.id();
            let pos = e.position();

            self.entity_pos.entry(pos).or_default().push(id);
            self.entity_phase.insert(id, e.phase());
        }
    }

    pub fn tick(&mut self) {
        // ===============================
        // FAZ 1: ENTITY INTERNAL TICK
        // ===============================
        for e in self.entities.iter_mut() {
            e.tick();
        }

        // Faz bilgilerini güncelle
        self.entity_phase.clear();
        for e in self.entities.iter() {
            self.entity_phase.insert(e.id(), e.phase());
        }

        // WorldView oluştur
        let view = WorldView::new(&self.map, &self.entity_pos, &self.entity_phase);

        // ===============================
        // FAZ 2: ACTION TOPLAMA (NİYET)
        // ===============================
        let actions: Vec<(usize, crate::entity::action::Action)> = self
            .entities
            .iter()
            .map(|e| (e.id(), e.think(&view)))
            .collect();

        // ===============================
        // FAZ 3: HAREKETLERİ GRUPLA
        // ===============================
        use crate::entity::action::Action;
        use crate::map::direction::Direction;

        // entity_id -> hedef pozisyon
        let mut move_intents: HashMap<usize, Position> = HashMap::new();

        for (id, action) in &actions {
            if let Action::Move(dir) = action {
                if let Some(e) = self.entities.iter().find(|e| e.id() == *id) {
                    let from = e.position();
                    let to = from + *dir;

                    // Harita sınırı ve yürünebilirlik kontrolü
                    if self.map.in_bounds(to) && self.map.is_walkable(to) {
                        move_intents.insert(*id, to);
                    }
                }
            }
        }

        // ===============================
        // FAZ 4: HAREKETLERİ UYGULA
        // ===============================
        //
        // Aynı hücreye birden fazla entity girebilir
        // Çatışma sonucu (savaş vs.) daha sonra
        for e in self.entities.iter_mut() {
            if let Some(to) = move_intents.get(&e.id()) {
                e.position_mut().set(*to);
                e.life_mut().on_move();
            }
        }

        // ===============================
        // FAZ 5: YEME / SALDIRI / DİĞERLERİ
        // ===============================
        for (id, action) in actions {
            if let Some(e) = self.entities.iter_mut().find(|e| e.id() == id) {
                match action {
                    Action::Eat => {
                        let pos = e.position();

                        // Haritadaki kaynaktan ye
                        if let Some(cell) = self.map.cell(pos) {
                            match cell {
                                crate::map::cell::Cell::Food { .. } => {
                                    self.map.reduce_cell_amount(pos, 10);
                                    e.life_mut().restore_energy(10);
                                }
                                crate::map::cell::Cell::Water { .. } => {
                                    self.map.reduce_cell_amount(pos, 5);
                                }
                                _ => {}
                            }
                        }
                    }

                    Action::Attack { target_id } => {
                        // Basit saldırı modeli
                        if let Some(target) = self.entities.iter_mut().find(|t| t.id() == target_id)
                        {
                            target.life_mut().health = target.life_mut().health.saturating_sub(10);
                        }
                    }

                    _ => {}
                }
            }
        }

        // ===============================
        // FAZ 6: ENTITY HARİTALARINI YENİLE
        // ===============================
        self.rebuild_entity_maps();
    }
}
