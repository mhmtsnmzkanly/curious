## src/main.rs
```
fn main() {}

```

## src/lib.rs
```
// Modülü dahil et
pub mod creatures;
pub mod entity;
pub mod map;
pub mod world;

pub fn generate_random_id() -> usize {
    // Geçici bir değişken oluşturup onun bellek adresini alıyoruz
    let variable = 0;
    let address = &variable as *const i32 as usize;

    // Adresi, işlemcinin zaman damgasıyla (TSC) harmanlayarak
    // rastgeleliği artırıyoruz
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as usize;

    // XOR ve bit kaydırma (bit-mixing) ile benzersiz bir sayı üretiyoruz
    let mut x = address ^ timestamp;
    x = x.wrapping_mul(0x517cc1b727220a95);
    x ^= x >> 31;

    x
}

```

## src/world.rs
```
use crate::{
    entity::{Entity, intent::Intent, perception::*, phase::EntityPhase},
    map::{Map, cell::Cell, position::Position},
};
use std::collections::{HashMap, HashSet};

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

    /// ===============================
    /// PERCEPTION BUILDER
    /// ===============================
    pub fn build_perception(
        view: &WorldView,
        center: Position,
        radius: usize,
        self_id: usize,
    ) -> Perception {
        let mut perception = Perception::empty();
        let mut seen_entities: HashSet<usize> = HashSet::new();
        let mut seen_cells: HashSet<Position> = HashSet::new();

        // ===============================
        // 1. ENTITY ALGILAMA
        // ===============================
        for (pos, id) in view.nearby_entities(center, radius) {
            if id == self_id || !seen_entities.insert(id) {
                continue;
            }

            match view.entity_phase.get(&id) {
                Some(p) if p.is_corpse() => {
                    perception.foods.push(PerceivedFood {
                        position: pos,
                        amount: 10, // ileride ceset ağırlığına bağlanabilir
                        is_corpse: true,
                    });
                }
                Some(p) if p.is_active() => {
                    perception
                        .enemies
                        .push(PerceivedEntity { id, position: pos });
                    perception.mates.push(PerceivedEntity { id, position: pos });
                }
                _ => {}
            }
        }

        // ===============================
        // 2. HÜCRE ALGILAMA
        // ===============================
        let r = radius as isize;
        let cx = center.x as isize;
        let cy = center.y as isize;

        for dx in -r..=r {
            for dy in -r..=r {
                if dx.abs() + dy.abs() > r {
                    continue;
                }

                let x = cx + dx;
                let y = cy + dy;

                if x < 0 || y < 0 {
                    continue;
                }

                let pos = Position {
                    x: x as usize,
                    y: y as usize,
                };

                if !view.in_bounds(pos) || !seen_cells.insert(pos) {
                    continue;
                }

                match view.cell(pos) {
                    Some(Cell::Food { amount }) if *amount > 0 => {
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

        perception
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
        let actions: Vec<(usize, Intent)> = self
            .entities
            .iter()
            .map(|e| (e.id(), e.make_intent(WorldView::make_percetion(view))))
            .collect();

        // ---------------------------
        // FAZ 3: HAREKET
        // ---------------------------
        let mut move_intents = std::collections::HashMap::new();
        for (id, intent) in &actions {
            if let Intent::Move { to } = intent {
                if let Some(e) = self.entities.iter().find(|ent| ent.id() == *id) {
                    let from = e.position();
                    let to = from + *to;

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

        for (id, intent) in &actions {
            if already_interacted.contains(id) {
                continue;
            }

            match intent {
                Intent::Eat { at } => {
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
                Intent::Mate { target } => {
                    if already_interacted.contains(target) {
                        continue;
                    }

                    let e_idx_opt = self.entities.iter().position(|ent| ent.id() == *id);
                    let t_idx_opt = self.entities.iter().position(|ent| ent.id() == *target);

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
                                already_interacted.insert(*target);
                            }
                        }
                    }
                }

                Intent::Attack { target } => {
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

```

