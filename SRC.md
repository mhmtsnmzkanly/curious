## src/main.rs
```
use curious::{
    entity::action::Action,
    map::{Map, cell::Cell, position::Position},
    world::{World, WorldView},
};

fn main() {
    println!("Hello, curious!");
}

```

## src/lib.rs
```
// Modülü dahil et
pub mod entity;
pub mod map;
pub mod world;

```

## src/world.rs
```
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

    /// Üreme için minimum yaş
    pub maturity_age: usize,

    /// Maksimum can
    pub max_health: usize,

    /// Maksimum enerji
    pub max_energy: usize,

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

    pub fn can_reproduce(&self) -> bool {
        self.is_alive()
            && self.is_mature()
            && self.reproduction_cooldown == 0
            && !self.is_energy_low()
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
        self.energy = (self.energy + amount).min(self.max_energy);
    }

    pub fn heal(&mut self, amount: usize) {
        self.health = (self.health + amount).min(self.max_health);
    }

    pub fn on_reproduce(&mut self) {
        self.reproduction_cooldown = 100;
        self.consume_energy(10);
    }
}

```

## src/entity/mod.rs
```
pub mod action;
pub mod lifestate;
pub mod perception;
pub mod phase;

use crate::{
    entity::{action::Action, lifestate::LifeState, perception::Perception, phase::EntityPhase},
    map::position::Position,
    world::WorldView,
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

    // Algılama
    fn perception(&self) -> &Perception;
    fn perception_mut(&mut self) -> &mut Perception;

    /// Karar verme (sadece okuma yapmalı)
    fn think(&self, ctx: &WorldView) -> Action;

    /// Tek tick güncellemesi
    fn tick(&mut self) {
        // Faz kontrolü
        match self.phase_mut() {
            EntityPhase::Sleeping { remaining } => {
                if *remaining > 0 {
                    *remaining -= 1;
                    return;
                } else {
                    *self.phase_mut() = EntityPhase::Active;
                }
            }
            EntityPhase::Corpse | EntityPhase::Removed => {
                return;
            }
            _ => {}
        }

        // Yaşam güncellemesi
        self.life_mut().tick();

        // Ölüm kontrolü
        if !self.life().is_alive() {
            *self.phase_mut() = EntityPhase::Corpse;
        }
    }

    /// Alınan kuralı uygula
    fn apply(&mut self, action: Action);
}

```

## src/entity/perception.rs
```
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

## src/entity/action.rs
```
use crate::map::direction::Direction;

pub enum Action {
    Move(Direction),
    Eat,
    Attack { target_id: usize },
    Flee(Direction),
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
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

```

