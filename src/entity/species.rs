#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Species {
    Herbivore,
    Carnivore,
    Omnivore,
}

impl Species {
    pub fn eats_plants(&self) -> bool {
        matches!(self, Self::Herbivore | Self::Omnivore)
    }
    pub fn eats_meat(&self) -> bool {
        matches!(self, Self::Carnivore | Self::Omnivore)
    }
    pub fn hunts(&self, other: &Self) -> bool {
        match self {
            Self::Herbivore => false,
            Self::Carnivore => other != &Self::Carnivore,
            Self::Omnivore => other == &Self::Herbivore,
        }
    }
}
