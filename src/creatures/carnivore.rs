use crate::entity::{
    Entity, intent::Intent, lifestate::LifeState, perception::Perception, species::Species,
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
                max_meal: crate::rules::biology::EAT_ENERGY_AMOUNT,
                reproduction_reserve_per_mille: 500,
                move_energy_cost: 1,
                max_stamina: 24,
                stamina: 24,
                exertion: 0,
                vision_metabolic_cost: 0,
                stamina_metabolic_cost: 0,
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
                reproduction_interval: crate::rules::biology::REPRODUCTION_COOLDOWN_TICKS,
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

    fn make_intent(&self, perception: Perception, rng: &mut crate::random::Random) -> Intent {
        crate::entity::behavior::choose(Species::Carnivore, &self.life_state, perception, rng)
    }

    fn decide(
        &self,
        view: Perception,
        rng: &mut crate::random::Random,
    ) -> crate::entity::decision::Decision {
        crate::entity::behavior::decide(Species::Carnivore, &self.life_state, view, rng)
    }

    fn tick(&mut self) {
        // Artik metabolism sistem tarafindan merkezi olarak yonetiliyor
    }

    fn reproduce(&self) -> Box<dyn Entity> {
        let mut child_life = self.life_state.clone();
        child_life.age = 0;
        child_life.stamina = child_life.max_stamina;
        child_life.exertion = 0;
        child_life.death_cause = None;
        child_life.reproduction_cooldown = 0;
        child_life.moves_used = 0;
        child_life.energy = child_life.max_energy / 2;
        child_life.water = child_life.max_water / 2;
        child_life.health = child_life.max_health / 2;
        Box::new(CarnivoreEntity::new(child_life))
    }
}
