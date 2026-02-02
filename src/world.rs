use crate::{
    entity::{Entity, intent::Intent, perception::*, phase::EntityPhase},
    map::{Map, movement::Position},
};
//use std::collections::{HashMap, HashSet};

/// Canlının yönetim biçimi
//#[derive(PartialEq, Eq)]
pub struct EntitySlot {
    /// Canlının benzersiz kimlik numarası
    pub id: usize,
    /// Canlının konumu
    pub pos: Position,
    /// Canlının bulunduğu durum (aktif, uykuda, ölü, silinecek)
    pub phase: EntityPhase,
    /// Canlının verisi
    pub base: Box<dyn Entity>,
}

impl EntitySlot {
    /// Yeni canlı oluştur
    pub fn new(id: usize, pos: Position, phase: EntityPhase, base: Box<dyn Entity>) -> EntitySlot {
        Self {
            id,
            pos,
            phase,
            base,
        }
    }

    /// Canlının bulunduğu konum
    pub fn position(&self) -> &Position {
        &self.pos
    }

    /// Canlının bulunduğu konumu (değiştirilebilir)
    pub fn position_mut(&mut self) -> &mut Position {
        &mut self.pos
    }

    /// Canlıyı döndürür, okumak için
    pub fn entity(&self) -> &dyn Entity {
        self.base.as_ref()
    }
    /// Canlıyı döndürür, yazmak için
    pub fn entity_mut(&mut self) -> &mut dyn Entity {
        self.base.as_mut()
    }

    /// Canlı durumunu döndürür
    pub fn phase(&self) -> &EntityPhase {
        &self.phase
    }

    /// Canlının durumunu değiştirilebilir
    pub fn phase_mut(&mut self) -> &mut EntityPhase {
        &mut self.phase
    }
}

/// Entity'ler burada tutulur,
/// Intent verebilicek durumda ki Entity'lere
/// Perception verip, Intent alarak
/// Kendi içerisinde ki kurallar dahilinde
/// Son kararı verir.
pub struct World {
    /// Simülasyon haritası
    pub map: Map,

    /// Tüm Canlıların ID, Pos ve Entity listesi
    pub entities: Vec<EntitySlot>,
}

impl World {
    pub fn new(x1: isize, x2: isize, y1: isize, y2: isize, entities: Vec<EntitySlot>) -> World {
        // Haritayı oluştur
        let mut map = Map::new(x1, x2, y1, y2);
        // Haritanın % kısmına rastgele kaynak yerleştir.
        map.populate_resources(0.2f32);
        // Döndür
        World { map, entities }
    }

    /// Tick, bir zaman birimidir
    /// Tick, canlının bulunduğu konumu baz alarak Perception oluşturur.
    /// Entity, verilen Perception ile karar alır.
    /// World, Perception -> Intent şeklinde yola koyulur.
    /// Son adımda sonuca Intent Resolver karar verir.
    /// BU KARAR KESİNLİK DEĞİLDİR, WORLD SON SÖZÜ SÖYLER
    /// ÇAKIŞAN NİYETLER İÇİN WORLD İNSİYATİF ALABİLİR
    pub fn tick(&mut self) {
        // Removed aşamasındaki entityleri sil
        self.entities.retain(|slot| {
            let remove = matches!(slot.phase, EntityPhase::Removed);
            if remove {
                println!("[@{}] Removed!", slot.id);
            }
            !remove
        });

        let mut intents: Vec<(usize, Intent)> = Vec::new();

        // Her entity için perception ve intent oluştur
        for slot in &self.entities {
            if !slot.phase.is_active() {
                println!("[@{}] Not Active!", slot.id);
                continue; // Sadece aktif canlılar karar verir
            }
            let perception = self.build_perception(slot);
            println!(
                "[@{id}] {pos:?}\n[@{id}] {life:?}\n[@{id}] {ption:?}",
                id = slot.id,
                pos = &slot.pos,
                life = &slot.base.as_ref().life(),
                ption = perception
            );
            let intent = slot.entity().make_intent(perception);
            println!("[@{}] Intent {:?}", slot.id, intent);
            intents.push((slot.id, intent));
        }

        // Intentleri çöz
        self.resolve_intent(intents);

        for slot in &mut self.entities {
            // Sadece canlı olanların tick güncellemelerini uygula (yaş, enerji, speed reset vb.)
            if slot.phase.is_active() {
                slot.entity_mut().tick();
            }
            // Fazları güncelle ve ölüleri işaretle
            slot.phase.tick();

            if slot.phase == EntityPhase::Active && !slot.entity().life().is_alive() {
                slot.phase = EntityPhase::Corpse { remaining: 5 }; // Ceset 50 tick kalacak
            }
        }
    }

