## src/main.rs
```
use curious::{
    creatures::herbivore::HerbivoreEntity,
    entity::phase::EntityPhase,
    set_global_seed_with_time,
    world::{EntitySlot, World},
};
use std::{thread, time::Duration};

fn main() {
    // RNG için seed'i zaman damgası olarak günceller
    set_global_seed_with_time();
    let entities: Vec<EntitySlot> = vec![
        EntitySlot::new(
            1,
            (0isize, 0isize).into(),
            EntityPhase::Active,
            Box::new(HerbivoreEntity::default()),
        ),
        EntitySlot::new(
            2,
            (3isize, 3isize).into(),
            EntityPhase::Active,
            Box::new(HerbivoreEntity::default()),
        ),
    ];
    // İnteraktif dünya
    let mut world = World::new(-15, 15, -15, 15, entities);
    // İnteraktif dünya sayacı
    let mut tick_counter: usize = 0;
    loop {
        tick_counter += 1;
        world.tick();
        print_map(&world, tick_counter);
        thread::sleep(Duration::from_millis(500));
    }
}

pub fn print_map(world: &World, tick: usize) {
    print!("\x1B[2J\x1B[1;1H");

    let map_width = (world.map.max_x - world.map.min_x) as usize;
    let map_height = (world.map.max_y - world.map.min_y) as usize;

    println!("=== CURIOUS SIMULATION | Tick: {} ===", tick);
    println!("{:-<1$}", "", map_width + 5);

    for y in world.map.min_y..=world.map.max_y {
        // --- SOL KOLON: HARİTA ---
        for x in world.map.min_x..=world.map.max_x {
            let pos = (x, y).into();

            // Hücredeki varlığı kontrol et (Öncelik: Canlı > Ceset > Yemek)
            if let Some(slot) = world.entities.iter().find(|e| e.pos == pos) {
                match slot.phase {
                    EntityPhase::Active => print!("@ "),        // Canlı
                    EntityPhase::Corpse { .. } => print!("X "), // Ceset
                    _ => print!("? "),
                }
            } else if let Some(curious::map::cell::Cell::Food { .. }) = world.map.cell(pos) {
                print!("f "); // Yemek (Food)
            } else {
                print!(". "); // Boş hücre
            }
        }

        // --- SAĞ KOLON: CANLI DURUMLARI ---
        // Sadece haritanın ilk birkaç satırında canlı bilgilerini yazdır
        let entity_index = (y - world.map.min_y) as usize;
        if let Some(slot) = world.entities.get(entity_index) {
            let life = slot.entity().life();
            print!(
                "  | ID:{:<2} HP:{:<3} EN:{:<3} AGE:{:<3} Ph:{:?}",
                slot.id, life.health, life.energy, life.age, slot.phase
            );
        }

        println!(); // Alt satıra geç
    }
    println!("{:-<1$}", "", map_width + 5);
    println!("@: Canlı | X: Ceset | f: Yemek | .: Boş");
}

```

## src/lib.rs
```
// Modülü dahil et
pub mod creatures;
pub mod entity;
pub mod map;
pub mod world;

use std::sync::atomic::{AtomicU64, Ordering};

/// Simülasyonda ki chunk büyüklüğü
pub const CHUNK_SIZE: usize = 16;

/// Rastgele sayı üretmek için tohum
static RNG_STATE: AtomicU64 = AtomicU64::new(12345);

/// Tohumu günceller
pub fn set_global_seed(seed: u64) {
    RNG_STATE.store(seed, Ordering::Relaxed);
}

/// Tohumu zaman damgası ile günceller
pub fn set_global_seed_with_time() {
    set_global_seed(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    );
}

/// Bir sonraki rastgele sayıyı atomik olarak üretir
pub fn next_rand() -> u64 {
    // fetch_update: Mevcut değeri güvenli bir şekilde okur,
    // hesaplamayı yapar ve kimse araya girmeden yeni değeri yazar.
    RNG_STATE
        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |state| {
            Some(state.wrapping_mul(6364136223846793005).wrapping_add(1))
        })
        .unwrap_or(0)
}

/// [min, max] aralığında sayı üretir
pub fn gen_range(min: isize, max: isize) -> isize {
    let range = (max - min).abs() as u64;
    if range == 0 {
        return min;
    }
    let rand_val = next_rand() % (range + 1);
    min + rand_val as isize
}

```

