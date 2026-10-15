pub mod carnivore;
pub mod herbivore;
pub mod omnivore;

use crate::entity::{Entity, lifestate::LifeState, species::Species};
pub fn from_life(species: Species, life: LifeState) -> Box<dyn Entity> {
    let mut entity: Box<dyn Entity> = match species {
        Species::Herbivore => Box::new(crate::creatures::herbivore::HerbivoreEntity::default()),
        Species::Carnivore => Box::new(crate::creatures::carnivore::CarnivoreEntity::default()),
        Species::Omnivore => Box::new(crate::creatures::omnivore::OmnivoreEntity::default()),
    };
    *entity.life_mut() = life;
    entity
}
