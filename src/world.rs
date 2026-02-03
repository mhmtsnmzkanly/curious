use crate::{
    entity::{Entity, intent::Intent, perception::*, phase::EntityPhase},
    gen_range,
    logger::{LogLevel, Logger},
    map::{
        movement::{Direction, Position, DIRECTION_ARRAY},
        Map,
    },
};
use std::collections::HashMap;

/// Canlının yönetim biçimi
pub struct EntitySlot {
    /// Canlının benzerhsiz kimlik numarası
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

    /// Simülasyon tur sayacı
    pub tick_counter: usize,

    /// Gelişmiş loglama
    pub logger: Logger,
}

impl World {
    pub fn new(x1: isize, x2: isize, y1: isize, y2: isize, entities: Vec<EntitySlot>) -> World {
        // Haritayı oluştur
        let mut map = Map::new(x1, x2, y1, y2);
        // Haritanın % kısmına rastgele kaynak yerleştir.
        // Kaynak yoğunluğunu biraz düşür (aşırı doygunluk davranışları baskılamasın)
        map.populate_resources(0.05f32);
        // Döndür
        // Her çalıştırmada ayrı log dosyası oluştur (okunabilir tarih/saat)
        let now = time::OffsetDateTime::now_utc();
        let format = time::format_description::parse("[year]-[month]-[day]_[hour]-[minute]-[second]")
            .unwrap_or_else(|_| time::format_description::parse("[year][month][day]_[hour][minute][second]").unwrap());
        let ts = now.format(&format).unwrap_or_else(|_| "unknown_time".to_string());
        let log_path = format!("logs/simulation_{}.log", ts);
        let mut logger = Logger::new(&log_path);
        logger.set_min_level(LogLevel::Info);
        World {
            map,
            entities,
            tick_counter: 0,
            logger,
        }
    }

