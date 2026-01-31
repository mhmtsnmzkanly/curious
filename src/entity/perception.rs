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
