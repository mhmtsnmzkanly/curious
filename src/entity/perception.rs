use crate::{
    entity::species::Species,
    map::movement::{Direction, Steps},
};
use std::{
    collections::HashMap,
    ops::{Add, AddAssign},
};

/// Algılanan tekil hedef
#[derive(Debug, Clone)]
pub struct PerceivedEntity {
    /// Algılanan canlının kimliği (Kaldırılabilir, Emin değilim)
    pub id: usize,
    /// Algılanan canlının türü
    pub species: Species,
    /// Algılanan canlının güç tahmini
    pub power: usize,
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
    pub fn add_entity(&mut self, id: usize, species: Species, power: usize, steps: Steps) {
        self.entities.push(PerceivedEntity {
            id,
            species,
            power,
            steps,
        });
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