    /// Intentleri çöz ve uygulama fonksiyonu
    pub fn resolve_intent(&mut self, intents: Vec<(usize, Intent)>) {
        // ------------------------------
        // 1. Move planları ve mate planlarını önceden topla
        // ------------------------------
        struct MovePlan {
            id: usize,
            new_pos: Position,
        }

        struct MatePlan {
            parent_id: usize,
            target_id: usize,
        }

        let mut move_plans: Vec<MovePlan> = Vec::new();
        let mut eat_plans: Vec<(usize, Position)> = Vec::new();
        let mut mate_plans: Vec<MatePlan> = Vec::new();

        for (id, intent) in intents {
            match intent {
                Intent::Move { steps } => {
                    if steps.is_empty() {
                        continue;
                    }

                    if let Some(slot) = self.entities.iter_mut().find(|s| s.id == id) {
                        let mut new_pos: Position = slot.pos;
                        for dir in steps.0.iter() {
                            if !self.map.is_walkable(new_pos + *dir) {
                                break;
                            }
                            slot.base.as_mut().life_mut().consume_energy(1);
                            new_pos = new_pos + *dir;
                        }
                        move_plans.push(MovePlan { id, new_pos });
                    }
                }
                Intent::Eat { at, corpse_id: _ } => {
                    if at.is_empty() {
                        continue;
                    }
                    if let Some(slot) = self.entities.iter().find(|s| s.id == id) {
                        let mut new_pos: Position = slot.pos;
                        for dir in at.0.iter() {
                            if !self.map.is_walkable(new_pos + *dir) {
                                break;
                            }
                            new_pos = new_pos + *dir;
                        }
                        eat_plans.push((id, new_pos));
                    }
                }
                Intent::Mate { target_id } => {
                    mate_plans.push(MatePlan {
                        parent_id: id,
                        target_id,
                    });
                }
                _ => {}
            }
        }

        // ------------------------------
        // 2. Move planlarını uygula (tek mutable borrow)
        // ------------------------------
        for plan in move_plans {
            if let Some(slot) = self.entities.iter_mut().find(|s| s.id == plan.id) {
                println!(
                    "[@{}] Entity moving from {:?} to {:?}",
                    slot.id, slot.pos, plan.new_pos
                );
                slot.pos = plan.new_pos;
                slot.entity_mut().life_mut().on_move();
            }
        }

        // ------------------------------
        // 3. Eat planlarını uygula
        // ------------------------------
        for (id, pos) in eat_plans {
            if let Some(slot) = self.entities.iter_mut().find(|s| s.id == id) {
                if let Some(cell) = self.map.cell(pos) {
                    if let crate::map::cell::Cell::Food { amount } = cell {
                        println!("[@{}] Entity eating from {:?}", slot.id, slot.pos);
                        let eat_amount = *amount.min(&5);
                        slot.entity_mut().life_mut().restore_energy(eat_amount);
                        self.map.reduce_cell_amount(pos, eat_amount);
                    }
                }
            }
        }

        // ------------------------------
        // 4. Mate planlarını uygula
        // ------------------------------
        let mut new_entities: Vec<crate::world::EntitySlot> = Vec::new();
        for plan in mate_plans {
            let can_mate = self.entities.iter().any(|s| s.id == plan.parent_id)
                && self.entities.iter().any(|s| s.id == plan.target_id);
            if !can_mate {
                continue;
            }

            // Mutable borrow tek seferde al
            let mut maybe_child = None;
            for slot in &mut self.entities {
                if slot.id == plan.parent_id {
                    maybe_child = Some(slot.entity_mut().reproduce());
                    slot.entity_mut().life_mut().on_reproduce();
                    break;
                }
            }

            if let Some(child) = maybe_child {
                let new_id = self.entities.iter().map(|s| s.id).max().unwrap_or(0) + 1;
                let parent_pos = self
                    .entities
                    .iter()
                    .find(|s| s.id == plan.parent_id)
                    .unwrap()
                    .pos;
                new_entities.push(crate::world::EntitySlot::new(
                    new_id,
                    parent_pos,
                    crate::entity::phase::EntityPhase::Active,
                    child,
                ));
            }
        }

        self.entities.extend(new_entities);
    }

