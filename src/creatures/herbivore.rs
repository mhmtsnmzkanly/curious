use crate::{
    entity::{
        Entity,
        intent::Intent,
        lifestate::LifeState,
        perception::Perception,
        species::Species,
    },
};

pub struct HerbivoreEntity {
    pub life_state: LifeState,
}

impl HerbivoreEntity {
    pub fn new(life_state: LifeState) -> Self {
        Self { life_state }
    }

    pub fn default() -> Self {
        Self {
            life_state: LifeState {
                death_cause: None,
                max_age: 105,
                max_health: 120,
                max_energy: 80,
                max_water: 60,
                maturity_age: 20,
                vision_range: 6,
                age: 0,
                health: 120,
                energy: 80,
                water: 60,
                reproduction_cooldown: 0,
                speed: 3,
                moves_used: 0,
            },
        }
    }
}

impl Entity for HerbivoreEntity {
    fn life(&self) -> &LifeState {
        &self.life_state
    }

    fn life_mut(&mut self) -> &mut LifeState {
        &mut self.life_state
    }

    fn species(&self) -> Species {
        Species::Herbivore
    }

    fn make_intent(&self, perception: Perception, rng:&mut crate::random::Random) -> Intent {
        crate::entity::behavior::choose(Species::Herbivore,&self.life_state,perception,rng)
    }

    fn tick(&mut self) {
        // Artik metabolism sistem tarafindan merkezi olarak yonetiliyor
    }

    fn reproduce(&self) -> Box<dyn Entity> {
        let mut child_life = self.life_state.clone();
        child_life.age = 0;
        child_life.death_cause = None;
        child_life.reproduction_cooldown = 0;
        child_life.moves_used = 0;
        child_life.energy = child_life.max_energy / 2;
        child_life.water = child_life.max_water / 2;
        child_life.health = child_life.max_health / 2;
        Box::new(HerbivoreEntity::new(child_life))
    }
}
