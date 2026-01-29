## src/main.rs
```
use curious::{
    map::Map,
    world::{World, WorldView},
    world_types::{Action, Cell, Position},
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
pub mod world_types;

```

## src/map.rs
```
use crate::world_types::{Cell, Position};

pub struct Map {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Cell>,
}

impl Map {
    pub fn index_of(&self, pos: Position) -> Option<usize> {
        if pos.x < self.width && pos.y < self.height {
            Some(pos.y * self.width + pos.x)
        } else {
            None
        }
    }

    pub fn get(&self, pos: Position) -> Option<&Cell> {
        self.index_of(pos).map(|i| &self.grid[i])
    }

    pub fn get_mut(&mut self, pos: Position) -> Option<&mut Cell> {
        self.index_of(pos).map(move |i| &mut self.grid[i])
    }
}

```

## src/world.rs
```
use crate::{entity::Entity, map::Map, world_types::Action};

pub struct WorldView<'a> {
    pub map: &'a Map,
}

pub struct World {
    pub map: Map,
    pub entities: Vec<Box<dyn Entity>>,
}

impl World {
    pub fn tick(&mut self) {
        let view = WorldView { map: &self.map };

        let actions: Vec<(usize, Action)> = self
            .entities
            .iter()
            .map(|e| (e.id(), e.think(&view)))
            .collect();

        for (id, action) in actions {
            if let Some(entity) = self.entities.iter_mut().find(|e| e.id() == id) {
                entity.apply(action);
            }
        }
    }
}

```

## src/world_types.rs
```
#[derive(Debug, Clone)]
pub enum Cell {
    Empty,
    Food { amount: u32 },
    Water { amount: u32 },
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    /// Negatif yön kullanmadan hareket:
    /// Taşma olursa None döner (harita sınırı kontrolü dışarıda yapılır)
    pub fn move_dir(&self, dir: Direction, amount: usize) -> Option<Self> {
        match dir {
            Direction::Up => self.y.checked_sub(amount).map(|y| Self { x: self.x, y }),
            Direction::Down => Some(Self {
                x: self.x,
                y: self.y + amount,
            }),
            Direction::Left => self.x.checked_sub(amount).map(|x| Self { x, y: self.y }),
            Direction::Right => Some(Self {
                x: self.x + amount,
                y: self.y,
            }),
        }
    }
}

pub enum Action {
    Move(Direction),
    Eat,
    Attack { target_id: usize },
    Flee(Direction),
    Idle,
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
pub mod lifestate;
pub mod perception;
pub mod phase;

use crate::{
    entity::{lifestate::LifeState, perception::Perception, phase::EntityPhase},
    world::WorldView,
    world_types::{Action, Position},
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
use crate::world_types::Position;

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

