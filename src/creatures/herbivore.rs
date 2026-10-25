use crate::entity::{lifestate::LifeState, species::Species};

pub struct HerbivoreEntity {
    pub life_state: LifeState,
}

impl HerbivoreEntity {
    pub fn new(life_state: LifeState) -> Self {
        Self { life_state }
    }

    pub fn default() -> Self {
        Self {
            life_state: crate::creatures::create_default_life_state(crate::creatures::CreatureConfig {
                max_stamina: 16,
                max_age: 105,
                max_health: 120,
                max_energy: 80,
                max_water: 60,
                maturity_age: 20,
                vision_range: 6,
                speed: 3,
            }),
        }
    }
}

super::impl_creature_entity!(HerbivoreEntity, Species::Herbivore);
