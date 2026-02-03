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

pub struct OmnivoreEntity {
    pub life_state: LifeState,
}

impl OmnivoreEntity {
    pub fn new(life_state: LifeState) -> Self {
        Self { life_state }
    }

    pub fn default() -> Self {
        Self {
            life_state: LifeState {
                max_age: 110,
                max_health: 130,
                max_energy: 85,
                max_water: 65,
                maturity_age: 22,
                vision_range: 6,
                age: 0,
                health: 130,
                energy: 85,
                water: 65,
                reproduction_cooldown: 0,
                speed: 3,
                moves_used: 0,
            },
        }
    }
}

impl Entity for OmnivoreEntity {
    fn life(&self) -> &LifeState {
        &self.life_state
    }

    fn life_mut(&mut self) -> &mut LifeState {
        &mut self.life_state
    }

    fn species(&self) -> Species {
        Species::Omnivore
    }

    fn make_intent(&self, perception: Perception) -> Intent {
        let decision =
            InstinctEvaluator::evaluate(&self.life_state, &perception, Species::Omnivore);

        let best_food = perception
            .foods
            .iter()
            .min_by_key(|f| (f.steps.len(), usize::MAX - f.amount));
        let best_water = perception
            .waters
            .iter()
            .min_by_key(|w| (w.steps.len(), usize::MAX - w.amount));
        let best_prey = perception
            .entities
            .iter()
            .filter(|e| e.species != Species::Omnivore)
            .min_by_key(|e| (e.steps.len(), e.power));

        match decision.instinct {
            Instinct::Threat => {
                if let Some(threat) = decision.threat {
                    if threat.can_win {
                        return Intent::Attack {
                            target_id: threat.target_id,
                        };
                    }
                    return Intent::Flee {
                        target_id: threat.target_id,
                    };
                }
                Intent::Idle { duration: 1 }
            }
            Instinct::Survival | Instinct::Hunger => {
                if let Some(food) = best_food {
                    if !self.life_state.is_energy_full() {
                        return Intent::Eat {
                            at: food.steps.clone(),
                            corpse_id: None,
                        };
                    }
                }

                if let Some(prey) = best_prey {
                    if prey.steps.len() <= 1 {
                        return Intent::Attack { target_id: prey.id };
                    }
                    return Intent::Move {
                        steps: prey.steps.clone(),
                    };
                }

                let mut steps = Steps::empty();
                for _ in 0..self.life_state.speed {
                    steps
                        .0
                        .push(DIRECTION_ARRAY[crate::gen_range(0, 7isize) as usize])
                }
                Intent::Move { steps }
            }
            Instinct::Thirst => {
                if let Some(water) = best_water {
                    if !self.life_state.is_water_full() {
                        return Intent::Drink {
                            at: water.steps.clone(),
                        };
                    }
                }
                Intent::Idle { duration: 1 }
            }
            Instinct::Mating => {
                if let Some(target) = perception
                    .entities
                    .iter()
                    .find(|e| e.species == Species::Omnivore)
                {
                    if target.steps.len() <= 1 {
                        Intent::Mate {
                            target_id: target.id,
                        }
                    } else {
                        // Çiftleşmek için hedefe yaklaş (tek adım)
                        let mut one_step = Steps::empty();
                        if let Some(first) = target.steps.0.first() {
                            one_step.0.push(*first);
                        }
                        Intent::Move { steps: one_step }
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
        child_life.water = child_life.max_water / 2;
        child_life.health = child_life.max_health / 2;
        Box::new(OmnivoreEntity::new(child_life))
    }
}
