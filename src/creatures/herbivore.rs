use crate::{
    entity::{
        Entity,
        instinct::{Instinct, InstinctEvaluator},
        intent::Intent,
        lifestate::LifeState,
        perception::Perception,
        species::Species,
    },
    map::movement::{DIRECTION_ARRAY, Steps},
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
                max_age: 105,
                max_health: 120,
                max_energy: 80,
                maturity_age: 20,
                vision_range: 6,
                age: 0,
                health: 120,
                energy: 80,
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

    fn make_intent(&self, perception: Perception) -> Intent {
        let decision = InstinctEvaluator::evaluate(&self.life_state, &perception);

        match decision.instinct {
            Instinct::Threat => {
                if let Some(threat) = decision.threat {
                    if threat.can_win {
                        return Intent::Attack {
                            target_id: threat.target_id,
                        };
                    }
                }
                let mut steps = Steps::empty();
                for _ in 0..self.life_state.speed {
                    steps
                        .0
                        .push(DIRECTION_ARRAY[crate::gen_range(0, 7isize) as usize])
                }
                Intent::Move { steps }
            }
            Instinct::Survival | Instinct::Hunger => {
                if let Some(food) = perception.foods.first() {
                    if !self.life_state.is_energy_full() {
                        return Intent::Eat {
                            at: food.steps.clone(),
                            corpse_id: None,
                        };
                    }
                }

                let mut steps = Steps::empty();
                for _ in 0..self.life_state.speed {
                    steps
                        .0
                        .push(DIRECTION_ARRAY[crate::gen_range(0, 7isize) as usize])
                }
                Intent::Move { steps }
            }
            Instinct::Mating => {
                if let Some(target) = perception
                    .entities
                    .iter()
                    .find(|e| e.species == Species::Herbivore)
                {
                    Intent::Mate {
                        target_id: target.id,
                    }
                } else {
                    Intent::Idle { duration: 1 }
                }
            }
            Instinct::Idle => Intent::Idle { duration: 1 },
        }
    }

    fn tick(&mut self) {
        self.life_state.tick();
    }

    fn reproduce(&self) -> Box<dyn Entity> {
        let mut child_life = self.life_state.clone();
        child_life.age = 0;
        child_life.energy = child_life.max_energy / 2;
        child_life.health = child_life.max_health / 2;
        Box::new(HerbivoreEntity::new(child_life))
    }
}