## src/world.rs
```
use crate::{
    entity::{Entity, intent::Intent, perception::*, phase::EntityPhase},
    map::{Map, position::Position},
};
use std::collections::{HashMap, HashSet};

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
        // Haritanın %10 kısmına rastgele kaynak yerleştir.
        map.populate_resources(0.1f32);
        // Döndür
        World { map, entities }
    }
    /// Her tick, simülasyonun bir adımıdır.
    pub fn tick(&mut self) {
        let mut intents: Vec<(usize, Intent)> = Vec::new();

        // ------------------------------
        // 1. Her entity için perception ve intent oluştur
        // ------------------------------
        for slot in &self.entities {
            if !slot.phase.is_active() {
                continue; // Sadece aktif canlılar karar verir
            }

            let perception = self.build_perception(slot);
            let intent = slot.entity().make_intent(perception);
            intents.push((slot.id, intent));
        }

        // ------------------------------
        // 2. Intentleri çöz
        // ------------------------------
        self.resolve_intent(intents);

        // ------------------------------
        // 3. Canlıların tick güncellemelerini uygula (yaş, enerji, speed reset vb.)
        // ------------------------------
        for slot in &mut self.entities {
            slot.entity_mut().tick();
        }

        // ------------------------------
        // 4. Fazları güncelle ve ölüleri işaretle
        // ------------------------------
        for slot in &mut self.entities {
            slot.phase.tick();

            if slot.phase == EntityPhase::Active && !slot.entity().life().is_alive() {
                slot.phase = EntityPhase::Corpse { remaining: 50 }; // Ceset 50 tick kalacak
            }
        }

        // ------------------------------
        // 5. Removed aşamasındaki entityleri sil
        // ------------------------------
        self.entities
            .retain(|slot| !matches!(slot.phase, EntityPhase::Removed));
    }

    /// Intentleri çöz ve uygulama fonksiyonu
    pub fn resolve_intent(&mut self, intents: Vec<(usize, Intent)>) {
        // ID -> pozisyon haritası
        let mut planned_positions: HashMap<usize, Position> = HashMap::new();

        for (id, intent) in intents {
            if let Some(slot) = self.entities.iter_mut().find(|s| s.id == id) {
                match intent {
                    Intent::Move { steps } => {
                        if steps.is_empty() {
                            continue;
                        }

                        // İlk adımı uygula
                        let dir = steps[0];
                        let new_pos = slot.pos.offset(dir);

                        // Eğer hedef hücre boş ve walkable ise
                        if self.map.in_bounds(new_pos) && self.map.is_walkable(new_pos) {
                            // Çakışma yoksa hareket et
                            if !planned_positions.values().any(|&p| p == new_pos) {
                                slot.pos = new_pos;
                                planned_positions.insert(id, new_pos);
                                slot.entity_mut().life_mut().on_move();
                            }
                        }
                    }

                    Intent::Eat { at, corpse_id: _ } => {
                        if at.is_empty() {
                            continue;
                        }

                        let dir = at[0];
                        let target_pos = slot.pos.offset(dir);

                        if let Some(cell) = self.map.cell(target_pos) {
                            match cell {
                                crate::map::cell::Cell::Food { amount } => {
                                    // Yemeği tüket
                                    let eat_amount: usize = *amount.min(&10usize); // Basit: max 10 enerji
                                    slot.entity_mut().life_mut().restore_energy(eat_amount);
                                    self.map.reduce_cell_amount(target_pos, eat_amount);
                                }
                                _ => {}
                            }
                        }
                    }

                    Intent::Mate { target_id } => {
                        if let Some(target_slot) = self.entities.iter().find(|s| s.id == target_id)
                        {
                            // Hedef canlı da uygun mu?
                            if target_slot.phase.is_active()
                                && target_slot.entity().life().can_reproduce()
                            {
                                // Yeni yavru oluştur
                                let child = slot.entity().reproduce();
                                let child_pos = slot.pos; // Basit: yavru aynı pozisyonda
                                let new_id =
                                    self.entities.iter().map(|s| s.id).max().unwrap_or(0) + 1;
                                let new_slot = crate::world::EntitySlot::new(
                                    new_id,
                                    child_pos,
                                    crate::entity::phase::EntityPhase::Active,
                                    child,
                                );
                                self.entities.push(new_slot);

                                // Yavrulama sonrası enerji düş
                                slot.entity_mut().life_mut().on_reproduce();
                            }
                        }
                    }

                    Intent::Flee { target_id: _ } => {
                        // Basit: sadece rastgele bir boş yöne git
                        use crate::map::direction::Direction::*;
                        let dirs = [Up, Down, Left, Right];
                        let dir = dirs[crate::gen_range(0, dirs.len() as isize - 1) as usize];
                        let new_pos = slot.pos.offset(dir);

                        if self.map.in_bounds(new_pos) && self.map.is_walkable(new_pos) {
                            if !planned_positions.values().any(|&p| p == new_pos) {
                                slot.pos = new_pos;
                                planned_positions.insert(id, new_pos);
                                slot.entity_mut().life_mut().on_move();
                            }
                        }
                    }

                    Intent::Attack { target_id: _ } => {
                        // Şimdilik implement edilmedi
                    }

                    Intent::Idle { duration: _ } | Intent::Sleep { duration: _ } => {
                        // Pasif
                    }
                }
            }
        }
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

```

