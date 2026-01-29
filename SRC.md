## src/main.rs
```
use curious::{
    action::Action,
    cell::Cell,
    map::Map,
    position::Position,
    world::{World, WorldView},
};

fn main() {
    println!("Hello, curious!");
}

```

## src/lib.rs
```
// Modülü dahil et
pub mod action;
pub mod cell;
pub mod entity;
pub mod map;
pub mod position;
pub mod world;

// Modülü kullan
//pub use action::Action;
//pub use position::Position;

```

## src/position.rs
```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn offset(&self, dx: i32, dy: i32) -> Option<Self> {
        let nx = self.x as i32 + dx;
        let ny = self.y as i32 + dy;

        if nx < 0 || ny < 0 {
            None
        } else {
            Some(Self {
                x: nx as usize,
                y: ny as usize,
            })
        }
    }
}

```

## src/action.rs
```
pub enum Action {
    Move { dx: i32, dy: i32 },
    Eat,
    Attack { target_id: usize },
    Flee { dx: i32, dy: i32 },
    Idle,
}

```

## src/cell.rs
```
#[derive(Debug, Clone)]
pub enum Cell {
    Empty,
    Food { amount: u32 },
    Water { amount: u32 },
}

```

## src/map.rs
```
use crate::{cell::Cell, position::Position};

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
use crate::{action::Action, entity::Entity, map::Map};

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

## src/entity.rs
```
use crate::{action::Action, position::Position, world::WorldView};

pub trait Entity {
    // Canlı Bilgisi
    fn id(&self) -> usize; // benzersiz kimlik
    fn species(&self) -> Species; //tüketim türü (etçil-otçul-hepçil)
    fn position(&self) -> Position; // bulunduğu konum
    fn life(&self) -> &LifeState; // yaşam durumu
    fn life_mut(&mut self) -> &mut LifeState; // değiştirilebilir yaşam durumu
    fn movement(&self) -> &Movement; // Hareket kabiliyeti
    fn movement_mut(&mut self) -> &mut Movement; // değiştirilebilir Hareket kabiliyeti

    // Canlı
    fn think(&self, ctx: &WorldView) -> Action; // Karar verme mekanizması
    fn apply(&mut self, action: Action); // Kararı işleme koy
}

// Canlının tüketim türü
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Species {
    Herbivore,
    Carnivore,
    Omnivore,
}

// Çiftleşme döngüsü
#[derive(Debug, Clone)]
pub struct ReproductionState {
    pub mature: bool,  // biyolojik olgunluk
    pub cooldown: u32, // son çiftleşmeden sonra bekleme süresi
}

// Canlılın yaşam döngüsü
#[derive(Debug, Clone)]
pub struct LifeState {
    pub age: u32,                        // Canlının yaşı
    pub energy: i32,                     // Enerji (yorgunluk)
    pub alive: bool,                     // Yaşıyor mu?
    pub reproduction: ReproductionState, // Çiftleşebilme kabiliyeti
}

// Canlının hareket kabiliyeti
#[derive(Debug, Clone)]
pub struct Movement {
    pub speed: u32,  // doğal hız (stat)
    pub points: u32, // bu tick biriken hareket puanı
}

```

