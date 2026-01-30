use std::collections::{HashMap, HashSet};

use crate::{
    entity::{Entity, action::Action, phase::EntityPhase},
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
        // ---------------------------
        // FAZ 1: İÇSEL GÜNCELLEME
        // Yaşlanma, enerji kaybı, ölüm
        // ---------------------------
        for e in self.entities.iter_mut() {
            e.tick();
        }

        // Ölenleri temizle ve haritaları taze tut
        self.entities.retain(|e| e.life().is_alive());
        self.rebuild_entity_maps();

        // ---------------------------
        // FAZ 2: ALGILAMA VE KARAR
        // Entity’ler çevreyi algılar ve eylem niyetlerini oluşturur
        // ---------------------------
        let view = WorldView::new(&self.map, &self.entity_pos, &self.entity_phase);
        let actions: Vec<(usize, Action)> = self
            .entities
            .iter()
            .map(|e| (e.id(), e.think(&view)))
            .collect();

        // ---------------------------
        // FAZ 3: HAREKET
        // ---------------------------
        let mut move_intents = std::collections::HashMap::new();
        for (id, action) in &actions {
            if let Action::Move(dir) = action {
                if let Some(e) = self.entities.iter().find(|ent| ent.id() == *id) {
                    let from = e.position();
                    let to = from + *dir;

                    if self.map.in_bounds(to) && self.map.is_walkable(to) {
                        move_intents.insert(*id, to);
                    }
                }
            }
        }

        for e in self.entities.iter_mut() {
            if let Some(&to) = move_intents.get(&e.id()) {
                e.position_mut().set(to);
                e.life_mut().on_move();
            }
        }

        // ---------------------------
        // FAZ 4: ETKİLEŞİMLER (Yeme, Üreme, Saldırı)
        // ---------------------------
        let mut already_interacted = std::collections::HashSet::new();
        let mut newborns: Vec<Box<dyn Entity>> = Vec::new();

        for (id, action) in &actions {
            if already_interacted.contains(id) {
                continue;
            }

            match action {
                Action::Eat => {
                    if let Some(e) = self.entities.iter_mut().find(|ent| ent.id() == *id) {
                        let pos = e.position();
                        if let Some(crate::map::cell::Cell::Food { amount: _ }) = self.map.cell(pos)
                        {
                            self.map.reduce_cell_amount(pos, 10);
                            e.life_mut().restore_energy(10);
                            already_interacted.insert(*id);
                        }
                    }
                }
                Action::Mate { target_id } => {
                    if already_interacted.contains(target_id) {
                        continue;
                    }

                    let e_idx_opt = self.entities.iter().position(|ent| ent.id() == *id);
                    let t_idx_opt = self.entities.iter().position(|ent| ent.id() == *target_id);

                    if let (Some(e_idx), Some(t_idx)) = (e_idx_opt, t_idx_opt) {
                        if e_idx == t_idx {
                            continue; // aynı canlıyı eşlemeye çalışmasın
                        }

                        // split_at_mut ile iki ayrı mutable referans alıyoruz
                        let (first_slice, second_slice) = if e_idx < t_idx {
                            let (left, right) = self.entities.split_at_mut(t_idx);
                            (left, right)
                        } else {
                            let (left, right) = self.entities.split_at_mut(e_idx);
                            (right, left)
                        };

                        let parent1: &mut Box<dyn Entity>;
                        let parent2: &mut Box<dyn Entity>;

                        if e_idx < t_idx {
                            parent1 = &mut first_slice[e_idx];
                            parent2 = &mut second_slice[0]; // t_idx, right slice’in 0. indexi
                        } else {
                            parent1 = &mut second_slice[0]; // e_idx, right slice’in 0. indexi
                            parent2 = &mut first_slice[t_idx];
                        }

                        if parent1.position().distance_to(parent2.position()) <= 1 {
                            if parent1.life().can_reproduce() && parent2.life().can_reproduce() {
                                let new_id = crate::generate_random_id();
                                newborns.push(parent1.reproduce(new_id, parent1.position()));

                                parent1.life_mut().on_reproduce();
                                parent2.life_mut().on_reproduce();

                                already_interacted.insert(*id);
                                already_interacted.insert(*target_id);
                            }
                        }
                    }
                }

                Action::Attack { target_id: _ } => {
                    // Saldırı mantığı buraya eklenebilir
                    already_interacted.insert(*id);
                }
                _ => {}
            }
        }

        // ---------------------------
        // FAZ 5: YENİ DOĞANLAR
        // ---------------------------
        self.entities.extend(newborns);

        // Son olarak haritaları tekrar güncelle
        self.rebuild_entity_maps();
    }
}