## src/entity/lifestate.rs
```
//use crate::entity::{intent::Intent, phase::EntityPhase};

/// Bu struct hem:
/// - genetik (sabit) bilgileri
/// - dinamik (tick ile değişen) bilgileri
/// birlikte tutar.
/// Ayrım yorumlar ve yardımcı fonksiyonlarla yapılır.
#[derive(Debug, Clone)]
pub struct LifeState {
    // -------- GENETİK (SABİT) --------
    /// Maksimum yaş (tick cinsinden)
    pub max_age: usize,

    /// Maksimum can
    pub max_health: usize,

    /// Maksimum enerji
    pub max_energy: usize,

    /// Üreme için minimum yaş
    pub maturity_age: usize,

    /// Canlının görüş açısı
    pub vision_range: usize, // Örn: 6

    // -------- DİNAMİK (DEĞİŞEN) --------
    /// Şu ana kadar geçen tick sayısı
    pub age: usize,

    /// Anlık can
    pub health: usize,

    /// Anlık enerji
    pub energy: usize,

    /// Son çiftleşmeden sonra kalan bekleme süresi
    pub reproduction_cooldown: usize,

    /// Tick başına maksimum hareket hakkı
    pub speed: usize,

    /// Bu tick içinde kullanılan hareket sayısı
    pub moves_used: usize,
}

impl LifeState {
    /// Her tick başında çağrılır.
    /// Hareket hakkı resetlenir.
    pub fn tick(&mut self) {
        // Yaşlanma
        self.age += 1;

        // Yaşlılıktan ölüm
        if self.age > self.max_age {
            self.health = 0;
            // Kendine not: Yaşlılıktan ölmek yerine her turda 5 can alacak şekilde değiştirilebilir.
            return; // Yaşlandığı için ekstra bir hesaplamaya gerek yok
        }

        // Üreme bekleme süresi
        if self.reproduction_cooldown > 0 {
            self.reproduction_cooldown -= 1;
        }

        // Pasif iyileşme süreci
        // 2 enerji'ye 1 can düşer; değerler değişebilir şimdilik bu
        if !self.is_energy_low() && self.health < self.max_health {
            self.consume_energy(2);
            self.heal(1);
        }

        // Can karşılığında Enerji kazanma
        // Enerji 0 ise, Can yakarak Enerji kazanma
        if self.energy == 0 && !self.is_health_low() {
            self.health -= 1;
            self.restore_energy(2);
        }

        // Bu tick için hareket sayacı sıfırlanır
        self.moves_used = 0;
    }

    // ===============================
    // DURUM SORGULARI
    // ===============================
    /// Enerji düşük kabul edilen eşik
    pub fn low_energy_threshold(&self) -> usize {
        self.max_energy / 4
    }
    /// Can düşük kabul edilen eşik
    pub fn low_health_threshold(&self) -> usize {
        self.max_health / 4
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0
    }

    pub fn is_mature(&self) -> bool {
        self.age >= self.maturity_age
    }

    pub fn is_energy_low(&self) -> bool {
        self.energy <= self.low_energy_threshold()
    }

    pub fn is_energy_full(&self) -> bool {
        self.energy >= self.max_energy
    }

    pub fn is_health_low(&self) -> bool {
        self.health <= self.low_health_threshold()
    }

    pub fn is_health_full(&self) -> bool {
        self.health >= self.max_health
    }

    // LifeState içinde
    pub fn can_reproduce(&self) -> bool {
        (self.age >= self.maturity_age) && (self.reproduction_cooldown == 0 && self.energy > 15)
        // Çok düşük tut ki ölmeden hemen önce bile deneyebilsinler
    }

    /// Bu tick içinde hareket edebilir mi?
    pub fn can_move(&self) -> bool {
        self.moves_used < self.speed
    }

    // ===============================
    // DURUM DEĞİŞTİRİCİLER
    // ===============================

    /// Bir hareket kullanıldığında çağrılır
    pub fn on_move(&mut self) {
        self.moves_used += 1;
        self.consume_energy(1);
    }

    pub fn consume_energy(&mut self, amount: usize) {
        self.energy = self.energy.saturating_sub(amount);
    }

    pub fn restore_energy(&mut self, amount: usize) {
        // Enerjiyi artır ama maksimum kapasiteyi aşma
        self.energy = (self.energy + amount).min(self.max_energy);
    }

    pub fn heal(&mut self, amount: usize) {
        self.health = (self.health + amount).min(self.max_health);
    }

    pub fn on_reproduce(&mut self) {
        println!("Entity is reproducing...");
        self.reproduction_cooldown = 100;
        self.consume_energy(10);
    }

    /*
    pub fn metabolic_cost(&self, phase: &EntityPhase, intent: Option<&Intent>) -> usize {
        // 1. Bazal Metabolizma Hızı (BMR): Sadece hayatta kalmak için gereken min. enerji
        let bmr = 1;

        match phase {
            // Ölüler enerji harcamaz
            EntityPhase::Corpse { .. } | EntityPhase::Removed => 0,

            // Uyku Modu: En düşük maliyet. Görüş kapalı, hareket yok.
            EntityPhase::Sleeping { .. } => bmr,

            // Aktif Mod: Canlı uyanık ve çevresini işliyor.
            EntityPhase::Active => {
                let mut cost = bmr;

                // Algı Maliyeti: Geniş bir alanı taramak (vision_range) beyin/göz yorar.
                cost += self.vision_range / 5; // Örn: Her 5 birim görüş +1 maliyet

                // Niyet (Aksiyon) Maliyeti:
                if let Some(action) = intent {
                    match *action {
                        Intent::Move { steps } | Intent::Flee { target_id: steps } => {
                            // Hareket maliyeti: Hız ve atılan adım sayısı ile orantılı
                            cost += self.speed + (steps.len() / 2);
                        }
                        Intent::Mate { .. } => {
                            cost += 5; // Üreme çok yüksek enerji gerektirir
                        }
                        Intent::Eat { .. } => {
                            cost += 1; // Sindirim ve çiğneme eforu
                        }
                        Intent::Idle { .. } => {
                            // Idle (Bekleme): Ekstra maliyet yok, sadece BMR + Algı.
                        }
                    }
                }
                cost
            }
        }
    }
    */
}

```

