use crate::entity::{lifestate::LifeState, species::Species};

pub struct OmnivoreEntity {
    pub life_state: LifeState,
}

impl OmnivoreEntity {
    pub fn new(life_state: LifeState) -> Self {
        Self { life_state }
    }

    pub fn default() -> Self {
        Self {
            life_state: crate::creatures::create_default_life_state(crate::creatures::CreatureConfig {
                max_stamina: 20,
                max_age: 110,
                max_health: 130,
                max_energy: 85,
                max_water: 65,
                maturity_age: 22,
                vision_range: 6,
                speed: 3,
            }),
        }
    }
}

super::impl_creature_entity!(OmnivoreEntity, Species::Omnivore);