## src/entity/lifestate.rs
```
/// ===============================
/// YAŞAM DURUMU
/// ===============================
///
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

    /// Enerji düşük kabul edilen eşik
    pub low_energy_threshold: usize,

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
    /// ===============================
    /// TICK
    /// ===============================
    ///
    /// Her tick başında çağrılır.
    /// Hareket hakkı resetlenir.
    pub fn tick(&mut self) {
        self.age += 1;

        // Pasif enerji kaybı
        self.energy = self.energy.saturating_sub(1);

        // Üreme bekleme süresi
        if self.reproduction_cooldown > 0 {
            self.reproduction_cooldown -= 1;
        }

        // Yaşlılıktan ölüm
        if self.age >= self.max_age {
            self.health = 0;
        }

        // Bu tick için hareket sayacı sıfırlanır
        self.moves_used = 0;
    }

    // ===============================
    // DURUM SORGULARI
    // ===============================

    pub fn is_alive(&self) -> bool {
        self.health > 0
    }

    pub fn is_mature(&self) -> bool {
        self.age >= self.maturity_age
    }

    pub fn is_energy_low(&self) -> bool {
        self.energy <= self.low_energy_threshold
    }

    pub fn is_energy_full(&self) -> bool {
        self.energy >= self.max_energy
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
}

```

## src/entity/mod.rs
```
pub mod intent;
pub mod lifestate;
pub mod perception;
pub mod phase;

use crate::{
    entity::{intent::Intent, lifestate::LifeState, perception::*, phase::EntityPhase},
    map::position::Position,
};

/// ===============================
/// CANLI ARAYÜZÜ
/// ===============================
pub trait Entity {
    /// Canlıya ait benzersiz kimlik
    fn id(&self) -> usize;

    /// Canlının bulunduğu konum
    fn position(&self) -> Position;
    fn position_mut(&mut self) -> &mut Position;

    /// Canlının yaşam durumu (genetik + dinamik)
    fn life(&self) -> &LifeState;
    fn life_mut(&mut self) -> &mut LifeState;

    // Varlık durumu
    fn phase(&self) -> EntityPhase;
    fn phase_mut(&mut self) -> &mut EntityPhase;

    /// Karar verme (sadece okuma yapmalı)
    fn make_intent(&self, ctx: Perception) -> Intent;

    /// Tek tick güncellemesi
    fn tick(&mut self);

    /// Canlının kendi türünden yeni bir üye (yavru) oluşturmasını sağlar.
    /// World bu metodu çağırır ama dönen somut türü (Herbivore vs.) bilmez.
    fn reproduce(&self, new_id: usize, pos: Position) -> Box<dyn Entity>;
}

```

## src/entity/perception.rs
```
use std::collections::HashSet;

use crate::map::{cell::Cell, position::Position};

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
    pub is_corpse: bool,
}

/// Entity'nin bir tick boyunca algıladığı dünya kesiti
#[derive(Debug, Clone)]
pub struct Perception {
    pub foods: Vec<PerceivedFood>,
    pub enemies: Vec<PerceivedEntity>,
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

```

## src/entity/phase.rs
```
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

```

## src/entity/intent.rs
```
use crate::map::direction::Direction;

/// =======================================================
/// INTENT
/// =======================================================
///
/// Action yerine geçer.
/// Daha zengin, karşılaştırılabilir bir yapı.
/// World bunu VALIDATE eder.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Intent {
    Move { to: Direction },
    Mate { target: usize },
    Eat { at: Direction },
    Attack { target: usize },
    Flee { from: usize },
    Idle,
}

```

