use crate::{
    entity::{
        Entity,
        intent::Intent,
        lifestate::LifeState,
        perception::Perception,
        species::Species,
    },
};

pub struct CarnivoreEntity {
    pub life_state: LifeState,
}

impl CarnivoreEntity {
    pub fn new(life_state: LifeState) -> Self {
        Self { life_state }
    }

    pub fn default() -> Self {
        Self {
            life_state: LifeState {
                death_cause: None,
                max_age: 120,
                max_health: 140,
                max_energy: 90,
                max_water: 70,
                maturity_age: 25,
                vision_range: 7,
                age: 0,
                health: 140,
                energy: 90,
                water: 70,
                reproduction_cooldown: 0,
                speed: 4,
                moves_used: 0,
            },
        }
    }
}

impl Entity for CarnivoreEntity {
    fn life(&self) -> &LifeState {
        &self.life_state
    }

    fn life_mut(&mut self) -> &mut LifeState {
        &mut self.life_state
    }

    fn species(&self) -> Species {
        Species::Carnivore
    }

    fn make_intent(&self, perception: Perception, rng:&mut crate::random::Random) -> Intent {
        crate::entity::behavior::choose(Species::Carnivore,&self.life_state,perception,rng)
    }

    fn decide(&self,view:Perception,rng:&mut crate::random::Random)->crate::entity::decision::Decision {
        crate::entity::behavior::decide(Species::Carnivore,&self.life_state,view,rng)
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
        Box::new(CarnivoreEntity::new(child_life))
    }
}
