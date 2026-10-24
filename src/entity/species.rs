#[derive(
    serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
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

impl Species {
    pub fn combat_profile(&self) -> crate::rules::combat::CombatProfile {
        crate::rules::combat::CombatProfile {
            damage: match self {
                Self::Herbivore => 6,
                Self::Carnivore => 20,
                Self::Omnivore => 14,
            },
            energy_cost: 3,
            stamina_cost: 2,
        }
    }
}