## src/entity/mod.rs
```
pub mod intent;
pub mod lifestate;
pub mod perception;
pub mod phase;
pub mod species;

use crate::entity::{intent::Intent, lifestate::LifeState, perception::*, species::Species};

/// Canlının temel alacağı arayüz
pub trait Entity {
    /// Canlının yaşam durumu (genetik + dinamik)
    fn life(&self) -> &LifeState;
    fn life_mut(&mut self) -> &mut LifeState;

    /// Varlık türü
    fn species(&self) -> Species;

    /// Karar verme (sadece okuma yapmalı)
    fn make_intent(&self, view: Perception) -> Intent;

    /// Tek tick güncellemesi
    /// World'un işini kolaylaştırmak için var;
    fn tick(&mut self);

    /// Canlının kendi türünden yeni bir üye (yavru) oluşturmasını sağlar.
    /// World bu metodu çağırır ama dönen somut türü (Herbivore vs.) bilmez.
    fn reproduce(&self) -> Box<dyn Entity>;
}

```

## src/entity/perception.rs
```
use std::{
    ops::{Add, AddAssign},
    collections::HashMap
};
use crate::{entity::species::Species, map::direction::{Direction, Steps}};

/// Algılanan tekil hedef
#[derive(Debug, Clone)]
pub struct PerceivedEntity {
    /// Algılanan canlının kimliği (Kaldırılabilir, Emin değilim)
    pub id: usize,
    /// Algılanan canlının türü
    pub species: Species,
    /// Algılanan canlının yön ve mesafe bilgisi
    pub steps: Steps,
}

/// Algılanan yiyecek
#[derive(Debug, Clone)]
pub struct PerceivedFood {
    /// Algılanan yemeğin miktarı
    pub amount: usize,
    /// Algılanan yemek ceset mi?
    pub is_corpse: bool,
    /// Algılanan yemeğin yön ve mesafe bilgisi
    pub steps: Steps,
}

/// Canlının görüş açısında olan Yemekler, Diğer canlılar, Gidebiliceği Mesafe
/// - Bu pozisyonda canlı var mı ve kaç tane var?
/// - Canlı mı / ceset mi?
/// - Yakın çevrede kimler var?
#[derive(Debug, Clone)]
pub struct Perception {
    /// Algılanan yemekler
    pub foods: Vec<PerceivedFood>,
    /// Algılanan canlılar
    pub entities: Vec<PerceivedEntity>,
    /// Gidilebilicek mesafe, u8 değeri hangi yöne kaç adımı gidebiliceği simgeler
    pub directions: HashMap<Direction, u8>,
}

impl Perception {
    /// Boş bir görüş açısı oluştur
    pub fn empty() -> Self {
        Self {
            foods: Vec::new(),
            entities: Vec::new(),
            directions: HashMap::new(),
        }
    }
    /// Algılanan yiyeceğe adım ekle
    pub fn add_food(&mut self, amount: usize, is_corpse: bool, steps: Steps) {
        self.foods.push(PerceivedFood {
            amount,
            is_corpse,
            steps,
        });
    }

    /// Algılanan canlıya adım ekle
    pub fn add_entity(&mut self, id: usize, species: Species, steps: Steps) {
        self.entities.push(PerceivedEntity { id, species, steps });
    }

    /// Bir yöne adım ekle veya mevcut adımı güncelle
    pub fn add_direction(&mut self, dir: Direction, distance: u8) {
        self.directions
        .entry(dir)
        .and_modify(|d| *d = (*d).max(distance))
        .or_insert(distance);
    }
}

impl Add<Direction> for PerceivedEntity {
    type Output = Self;
    fn add(mut self, dir: Direction) -> Self {
        self.steps += dir;
        self
    }
}

impl Add<Steps> for PerceivedEntity {
    type Output = Self;
    fn add(mut self, steps: Steps) -> Self {
        self.steps += steps;
        self
    }
}

impl AddAssign<Direction> for PerceivedEntity {
    fn add_assign(&mut self, dir: Direction) {
        self.steps += dir;
    }
}

impl AddAssign<Steps> for PerceivedEntity {
    fn add_assign(&mut self, steps: Steps) {
        self.steps += steps;
    }
}

impl Add<Direction> for PerceivedFood {
    type Output = Self;
    fn add(mut self, dir: Direction) -> Self {
        self.steps += dir;
        self
    }
}

impl Add<Steps> for PerceivedFood {
    type Output = Self;
    fn add(mut self, steps: Steps) -> Self {
        self.steps += steps;
        self
    }
}

impl AddAssign<Direction> for PerceivedFood {
    fn add_assign(&mut self, dir: Direction) {
        self.steps += dir;
    }
}

impl AddAssign<Steps> for PerceivedFood {
    fn add_assign(&mut self, steps: Steps) {
        self.steps += steps;
    }
}

impl Add<PerceivedEntity> for Perception {
    type Output = Self;

    fn add(mut self, entity: PerceivedEntity) -> Self {
        self.entities.push(entity);
        self
    }
}

impl AddAssign<PerceivedEntity> for Perception {
    fn add_assign(&mut self, entity: PerceivedEntity) {
        self.entities.push(entity);
    }
}

impl Add<PerceivedFood> for Perception {
    type Output = Self;

    fn add(mut self, food: PerceivedFood) -> Self {
        self.foods.push(food);
        self
    }
}

impl AddAssign<PerceivedFood> for Perception {
    fn add_assign(&mut self, food: PerceivedFood) {
        self.foods.push(food);
    }
}

```