    /// Tick, bir zaman birimidir
    /// Tick, canlının bulunduğu konumu baz alarak Perception oluşturur.
    /// Entity, verilen Perception ile karar alır.
    /// World, Perception -> Intent şeklinde yola koyulur.
    /// Son adımda sonuca Intent Resolver karar verir.
    /// BU KARAR KESİNLİK DEĞİLDİR, WORLD SON SÖZÜ SÖYLER
    /// ÇAKIŞAN NİYETLER İÇİN WORLD İNSİYATİF ALABİLİR
    pub fn tick(&mut self) {
        self.tick_counter += 1;

        // Removed aşamasındaki entityleri sil
        self.entities
            .retain(|slot| !matches!(slot.phase, EntityPhase::Removed));

        let mut log_lines: Vec<String> = Vec::new();
        log_lines.push(format!("=== Tick {} ===", self.tick_counter));

        // Çakışma çözümü ve hızlı erişim için dolu hücre haritası
        let mut occupied: HashMap<Position, usize> = self
            .entities
            .iter()
            .filter(|slot| !matches!(slot.phase, EntityPhase::Corpse { .. } | EntityPhase::Removed))
            .map(|slot| (slot.pos, slot.id))
            .collect();

        let mut intents: Vec<(usize, Intent)> = Vec::new();

        // Her entity için perception ve intent oluştur
        for slot in &self.entities {
            if !slot.phase.is_active() {
                continue; // Sadece aktif canlılar karar verir
            }
            let perception = self.build_perception(slot);
            let intent = slot.entity().make_intent(perception);
            intents.push((slot.id, intent));

            // Niyet logu (Idle ise her 5 tick'te bir yaz)
            let last_intent = intents.last().unwrap().1.clone();
            let should_log_intent = !matches!(last_intent, Intent::Idle { .. })
                || (self.tick_counter % 5 == 0);
            if should_log_intent {
                log_lines.push(format!(
                    "[Niyet] @{} {:?} Pos:{:?} => {:?}",
                    slot.id,
                    slot.base.species(),
                    slot.pos,
                    last_intent
                ));
            }
        }

        // Intentleri çöz
        //
        // 1. Move planları ve mate planlarını önceden topla
        let mut move_plans: Vec<(usize, Position, usize)> = Vec::new();
        let mut eat_plans: Vec<(usize, Position, usize)> = Vec::new();
        let mut drink_plans: Vec<(usize, Position, usize)> = Vec::new();
        let mut mate_plans: Vec<(usize, usize)> = Vec::new();
        let mut attack_plans: Vec<(usize, usize)> = Vec::new();
        let mut flee_plans: Vec<(usize, Position, usize)> = Vec::new();
        let mut sleep_plans: Vec<(usize, usize)> = Vec::new();

        for (id, intent) in intents {
            match intent {
                Intent::Move { steps } => {
                    if !steps.is_empty() {
                        if let Some(slot) = self.entities.iter_mut().find(|s| s.id == id) {
                            let mut new_pos: Position = slot.pos;
                            let mut cost: usize = 0;

                            for dir in steps.0.iter() {
                                if !self.map.is_walkable(new_pos + *dir)
                                    || !slot.base.life().can_move_for(cost + 1)
                                {
                                    break;
                                }
                                cost += 1;
                                new_pos = new_pos + *dir;
                            }
                            move_plans.push((id, new_pos, cost));
                            log_lines.push(format!(
                                "[Plan] Move  @{} {:?} -> {:?} adim:{}",
                                slot.id,
                                slot.base.species(),
                                new_pos,
                                cost
                            ));
                        }
                    }
                }
                Intent::Eat { at, corpse_id: _ } => {
                    if let Some(slot) = self.entities.iter().find(|s| s.id == id) {
                        let mut new_pos: Position = slot.pos;
                        let mut cost: usize = 0;
                        for dir in at.0.iter() {
                            if !self.map.is_walkable(new_pos + *dir)
                                || !slot.base.life().can_move_for(cost + 1)
                            {
                                break;
                            }
                            cost += 1;
                            new_pos = new_pos + *dir;
                        }
                        // Aynı hücredeyse de yeme planı üret
                        eat_plans.push((id, new_pos, cost));
                        log_lines.push(format!(
                            "[Plan] Eat   @{} {:?} -> {:?} adim:{}",
                            slot.id,
                            slot.base.species(),
                            new_pos,
                            cost
                        ));
                    }
                }
                Intent::Drink { at } => {
                    if let Some(slot) = self.entities.iter().find(|s| s.id == id) {
                        let mut new_pos: Position = slot.pos;
                        let mut cost: usize = 0;
                        for dir in at.0.iter() {
                            if !self.map.is_walkable(new_pos + *dir)
                                || !slot.base.life().can_move_for(cost + 1)
                            {
                                break;
                            }
                            cost += 1;
                            new_pos = new_pos + *dir;
                        }
                        // Aynı hücredeyse de içme planı üret
                        drink_plans.push((id, new_pos, cost));
                        log_lines.push(format!(
                            "[Plan] Drink @{} {:?} -> {:?} adim:{}",
                            slot.id,
                            slot.base.species(),
                            new_pos,
                            cost
                        ));
                    }
                }
                Intent::Mate { target_id } => {
                    mate_plans.push((id, target_id));
                    log_lines.push(format!(
                        "[Plan] Mate  @{} -> @{}",
                        id, target_id
                    ));
                }
                Intent::Attack { target_id } => {
                    attack_plans.push((id, target_id));
                    log_lines.push(format!(
                        "[Plan] Attack @{} -> @{}",
                        id, target_id
                    ));
                }
                Intent::Flee { target_id } => {
                    let target_pos = match self.entities.iter().find(|s| s.id == target_id) {
                        Some(t) => t.pos,
                        None => continue,
                    };
                    if let Some(slot) = self.entities.iter().find(|s| s.id == id) {
                        let mut new_pos: Position = slot.pos;
                        let mut cost: usize = 0;

                        for _ in 0..slot.base.life().speed {
                            // Hedefe en çok uzaklaştıran yönü seç
                            let mut best_dir: Option<Direction> = None;
                            let mut best_dist: usize = new_pos.distance_to(target_pos);

                            for dir in DIRECTION_ARRAY {
                                let candidate = new_pos + dir;
                                if !self.map.is_walkable(candidate) {
                                    continue;
                                }
                                let dist = candidate.distance_to(target_pos);
                                if dist > best_dist {
                                    best_dist = dist;
                                    best_dir = Some(dir);
                                }
                            }

                            // Daha iyi bir yön yoksa, yürünebilir herhangi bir yönü seç
                            let dir = match best_dir {
                                Some(d) => d,
                                None => {
                                    let mut fallback: Option<Direction> = None;
                                    for d in DIRECTION_ARRAY {
                                        let candidate = new_pos + d;
                                        if self.map.is_walkable(candidate) {
                                            fallback = Some(d);
                                            break;
                                        }
                                    }
                                    let Some(d) = fallback else { break };
                                    d
                                }
                            };
                            if !slot.base.life().can_move_for(cost + 1) {
                                break;
                            }
                            cost += 1;
                            new_pos = new_pos + dir;
                        }

                        flee_plans.push((id, new_pos, cost));
                        log_lines.push(format!(
                            "[Plan] Flee  @{} -> {:?} (hedef @{}) adim:{}",
                            id, new_pos, target_id, cost
                        ));
                    }
                }
                Intent::Idle { duration: _ } => {
                    if let Some(slot) = self.entities.iter().find(|s| s.id == id) {
                        // Hafif gezinme: %30 ihtimalle 1 adım rastgele dene
                        const IDLE_MOVE_CHANCE: isize = 30;
                        let roll = gen_range(1, 100);
                        if roll <= IDLE_MOVE_CHANCE && slot.base.life().can_move_for(1) {
                            let mut chosen: Option<Position> = None;
                            for _ in 0..8 {
                                let dir = DIRECTION_ARRAY[gen_range(0, 7isize) as usize];
                                let candidate = slot.pos + dir;
                                if self.map.is_walkable(candidate) {
                                    chosen = Some(candidate);
                                    break;
                                }
                            }
                            if let Some(pos) = chosen {
                                move_plans.push((id, pos, 1));
                                log_lines.push(format!(
                                    "[Plan] Idle->Move @{} {:?} -> {:?} adim:1",
                                    slot.id,
                                    slot.base.species(),
                                    pos
                                ));
                            }
                        }
                    }
                }
                Intent::Sleep { duration } => {
                    sleep_plans.push((id, duration));
                    log_lines.push(format!(
                        "[Plan] Sleep @{} sure:{}",
                        id, duration
                    ));
                }
                _ => {}
            }
        }

        // ------------------------------
        // 2. Move planlarını uygula (çakışma çözümü ile)
        // ------------------------------
        let mut move_candidates: HashMap<Position, Vec<(usize, Position, usize)>> = HashMap::new();
        for plan in &move_plans {
            move_candidates.entry(plan.1).or_default().push(*plan);
        }

        let mut move_winners: Vec<(usize, Position, usize)> = move_candidates
            .into_values()
            .map(|mut group| {
                group.sort_by_key(|(id, _, _)| *id);
                group[0]
            })
            .collect();

        move_winners.sort_by_key(|(id, _, _)| *id);

        for (id, new_pos, cost) in move_winners {
            // Başka biri orayı tutuyorsa hareketi engelle
            if let Some(other_id) = occupied.get(&new_pos) {
                if *other_id != id {
                    log_lines.push(format!(
                        "[Engel] Move  @{} -> {:?} (doluluk @{})",
                        id, new_pos, other_id
                    ));
                    continue;
                }
            }

            if let Some(slot) = self.entities.iter_mut().find(|s| s.id == id) {
                // Eski pozisyonu boşalt
                occupied.remove(&slot.pos);
                slot.base.life_mut().on_move(cost);
                slot.pos = new_pos;
                occupied.insert(new_pos, id);

                log_lines.push(format!(
                    "[Uygula] Move  @{} -> {:?} adim:{}",
                    id, new_pos, cost
                ));
            }
        }

        // ------------------------------
        // 3. Eat planlarını uygula (çakışma çözümü ile)
        // ------------------------------
        let mut eat_candidates: HashMap<Position, Vec<(usize, Position, usize)>> = HashMap::new();
        for plan in &eat_plans {
            eat_candidates.entry(plan.1).or_default().push(*plan);
        }

        let mut eat_winners: Vec<(usize, Position, usize)> = eat_candidates
            .into_values()
            .map(|mut group| {
                group.sort_by_key(|(id, _, _)| *id);
                group[0]
            })
            .collect();

        eat_winners.sort_by_key(|(id, _, _)| *id);

        for (id, new_pos, cost) in eat_winners {
            if let Some(other_id) = occupied.get(&new_pos) {
                if *other_id != id {
                    log_lines.push(format!(
                        "[Engel] Eat   @{} -> {:?} (doluluk @{})",
                        id, new_pos, other_id
                    ));
                    continue;
                }
            }

            if let Some(slot) = self.entities.iter_mut().find(|s| s.id == id) {
                occupied.remove(&slot.pos);
                slot.pos = new_pos;
                slot.base.life_mut().on_move(cost);
                occupied.insert(new_pos, id);

                if let Some(cell) = self.map.cell(new_pos) {
                    if let crate::map::cell::Cell::Food { amount } = cell {
                        //println!("[@{}] Entity eating from {:?}", slot.id, slot.pos);
                        let eat_amount = *amount.min(&5);
                        slot.entity_mut().life_mut().restore_energy(eat_amount);
                        self.map.reduce_cell_amount(new_pos, eat_amount);

                        log_lines.push(format!(
                            "[Uygula] Eat   @{} -> {:?} miktar:{}",
                            id, new_pos, eat_amount
                        ));
                    }
                }
            }
        }

        // ------------------------------
        // 3.1 Drink planlarını uygula (çakışma çözümü ile)
        // ------------------------------
        let mut drink_candidates: HashMap<Position, Vec<(usize, Position, usize)>> = HashMap::new();
        for plan in &drink_plans {
            drink_candidates.entry(plan.1).or_default().push(*plan);
        }

        let mut drink_winners: Vec<(usize, Position, usize)> = drink_candidates
            .into_values()
            .map(|mut group| {
                group.sort_by_key(|(id, _, _)| *id);
                group[0]
            })
            .collect();

        drink_winners.sort_by_key(|(id, _, _)| *id);

        for (id, new_pos, cost) in drink_winners {
            if let Some(other_id) = occupied.get(&new_pos) {
                if *other_id != id {
                    log_lines.push(format!(
                        "[Engel] Drink @{} -> {:?} (doluluk @{})",
                        id, new_pos, other_id
                    ));
                    continue;
                }
            }

            if let Some(slot) = self.entities.iter_mut().find(|s| s.id == id) {
                occupied.remove(&slot.pos);
                slot.pos = new_pos;
                slot.base.life_mut().on_move(cost);
                occupied.insert(new_pos, id);

                if let Some(cell) = self.map.cell(new_pos) {
                    if let crate::map::cell::Cell::Water { amount } = cell {
                        let drink_amount = *amount.min(&5);
                        slot.entity_mut().life_mut().restore_water(drink_amount);
                        self.map.reduce_cell_amount(new_pos, drink_amount);

                        log_lines.push(format!(
                            "[Uygula] Drink @{} -> {:?} miktar:{}",
                            id, new_pos, drink_amount
                        ));
                    }
                }
            }
        }

        // ------------------------------
        // 4. Mate planlarını uygula
        // ------------------------------

        let mut new_entities: Vec<crate::world::EntitySlot> = Vec::new();
        let id_to_index: HashMap<usize, usize> = self
            .entities
            .iter()
            .enumerate()
            .map(|(i, s)| (s.id, i))
            .collect();

        for (self_id, target_id) in mate_plans {
            let self_index = id_to_index.get(&self_id).copied();
            let target_index = id_to_index.get(&target_id).copied();

            let (self_index, target_index) = match (self_index, target_index) {
                (Some(a), Some(t)) if a != t => (a, t),
                _ => continue,
            };

            // Aynı anda iki mutable borrow için split_at_mut kullanılır
            let (left, right) = if self_index < target_index {
                let (l, r) = self.entities.split_at_mut(target_index);
                (l, r)
            } else {
                let (l, r) = self.entities.split_at_mut(self_index);
                (r, l)
            };

            let (self_slot, target_slot) = if self_index < target_index {
                (&mut left[self_index], &mut right[0])
            } else {
                (&mut left[0], &mut right[target_index])
            };

            // İkisi de aktif olmalı
            if !self_slot.phase.is_active() || !target_slot.phase.is_active() {
                continue;
            }

            // Yakınlık kontrolü (çapraz dahil komşu)
            let dx = (self_slot.pos.x - target_slot.pos.x).abs();
            let dy = (self_slot.pos.y - target_slot.pos.y).abs();
            if dx > 1 || dy > 1 {
                log_lines.push(format!(
                    "[Engel] Mate  @{} + @{} (mesafe x:{} y:{})",
                    self_id, target_id, dx, dy
                ));
                continue;
            }

            // İki tarafın da üreme koşulları uygun olmalı
            if !self_slot.entity().life().can_reproduce()
                || !target_slot.entity().life().can_reproduce()
            {
                continue;
            }

            // Çocuğun doğacağı boş bir komşu hücre bul
            let mut child_pos: Option<Position> = None;
            for dir in DIRECTION_ARRAY {
                let candidate = target_slot.pos + dir;
                if self.map.is_walkable(candidate) && !occupied.contains_key(&candidate) {
                    child_pos = Some(candidate);
                    break;
                }
            }
            let Some(child_pos) = child_pos else {
                log_lines.push(format!(
                    "[Engel] Mate  @{} + @{} (bos komsu yok)",
                    self_id, target_id
                ));
                continue;
            };

            // Üreme maliyetleri
            self_slot.entity_mut().life_mut().on_reproduce();
            target_slot.entity_mut().life_mut().on_reproduce();

            let child = target_slot.entity_mut().reproduce();
            let new_id = self.entities.iter().map(|s| s.id).max().unwrap_or(0) + 1;

            new_entities.push(crate::world::EntitySlot::new(
                new_id,
                child_pos,
                crate::entity::phase::EntityPhase::Active,
                child,
            ));

            // Yeni doğan pozisyonu işgal edildi
            occupied.insert(child_pos, new_id);

            log_lines.push(format!(
                "[Uygula] Mate  @{} + @{} => @{} {:?}",
                self_id, target_id, new_id, child_pos
            ));
        }
        self.entities.extend(new_entities);

        // ------------------------------
        // 5. Attack planlarını uygula
        // ------------------------------
        let id_to_index: HashMap<usize, usize> = self
            .entities
            .iter()
            .enumerate()
            .map(|(i, s)| (s.id, i))
            .collect();

        for (attacker_id, target_id) in attack_plans {
            let attacker_index = id_to_index.get(&attacker_id).copied();
            let target_index = id_to_index.get(&target_id).copied();

            let (attacker_index, target_index) = match (attacker_index, target_index) {
                (Some(a), Some(t)) if a != t => (a, t),
                _ => continue,
            };

            // Aynı anda iki mutable borrow için split_at_mut kullanılır
            let (left, right) = if attacker_index < target_index {
                let (l, r) = self.entities.split_at_mut(target_index);
                (l, r)
            } else {
                let (l, r) = self.entities.split_at_mut(attacker_index);
                (r, l)
            };

            let (attacker, target) = if attacker_index < target_index {
                (&mut left[attacker_index], &mut right[0])
            } else {
                (&mut left[0], &mut right[target_index])
            };

            // Sadece aktif hedefe saldır
            if !target.phase.is_active() {
                log_lines.push(format!(
                    "[Engel] Attack @{} -> @{} (hedef aktif degil)",
                    attacker_id, target_id
                ));
                continue;
            }

            // Yakınlık kontrolü (çapraz dahil komşu)
            let dx = (attacker.pos.x - target.pos.x).abs();
            let dy = (attacker.pos.y - target.pos.y).abs();
            if dx <= 1 && dy <= 1 {
                // Basit hasar modeli
                attacker.entity_mut().life_mut().consume_energy(3);
                target.entity_mut().life_mut().take_damage(6);

                log_lines.push(format!(
                    "[Uygula] Attack @{} -> @{} hasar:{}",
                    attacker_id, target_id, 6
                ));
            } else {
                log_lines.push(format!(
                    "[Engel] Attack @{} -> @{} (mesafe x:{} y:{})",
                    attacker_id, target_id, dx, dy
                ));
            }
        }

        // ------------------------------
        // 6. Flee planlarını uygula (çakışma çözümü ile)
        // ------------------------------
        let mut flee_candidates: HashMap<Position, Vec<(usize, Position, usize)>> = HashMap::new();
        for plan in &flee_plans {
            flee_candidates.entry(plan.1).or_default().push(*plan);
        }

        let mut flee_winners: Vec<(usize, Position, usize)> = flee_candidates
            .into_values()
            .map(|mut group| {
                group.sort_by_key(|(id, _, _)| *id);
                group[0]
            })
            .collect();

        flee_winners.sort_by_key(|(id, _, _)| *id);

        for (id, new_pos, cost) in flee_winners {
            if let Some(other_id) = occupied.get(&new_pos) {
                if *other_id != id {
                    log_lines.push(format!(
                        "[Engel] Flee  @{} -> {:?} (doluluk @{})",
                        id, new_pos, other_id
                    ));
                    continue;
                }
            }

            if let Some(slot) = self.entities.iter_mut().find(|s| s.id == id) {
                occupied.remove(&slot.pos);
                if new_pos != slot.pos {
                    slot.base.life_mut().on_move(cost);
                    slot.pos = new_pos;
                } else {
                    log_lines.push(format!(
                        "[Engel] Flee  @{} -> {:?} (yerinde kaldı)",
                        id, slot.pos
                    ));
                }
                occupied.insert(slot.pos, id);

                log_lines.push(format!(
                    "[Uygula] Flee  @{} -> {:?} adim:{}",
                    id, slot.pos, cost
                ));
            }
        }

        // ------------------------------
        // 7. Sleep planlarını uygula
        // ------------------------------
        for (id, duration) in sleep_plans {
            if let Some(slot) = self.entities.iter_mut().find(|s| s.id == id) {
                if slot.phase.is_active() {
                    slot.phase = EntityPhase::Sleeping { remaining: duration };
                    log_lines.push(format!(
                        "[Uygula] Sleep @{} sure:{}",
                        id, duration
                    ));
                }
            }
        }

        for slot in &mut self.entities {
            // Sadece canlı olanların tick güncellemelerini uygula (yaş, enerji, speed reset vb.)
            if slot.phase.is_active() {
                slot.entity_mut().tick();
            }
            // Fazları güncelle ve ölüleri işaretle
            slot.phase.tick();

            if slot.phase == EntityPhase::Active && !slot.entity().life().is_alive() {
                slot.phase = EntityPhase::Corpse { remaining: 5 }; // Ceset 5 tick kalacak
                // Cesedi yiyeceğe dönüştür
                let life = slot.entity().life();
                let amount = (life.max_health / 4).max(5);
                self.map.add_food(slot.pos, amount);

                log_lines.push(format!(
                    "[Durum] Ceset @{} -> Food miktar:{}",
                    slot.id, amount
                ));
            }
        }

        // Tick sonunda logları yaz
        self.logger.log_many(LogLevel::Info, &log_lines);
    }

    /// Intentleri çöz ve uygulama fonksiyonu
    //pub fn resolve_intent(&mut self, intents: Vec<(usize, Intent)>) {}

    /// Entity "Intent" üretebilmesi için "Perception" üretir
    pub fn build_perception(&self, current_slot: &EntitySlot) -> Perception {
        let mut perception = Perception::empty();
        let radius = current_slot.base.life().vision_range; // Görüş mesafesi (yarıçap)

        // 1. Yakındaki Yiyecekleri Algıla
        let found_foods = self.map.scan_foods_within(current_slot.pos, radius);
        for (_f_pos, steps, amount) in found_foods {
            perception.add_food(amount, false, steps);
        }

        // 1.1 Yakındaki Suları Algıla
        let found_waters = self.map.scan_waters_within(current_slot.pos, radius);
        for (_w_pos, steps, amount) in found_waters {
            perception.add_water(amount, steps);
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
                    let other_life = other.entity().life();
                    let power = other_life.health + other_life.energy;
                    perception.add_entity(other.id, other.entity().species(), power, steps);
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
