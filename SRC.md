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

// Modülü kullan
//pub use action::Action;
//pub use position::Position;

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

    /// Doğal hız (stat)
    pub speed: usize,

    /// Tick boyunca biriken hareket puanı
    pub points: usize,
}

impl LifeState {
    /// Her tick çağrılır
    pub fn tick(&mut self) {
        self.age += 1;

        // Pasif enerji kaybı
        self.energy = self.energy.saturating_sub(1);

        // Üreme bekleme süresi azalır
        if self.reproduction_cooldown > 0 {
            self.reproduction_cooldown -= 1;
        }

        // Yaşlılıktan ölüm
        if self.age >= self.max_age {
            self.health = 0;
        }

        self.points += self.speed;
    }

    // -------- DURUM SORGULARI --------

    /// Canlı yaşıyor mu?
    pub fn is_alive(&self) -> bool {
        self.health > 0
    } // ===============================
    /// YAŞAM DURUMU
    /// ===============================
    //

    /// Üreme olgunluğuna erişti mi?
    pub fn is_mature(&self) -> bool {
        self.age >= self.maturity_age
    }

    /// Enerji kritik seviyede mi?
    pub fn is_energy_low(&self) -> bool {
        self.energy <= self.low_energy_threshold
    }

    /// Enerji tam mı?
    pub fn is_energy_full(&self) -> bool {
        self.energy >= self.max_energy
    }

    /// Çiftleşmeye uygun mu?
    pub fn can_reproduce(&self) -> bool {
        self.is_alive()
            && self.is_mature()
            && self.reproduction_cooldown == 0
            && !self.is_energy_low()
    }

    // -------- DURUM DEĞİŞTİRİCİLER --------

    /// Enerji harcama
    pub fn consume_energy(&mut self, amount: usize) {
        self.energy = self.energy.saturating_sub(amount);
    }

    /// Enerji kazanma
    pub fn restore_energy(&mut self, amount: usize) {
        self.energy = (self.energy + amount).min(self.max_energy);
    }

    /// Can iyileştirme
    pub fn heal(&mut self, amount: usize) {
        self.health = (self.health + amount).min(self.max_health);
    }

    /// Çiftleşme sonrası çağrılır
    pub fn on_reproduce(&mut self) {
        self.reproduction_cooldown = 100;
        self.consume_energy(10);
    }

    /// Yeterli puan var mı?
    pub fn can_move(&self, cost: usize) -> bool {
        self.points >= cost
    }

    /// Hareket puanı harca
    pub fn spend(&mut self, cost: usize) {
        self.points = self.points.saturating_sub(cost);
    }
}

```

## src/entity/mod.rs
```
pub mod lifestate;

use crate::{
    entity::lifestate::LifeState,
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

    /// Konumun değiştirilebilir hali
    fn position_mut(&mut self) -> &mut Position;

    /// Canlının yaşam durumu (genetik + dinamik)
    fn life(&self) -> &LifeState;

    /// Değiştirilebilir yaşam durumu
    fn life_mut(&mut self) -> &mut LifeState;

    /// Karar verme (sadece okuma yapmalı)
    fn think(&self, ctx: &WorldView) -> Action;

    /// Tek tick güncellemesi
    fn tick(&mut self) {
        self.life_mut().tick();
    }

    /// Alınan kuralı uygula
    fn apply(&mut self, action: Action);
}

```