## src/entity/phase.rs
```
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

```

## src/entity/intent.rs
```
use crate::map::direction::Direction;

/// World -> Perception -> Intent şeklinde yola koyulur.
/// World, canlının bulunduğu konumu baz alarak Perception oluşturur.
/// Entity, bu Perception ile kendi içerisinde ki özel mekanizma ile karar alır
/// BU KARAR KESİNLİK DEĞİLDİR, WORLD SON SÖZÜ SÖYLER
/// ÇAKIŞAN NİYETLER İÇİN WORLD İNSİYATİF ALABİLİR
#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    /// Gidilmek istenilen nokta
    Move { steps: Vec<Direction> },
    /// Yenilmek istenilen yemeğin konumu,
    /// Not: Yemek aynı hücrede ise at okunmaz,
    /// miktar canlının yiyebiliceği ve World izin verdiği miktarda olur
    Eat { at: Vec<Direction>, corpse_id: Option<usize> },
    /// Çiftleşmek istenilen canlı
    Mate { target_id: usize },
    /// Saldırılmak istenilen canlı
    Attack { target_id: usize },
    /// Kaçınılmak istenilen canlı
    Flee { target_id: usize },
    /// Bekleme niyeti, iyileşme için (yavaş)
    Idle { duration: usize },
    /// Keyfi olarak uyuma eylemi, iyileşme için (hızlı)
    Sleep { duration: usize },
}

```

## src/entity/species.rs
```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Species {
    Herbivore,
    Carnivore,
    Omnivore,
}

```

