use crate::entity::{lifestate::LifeState, species::Species};

pub struct CarnivoreEntity {
    pub life_state: LifeState,
}

impl CarnivoreEntity {
    pub fn new(life_state: LifeState) -> Self {
        Self { life_state }
    }

    pub fn default() -> Self {
        Self {
            life_state: crate::creatures::create_default_life_state(crate::creatures::CreatureConfig {
                max_stamina: 24,
                max_age: 120,
                max_health: 140,
                max_energy: 90,
                max_water: 70,
                maturity_age: 25,
                vision_range: 7,
                speed: 4,
            }),
        }
    }
}

super::impl_creature_entity!(CarnivoreEntity, Species::Carnivore);