    /// Entity "Intent" üretebilmesi için "Perception" üretir
    pub fn build_perception(&self, current_slot: &EntitySlot) -> Perception {
        let mut perception = Perception::empty();
        let radius = current_slot.base.life().vision_range; // Görüş mesafesi (yarıçap)

        // 1. Yakındaki Yiyecekleri Algıla
        let found_foods = self.map.scan_foods_within(current_slot.pos, radius);
        for (_f_pos, steps, amount) in found_foods {
            perception.add_food(amount, false, steps);
        }

        // 2. Yakındaki Diğer Canlıları Algıla
        for other in &self.entities {
            // Kendisini algılamasın
            if other.id == current_slot.id {
                continue;
            }

            // Mesafe kontrolü (Manhattan mesafesi kullanılıyor)
            let dist = current_slot.pos.distance_to(other.pos);

            if dist <= radius {
                // Canlıya giden yolu (Steps) BFS ile hesapla
                if let Some(steps) = self.map.bfs_steps_to(current_slot.pos, other.pos, radius) {
                    // Algılanan canlıyı ekle (ID, Tür ve Adımlar)
                    perception.add_entity(other.id, other.entity().species(), steps);
                }
            }
        }

        // 3. Yürünebilir Yönleri ve Mesafeleri Algıla
        let walkable_map = self.map.walkable_distances(current_slot.pos);
        for (dir, dist) in walkable_map {
            perception.add_direction(dir, dist);
        }

        perception
    }

    // Bu pozisyonda entity var mı?
    //pub fn has_entity(&self, pos: Position) -> bool { self.entity_pos.contains_key(&pos) }
    // Bu pozisyondaki entity id'leri
    //pub fn entities_at(&self, pos: Position) -> &[usize] { self.entity_pos.get(&pos).map(|v| v.as_slice()).unwrap_or(&[]) }
    // Bu pozisyonda canlı entity var mı?
    //pub fn has_alive_entity(&self, pos: Position) -> bool { self.entities_at(pos).iter().any(|id| self.entity_phase.get(id).is_some_and(|p| p.is_active()))}
    // Bu pozisyonda ceset var mı?
    //pub fn has_corpse(&self, pos: Position) -> bool {self.entities_at(pos).iter().any(|id| self.entity_phase.get(id).is_some_and(|p| p.is_corpse()))}
    // Belirli bir merkez etrafında (Manhattan mesafe)
    // entity olan pozisyonları döner
    //pub fn nearby_entities(&self, center: Position, radius: usize) -> Vec<(Position, usize)> {      let mut result = Vec::new();        for (pos, ids) in self.entity_pos.iter() {            let dx = pos.x.abs_diff(center.x);            let dy = pos.y.abs_diff(center.y);            if dx + dy <= radius {                for id in ids {       result.push((*pos, *id));                }            }        }   result  }
}