## src/map/mod.rs
```
pub mod cell;
pub mod direction;
pub mod position;

use std::collections::{HashMap, VecDeque};

use crate::{
    CHUNK_SIZE, gen_range,
    map::{cell::Cell, direction::Direction, direction::Steps, position::Position},
    next_rand,
};

struct Chunk {
    cells: Vec<Cell>,
}

impl Chunk {
    fn new() -> Self {
        Self {
            cells: vec![Cell::Empty; CHUNK_SIZE * CHUNK_SIZE],
        }
    }

    /// Hücre indexi oluştur
    #[inline]
    fn idx(x: usize, y: usize) -> usize {
        y * CHUNK_SIZE + x
    }

    /// Hücreyi oku
    fn cell(&self, x: usize, y: usize) -> &Cell {
        &self.cells[Self::idx(x, y)]
    }

    /// Hücreyi değiştir
    fn cell_mut(&mut self, x: usize, y: usize) -> &mut Cell {
        &mut self.cells[Self::idx(x, y)]
    }

    /// Hücre tamamen boşalmış mı?
    fn is_completely_empty(&self) -> bool {
        self.cells.iter().all(|c| matches!(c, Cell::Empty))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoord {
    cx: isize,
    cy: isize,
}

pub struct Map {
    /// Yatay eksende sağ kısım
    pub min_x: isize,
    /// Yatay eksende sol kısım
    pub max_x: isize,
    /// Dikey eksende sağ kısım
    pub min_y: isize,
    /// Dikey eksende sol kısım
    pub max_y: isize,
    /// Parçalara ayrılmış harita.
    chunks: HashMap<ChunkCoord, Chunk>,
}

impl Map {
    /// Sınırları kontrol ederek güvenli bir dünya oluşturur
    pub fn new(x1: isize, x2: isize, y1: isize, y2: isize) -> Self {
        // Kullanıcı değerleri ters girse bile (min/max) doğru eşleştirilir
        let (min_x, max_x) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
        let (min_y, max_y) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };

        Self {
            min_x,
            max_x,
            min_y,
            max_y,
            chunks: HashMap::new(),
        }
    }

    /// Bir dünya pozisyonunun hangi chunk koordinatına denk geldiğini döndürür
    pub fn chunk_coord(pos: Position) -> ChunkCoord {
        ChunkCoord {
            cx: pos.x.div_euclid(CHUNK_SIZE as isize),
            cy: pos.y.div_euclid(CHUNK_SIZE as isize),
        }
    }

    fn local_coord(pos: Position) -> (usize, usize) {
        (
            pos.x.rem_euclid(CHUNK_SIZE as isize) as usize,
            pos.y.rem_euclid(CHUNK_SIZE as isize) as usize,
        )
    }

    fn index_of(&self, pos: Position) -> (ChunkCoord, usize, usize) {
        let cc = Self::chunk_coord(pos);
        let (lx, ly) = Self::local_coord(pos);
        (cc, lx, ly)
    }

    pub fn in_bounds(&self, pos: Position) -> bool {
        pos.x >= self.min_x && pos.x <= self.max_x && pos.y >= self.min_y && pos.y <= self.max_y
    }

    pub fn cell(&self, pos: Position) -> Option<&Cell> {
        if !self.in_bounds(pos) {
            return None;
        }
        let (cc, lx, ly) = self.index_of(pos);
        self.chunks.get(&cc).map(|c| c.cell(lx, ly))
    }

    pub fn is_cell(&self, pos: Position, expected: &Cell) -> bool {
        self.cell(pos).map(|c| c == expected).unwrap_or(false)
    }

    pub fn is_walkable(&self, pos: Position) -> bool {
        matches!(
            self.cell(pos),
            Some(Cell::Empty | Cell::Food { .. } | Cell::Water { .. })
        )
    }

    pub fn set_cell(&mut self, pos: Position, cell: Cell) {
        // 1. Adım: Dünya sınırları kontrolü
        if !self.in_bounds(pos) {
            return;
        }

        let (cc, lx, ly) = self.index_of(pos);

        // 2. Adım: Eğer hücre boşsa ve chunk yoksa, boş bir hücre için yeni chunk yaratma.
        if cell == Cell::Empty && !self.chunks.contains_key(&cc) {
            return;
        }

        // 3. Adım: Chunk'ı al veya oluştur, ardından hücreyi yaz
        let chunk = self.chunks.entry(cc).or_insert_with(Chunk::new);
        *chunk.cell_mut(lx, ly) = cell;
    }

    pub fn reduce_cell_amount(&mut self, pos: Position, amount: usize) -> bool {
        if !self.in_bounds(pos) {
            return false;
        }

        let (cc, lx, ly) = self.index_of(pos);

        let should_remove = {
            let chunk = match self.chunks.get_mut(&cc) {
                Some(c) => c,
                None => return false,
            };

            match chunk.cell_mut(lx, ly) {
                Cell::Food { amount: a } | Cell::Water { amount: a } => {
                    *a = a.saturating_sub(amount);
                    if *a == 0 {
                        *chunk.cell_mut(lx, ly) = Cell::Empty;
                    }
                }
                _ => return false,
            }
            // Hücre boşaldıktan sonra chunk'ın durumunu kontrol et
            chunk.is_completely_empty()
        };

        if should_remove {
            self.chunks.remove(&cc);
        }
        true
    }

    pub fn clear_cell(&mut self, pos: Position) {
        self.set_cell(pos, Cell::Empty);
    }

    /// Bir yönde engel gelene kadar kaç adım?
    pub fn walkable_distance(&self, from: Position, dir: Direction) -> u8 {
        let mut cur = from;
        let mut steps = 0u8;

        loop {
            let next = cur + dir;
            if !self.in_bounds(next) || !self.is_walkable(next) {
                break;
            }
            steps += 1;
            cur = next;
            if steps == u8::MAX {
                break;
            }
        }
        steps
    }

    pub fn walkable_distances(&self, from: Position) -> HashMap<Direction, u8> {
        let mut map = HashMap::new();
        for d in [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ] {
            map.insert(d, self.walkable_distance(from, d));
        }
        map
    }

    /// Radius ile sınırlı BFS
    pub fn bfs_steps_to(&self, start: Position, goal: Position, radius: usize) -> Option<Steps> {
        if !self.in_bounds(goal) || !self.is_walkable(goal) {
            return None;
        }

        let mut queue = VecDeque::new();
        let mut came_from: HashMap<Position, (Position, Direction)> = HashMap::new();

        queue.push_back(start);

        while let Some(current) = queue.pop_front() {
            if current == goal {
                break;
            }

            let dist = (current.x - start.x).abs() + (current.y - start.y).abs();
            if dist as usize >= radius {
                continue;
            }
            for dir in [
                Direction::Up,
                Direction::Down,
                Direction::Left,
                Direction::Right,
                Direction::UpLeft,
                Direction::UpRight,
                Direction::DownLeft,
                Direction::DownRight,
            ] {
                let next = current + dir;
                if !self.in_bounds(next) || !self.is_walkable(next) {
                    continue;
                }
                if came_from.contains_key(&next) || next == start {
                    continue;
                }

                came_from.insert(next, (current, dir));
                queue.push_back(next);
            }
        }

        // Path reconstruction
        let mut steps = Vec::new();
        let mut cur = goal;

        while cur != start {
            let (prev, dir) = *came_from.get(&cur)?;
            steps.push(dir);
            cur = prev;
        }

        steps.reverse();
        Some(Steps(steps))
    }

    pub fn scan_foods_within(
        &self,
        center: Position,
        radius: usize,
    ) -> Vec<(Position, Steps, usize)> {
        let mut result = Vec::new();

        for y in (center.y - radius as isize)..=(center.y + radius as isize) {
            for x in (center.x - radius as isize)..=(center.x + radius as isize) {
                let pos = Position { x, y };

                if !self.in_bounds(pos) {
                    continue;
                }

                let manhattan = (center.x - x).abs() + (center.y - y).abs();
                if manhattan as usize > radius {
                    continue;
                }

                if let Some(Cell::Food { amount }) = self.cell(pos) {
                    if let Some(steps) = self.bfs_steps_to(center, pos, radius) {
                        result.push((pos, steps, *amount));
                    }
                }
            }
        }

        result
    }

    /// Tüm haritayı chunk chunk doldurur (Orkestra Şefi)
    pub fn populate_resources(&mut self, density: f32) {
        // Haritanın kapsadığı chunk sınırlarını hesapla
        let min_cx = self.min_x / CHUNK_SIZE as isize;
        let max_cx = self.max_x / CHUNK_SIZE as isize;
        let min_cy = self.min_y / CHUNK_SIZE as isize;
        let max_cy = self.max_y / CHUNK_SIZE as isize;

        for cx in min_cx..=max_cx {
            for cy in min_cy..=max_cy {
                let coord = ChunkCoord { cx, cy };
                // İşi uzmanına (populate_chunk) devret
                self.populate_chunk(coord, density);
            }
        }
    }

    /// Sadece belirli bir chunk içine odaklanır (Uzman)
    pub fn populate_chunk(&mut self, coord: ChunkCoord, density: f32) {
        let start_x = coord.cx * CHUNK_SIZE as isize;
        let start_y = coord.cy * CHUNK_SIZE as isize;

        // Bir chunk içindeki toplam deneme sayısı (16x16 = 256 hücre)
        let spawn_attempts = ((CHUNK_SIZE * CHUNK_SIZE) as f32 * density) as usize;

        for _ in 0..spawn_attempts {
            let lx = gen_range(0, (CHUNK_SIZE - 1) as isize);
            let ly = gen_range(0, (CHUNK_SIZE - 1) as isize);
            let world_pos = Position::new(start_x + lx, start_y + ly);

            // Sınır ve boşluk kontrolü
            if self.in_bounds(world_pos)
                && self
                    .cell(world_pos)
                    .map_or(true, |c| matches!(c, Cell::Empty))
            {
                let roll = next_rand() % 100;
                let cell = if roll < 70 {
                    Cell::Food {
                        amount: (next_rand() % 10 + 5) as usize,
                    }
                } else {
                    Cell::Water {
                        amount: (next_rand() % 15 + 10) as usize,
                    }
                };
                self.set_cell(world_pos, cell);
            }
        }
    }
}

```

