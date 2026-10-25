use crate::entity::{Entity, lifestate::LifeState, species::Species};

/// Computes the initial `LifeState` for newborn offspring.
pub fn child_life_state(parent: &LifeState) -> LifeState {
    let mut child_life = parent.clone();
    child_life.age = 0;
    child_life.stamina = child_life.max_stamina;
    child_life.exertion = 0;
    child_life.death_cause = None;
    child_life.reproduction_cooldown = 0;
    child_life.moves_used = 0;
    child_life.energy = child_life.max_energy / 2;
    child_life.water = child_life.max_water / 2;
    child_life.health = child_life.max_health / 2;
    child_life
}

/// Baseline parameters that vary per creature species.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatureConfig {
    pub max_stamina: usize,
    pub max_age: usize,
    pub max_health: usize,
    pub max_energy: usize,
    pub max_water: usize,
    pub maturity_age: usize,
    pub vision_range: usize,
    pub speed: usize,
}

/// Constructs the default `LifeState` using species-specific parameters
/// and the unified biological rules / constants.
pub fn create_default_life_state(cfg: CreatureConfig) -> LifeState {
    LifeState {
        max_meal: crate::rules::biology::EAT_ENERGY_AMOUNT,
        reproduction_reserve_per_mille: 500,
        move_energy_cost: 1,
        max_stamina: cfg.max_stamina,
        stamina: cfg.max_stamina,
        exertion: 0,
        vision_metabolic_cost: 0,
        stamina_metabolic_cost: 0,
        death_cause: None,
        max_age: cfg.max_age,
        max_health: cfg.max_health,
        max_energy: cfg.max_energy,
        max_water: cfg.max_water,
        maturity_age: cfg.maturity_age,
        vision_range: cfg.vision_range,
        age: 0,
        health: cfg.max_health,
        energy: cfg.max_energy,
        water: cfg.max_water,
        reproduction_cooldown: 0,
        reproduction_interval: crate::rules::biology::REPRODUCTION_COOLDOWN_TICKS,
        speed: cfg.speed,
        moves_used: 0,
    }
}

/// Implements the `Entity` trait and `Default` for creature entity structs,
/// deduplicating boilerplate across species.
macro_rules! impl_creature_entity {
    ($struct_name:ident, $species:expr) => {
        impl $crate::entity::Entity for $struct_name {
            fn life(&self) -> &$crate::entity::lifestate::LifeState {
                &self.life_state
            }

            fn life_mut(&mut self) -> &mut $crate::entity::lifestate::LifeState {
                &mut self.life_state
            }

            fn species(&self) -> $crate::entity::species::Species {
                $species
            }

            fn make_intent(
                &self,
                perception: $crate::entity::perception::Perception,
                rng: &mut $crate::random::Random,
            ) -> $crate::entity::intent::Intent {
                $crate::entity::behavior::choose($species, &self.life_state, perception, rng)
            }

            fn decide(
                &self,
                view: $crate::entity::perception::Perception,
                rng: &mut $crate::random::Random,
            ) -> $crate::entity::decision::Decision {
                $crate::entity::behavior::decide($species, &self.life_state, view, rng)
            }

            fn tick(&mut self) {
                // Central metabolism system handles per-tick biology.
            }

            fn reproduce(&self) -> Box<dyn $crate::entity::Entity> {
                Box::new($struct_name::new($crate::creatures::child_life_state(
                    &self.life_state,
                )))
            }
        }

        impl Default for $struct_name {
            fn default() -> Self {
                Self::default()
            }
        }
    };
}
pub(crate) use impl_creature_entity;

pub mod carnivore;
pub mod herbivore;
pub mod omnivore;

pub use carnivore::CarnivoreEntity;
pub use herbivore::HerbivoreEntity;
pub use omnivore::OmnivoreEntity;

pub fn from_life(species: Species, life: LifeState) -> Box<dyn Entity> {
    let mut entity: Box<dyn Entity> = match species {
        Species::Herbivore => Box::new(HerbivoreEntity::default()),
        Species::Carnivore => Box::new(CarnivoreEntity::default()),
        Species::Omnivore => Box::new(OmnivoreEntity::default()),
    };
    *entity.life_mut() = life;
    entity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn species_defaults_and_trait_contract() {
        let h = HerbivoreEntity::default();
        assert_eq!(h.species(), Species::Herbivore);
        assert_eq!(h.life().health, 120);
        assert_eq!(h.life().speed, 3);
        assert_eq!(h.life().max_stamina, 16);

        let c = CarnivoreEntity::default();
        assert_eq!(c.species(), Species::Carnivore);
        assert_eq!(c.life().health, 140);
        assert_eq!(c.life().speed, 4);
        assert_eq!(c.life().max_stamina, 24);

        let o = OmnivoreEntity::default();
        assert_eq!(o.species(), Species::Omnivore);
        assert_eq!(o.life().health, 130);
        assert_eq!(o.life().speed, 3);
        assert_eq!(o.life().max_stamina, 20);
    }

    #[test]
    fn reproduction_halves_resources_and_resets_lifecycle() {
        let mut parent = HerbivoreEntity::default();
        parent.life_state.age = 50;
        parent.life_state.stamina = 5;
        parent.life_state.exertion = 11;
        parent.life_state.reproduction_cooldown = 10;
        parent.life_state.moves_used = 2;
        parent.life_state.energy = 80;
        parent.life_state.water = 60;
        parent.life_state.health = 120;

        let child = parent.reproduce();
        assert_eq!(child.species(), Species::Herbivore);
        assert_eq!(child.life().age, 0);
        assert_eq!(child.life().stamina, child.life().max_stamina);
        assert_eq!(child.life().exertion, 0);
        assert_eq!(child.life().reproduction_cooldown, 0);
        assert_eq!(child.life().moves_used, 0);
        assert_eq!(child.life().energy, 40);
        assert_eq!(child.life().water, 30);
        assert_eq!(child.life().health, 60);
    }
}