## src/map/mod.rs
```
pub mod cell;
pub mod direction;
pub mod position;

use crate::map::{cell::Cell, position::Position};

/// ===============================
/// MAP
/// ===============================
///
/// Map:
/// - Dünyanın çevresel durumunu tutar
/// - Entity bilgisi tutmaz
/// - Sadece "burada ne var?" sorusuna cevap verir
///
/// Entity çakışmaları, canlı/ceset kontrolü World seviyesinde yapılır.
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Cell>,
}

impl Map {
    /// ===============================
    /// KONUM YARDIMCILARI
    /// ===============================

    /// Bu pozisyon harita sınırları içinde mi?
    pub fn in_bounds(&self, pos: Position) -> bool {
        pos.x < self.width && pos.y < self.height
    }

    /// Pozisyondan index üret
    fn index_of(&self, pos: Position) -> Option<usize> {
        if self.in_bounds(pos) {
            Some(pos.y * self.width + pos.x)
        } else {
            None
        }
    }

    /// ===============================
    /// OKUMA
    /// ===============================

    /// Burada ne var?
    pub fn cell(&self, pos: Position) -> Option<&Cell> {
        self.index_of(pos).map(|i| &self.grid[i])
    }

    /// Buradaki şey bu mu?
    pub fn is_cell(&self, pos: Position, expected: &Cell) -> bool {
        self.cell(pos).map(|c| c == expected).unwrap_or(false)
    }

    /// Buraya hareket edilebilir mi?
    ///
    /// Şimdilik:
    /// - Empty -> evet
    /// - Food / Water -> evet
    ///
    /// Entity kontrolü burada yapılmaz.
    pub fn is_walkable(&self, pos: Position) -> bool {
        matches!(
            self.cell(pos),
            Some(Cell::Empty | Cell::Food { .. } | Cell::Water { .. })
        )
    }

    /// ===============================
    /// YAZMA
    /// ===============================
    /// ⚠️ Map mutable ama "kontrollü" değişir
    /// Entity logic buraya gömülmez

    /// Konuma yeni bir şey yerleştir
    ///
    /// Örnek:
    /// - Food eklemek
    /// - Ceset bırakmak
    pub fn set_cell(&mut self, pos: Position, cell: Cell) -> bool {
        if let Some(i) = self.index_of(pos) {
            self.grid[i] = cell;
            true
        } else {
            false
        }
    }

    /// Konumdaki miktarı azalt
    ///
    /// amount kadar düşer,
    /// 0 veya altına inerse Empty olur
    pub fn reduce_cell_amount(&mut self, pos: Position, amount: usize) -> bool {
        if let Some(i) = self.index_of(pos) {
            match &mut self.grid[i] {
                Cell::Food { amount: a } | Cell::Water { amount: a } => {
                    *a = a.saturating_sub(amount);
                    if *a == 0 {
                        self.grid[i] = Cell::Empty;
                    }
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }

    /// Konumdaki şeyi tamamen sil
    pub fn clear_cell(&mut self, pos: Position) -> bool {
        self.set_cell(pos, Cell::Empty)
    }
}

```

## src/map/position.rs
```
use crate::map::direction::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    /// x ve y değerlerinden yeni bir değer oluştur
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
    /// Pozisyonu doğrudan güncellemek için
    pub fn set(&mut self, other: Position) {
        self.x = other.x;
        self.y = other.y;
    }

    /// Manhattan mesafesini hesaplar
    pub fn distance_to(&self, other: Position) -> usize {
        ((self.x as isize - other.x as isize).abs() + (self.y as isize - other.y as isize).abs())
            as usize
    }
}

impl std::ops::Add<Direction> for Position {
    type Output = Position;

    fn add(self, dir: Direction) -> Position {
        match dir {
            Direction::Up => Position {
                x: self.x,
                y: self.y.saturating_sub(1),
            },
            Direction::Down => Position {
                x: self.x,
                y: self.y + 1,
            },
            Direction::Left => Position {
                x: self.x.saturating_sub(1),
                y: self.y,
            },
            Direction::Right => Position {
                x: self.x + 1,
                y: self.y,
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

```