## src/map/position.rs
```
use std::ops::Add;

use crate::map::direction::Direction;

/// - Dünya koordinatıdır (chunk bağımsız)
/// - Negatif koordinatları destekler
/// - (0,0) merkezli dünya için uygundur
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: isize,
    pub y: isize,
}

impl Position {
    /// Yeni bir pozisyon oluştur
    pub fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }

    /// Pozisyonu doğrudan güncelle
    pub fn set(&mut self, other: Position) {
        self.x = other.x;
        self.y = other.y;
    }

    /// Manhattan mesafesi
    /// Çapraz yönler olsa bile karar mekanizması için hâlâ en stabil metriktir
    pub fn distance_to(&self, other: Position) -> usize {
        ((self.x - other.x).abs() + (self.y - other.y).abs()) as usize
    }

    /// Yön bazlı yeni pozisyon (immutable)
    pub fn offset(&self, dir: Direction) -> Position {
        *self + dir
    }
}

impl From<(isize, isize)> for Position {
    fn from(t: (isize, isize)) -> Position {
        Position { x: t.0, y: t.1 }
    }
}

/// Position + Direction → Position
///
/// Çapraz hareketler desteklenir.
/// World isterse çaprazı yasaklayabilir (Map / validation katmanı).
impl Add<Direction> for Position {
    type Output = Position;

    fn add(self, dir: Direction) -> Position {
        match dir {
            Direction::Up => Position {
                x: self.x,
                y: self.y - 1,
            },
            Direction::Down => Position {
                x: self.x,
                y: self.y + 1,
            },
            Direction::Left => Position {
                x: self.x - 1,
                y: self.y,
            },
            Direction::Right => Position {
                x: self.x + 1,
                y: self.y,
            },
            Direction::UpLeft => Position {
                x: self.x - 1,
                y: self.y - 1,
            },
            Direction::UpRight => Position {
                x: self.x + 1,
                y: self.y - 1,
            },
            Direction::DownLeft => Position {
                x: self.x - 1,
                y: self.y + 1,
            },
            Direction::DownRight => Position {
                x: self.x + 1,
                y: self.y + 1,
            },
        }
    }
}

```

## src/map/cell.rs
```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Food { amount: usize },
    Water { amount: usize },
}

```

