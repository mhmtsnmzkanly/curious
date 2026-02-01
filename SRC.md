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
    pub map: Map,

    /// Tüm Canlıların ID, Pos ve Entity listesi
    pub entities: Vec<EntitySlot>,
}

impl World {
    /// Her "tick", World içinde bir zaman birimidir.
    pub fn tick(&mut self) {}

    /// Entity "Intent" üretebilmesi için "Perception" üretir
    pub fn build_perception(&self, entity: EntitySlot) -> Perception {
        let mut perception = Perception::empty();

        perception
    }

    /// Çakışan "Intent" için çözümleyici
    pub fn resolve_intent() {}

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

```

## src/entity/lifestate.rs
```
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
pub mod species;

use crate::{
    entity::{
        intent::Intent, lifestate::LifeState, perception::*, species::Species,
    },
    map::position::Position,
};

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
    fn reproduce(&self, pos: Position) -> Box<dyn Entity>;
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
#[derive(Debug, Clone)]
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

use crate::map::{cell::Cell, position::Position};

/// Dünyanın harita bilgisini tutar,
/// Cell'den oluşan bir matris tutar
/// Dış dünya ile bilgi alışverişi ve
/// İçerisinde ki veri saklama biçimi farklıdır.
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Cell>,
}
/*
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
*/

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