## src/map/direction.rs
```
use std::ops::{Add, AddAssign};
#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

/// Hareket etme talimat dizisi
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Steps(pub Vec<Direction>);

impl Steps {
    /// Boş bir hareket dizisi oluşturur
    pub fn empty() -> Steps {
        Steps(Vec::new())
    }

    /// Girdiyi Hareket talimatı yapısına ekler
    pub fn new(value: Vec<Direction>) -> Steps {
        Steps(value)
    }

    /// Listenin başından bir eleman al
    /// Eğer boşsa None döner
    pub fn pop_front(&mut self) -> Option<Direction> {
        if self.0.is_empty() {
            None
        } else {
            Some(self.0.remove(0))
        }
    }

    /// Listenin başındaki elemanı gör ama silme
    pub fn peek_front(&self) -> Option<&Direction> {
        self.0.first()
    }

    /// Vec<Direction> ekle
    pub fn extend(&mut self, other: Steps) {
        self.0.extend(other.0);
    }

    /// Iterator ile erişim
    pub fn iter(&self) -> std::slice::Iter<'_, Direction> {
        self.0.iter()
    }

    /// Mutable iterator
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Direction> {
        self.0.iter_mut()
    }
    /// Adım sayısını döner
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Boş mu kontrol
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// IntoIterator implementasyonu (for x in steps)
impl IntoIterator for Steps {
    type Item = Direction;
    type IntoIter = std::vec::IntoIter<Direction>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// &Steps için iterator
impl<'a> IntoIterator for &'a Steps {
    type Item = &'a Direction;
    type IntoIter = std::slice::Iter<'a, Direction>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// &mut Steps için iterator
impl<'a> IntoIterator for &'a mut Steps {
    type Item = &'a mut Direction;
    type IntoIter = std::slice::IterMut<'a, Direction>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

// + Direction -> yeni Steps
impl Add<Direction> for Steps {
    type Output = Steps;

    fn add(mut self, rhs: Direction) -> Steps {
        self.0.push(rhs);
        self
    }
}

// + Vec<Direction> -> yeni Steps
impl Add<Vec<Direction>> for Steps {
    type Output = Steps;

    fn add(mut self, rhs: Vec<Direction>) -> Steps {
        self.0.extend(rhs);
        self
    }
}

/// `Steps += Direction` ile sonuna ekleme
impl AddAssign<Direction> for Steps {
    fn add_assign(&mut self, rhs: Direction) {
        self.0.push(rhs);
    }
}

/// `Steps += Vec<Direction>` ile birden fazla ekleme
impl AddAssign<Vec<Direction>> for Steps {
    fn add_assign(&mut self, rhs: Vec<Direction>) {
        self.0.extend(rhs);
    }
}

/// `Steps += Steps` ile birden fazla ekleme
impl AddAssign<Steps> for Steps {
    fn add_assign(&mut self, rhs: Steps) {
        self.0.extend(rhs);
    }
}

/// Vec<Direction> -> Steps
impl From<Vec<Direction>> for Steps {
    fn from(vec: Vec<Direction>) -> Steps {
        Steps(vec)
    }
}

/// Steps -> Vec<Direction>
impl From<Steps> for Vec<Direction> {
    fn from(steps: Steps) -> Vec<Direction> {
        steps.0
    }
}

```

## src/creatures/mod.rs
```
pub mod herbivore;

```

## src/creatures/herbivore.rs
```
use crate::entity::{
    Entity, intent::Intent, lifestate::LifeState, perception::Perception, species::Species,
};

pub struct HerbivoreEntity {
    pub life_state: LifeState,
}

impl HerbivoreEntity {
    pub fn new(life_state: LifeState) -> Self {
        Self { life_state }
    }

    pub fn default() -> Self {
        Self {
            life_state: LifeState {
                max_age: 105,
                max_health: 120,
                max_energy: 80,
                maturity_age: 20,
                vision_range: 6,
                age: 0,
                health: 120,
                energy: 80,
                reproduction_cooldown: 0,
                speed: 3,
                moves_used: 0,
            },
        }
    }
}

impl Entity for HerbivoreEntity {
    fn life(&self) -> &LifeState {
        &self.life_state
    }

    fn life_mut(&mut self) -> &mut LifeState {
        &mut self.life_state
    }

    fn species(&self) -> Species {
        Species::Herbivore
    }

    fn make_intent(&self, perception: Perception) -> Intent {
        // ===============================
        // 1. Yakınında yiyecek var mı?
        // ===============================
        if perception.foods.is_empty() {
            // Yiyecek yok → ara
            // Basit: rastgele bir yön seç
            use crate::map::direction::Direction::*;
            let dirs = [Up, Down, Left, Right];
            let dir = dirs[crate::gen_range(0, dirs.len() as isize - 1) as usize];
            Intent::Move { steps: vec![dir] }
        } else {
            // Yiyecek var → yemeyi planla
            let nearest_food = &perception.foods[0]; // Basit: ilk bulduğu yiyecek
            // Eğer tok değilse ye
            if !self.life_state.is_energy_full() {
                Intent::Eat {
                    at: vec![nearest_food.steps.0[0]],
                    corpse_id: None,
                }
            } else if self.life_state.can_reproduce() {
                // Tok ve üreme zamanı → eş ara
                if let Some(target) = perception
                    .entities
                    .iter()
                    .find(|e| e.species == Species::Herbivore)
                {
                    Intent::Mate {
                        target_id: target.id,
                    }
                } else {
                    // Eş yoksa yakındaki bir yere hareket et
                    Intent::Idle { duration: 1 }
                }
            } else {
                // Tok ama üreme zamanı değil → bekle
                Intent::Idle { duration: 1 }
            }
        }
    }

    fn tick(&mut self) {
        self.life_state.tick();
    }

    fn reproduce(&self) -> Box<dyn Entity> {
        let mut child_life = self.life_state.clone();
        child_life.age = 0;
        child_life.energy = child_life.max_energy / 2;
        child_life.health = child_life.max_health / 2;
        Box::new(HerbivoreEntity::new(child_life))
    }
}

```

