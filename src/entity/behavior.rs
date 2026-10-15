//! Shared rule-based decision policy. World rules remain outside this module.
use super::{
    instinct::{Instinct, InstinctEvaluator},
    intent::Intent,
    lifestate::LifeState,
    perception::Perception,
    species::Species,
};
use crate::{
    map::movement::{DIRECTION_ARRAY, Steps},
    random::Random,
};

fn explore(life: &LifeState, view: &Perception, rng: &mut Random) -> Intent {
    let available: Vec<_> = DIRECTION_ARRAY
        .into_iter()
        .filter(|d| view.directions.get(d).copied().unwrap_or(0) > 0)
        .collect();
    if available.is_empty() || life.energy == 0 {
        return Intent::Idle { duration: 1 };
    }
    let dir = available[rng.range(0, available.len() as isize - 1) as usize];
    let distance = (view.directions[&dir] as usize)
        .min(life.speed)
        .min(life.energy);
    Intent::Move {
        steps: Steps(vec![dir; distance]),
    }
}

pub fn choose(species: Species, life: &LifeState, view: Perception, rng: &mut Random) -> Intent {
    let decision = InstinctEvaluator::evaluate(life, &view, species.clone());
    match decision.instinct {
        Instinct::Threat => {
            if let Some(threat) = decision.threat {
                if species != Species::Herbivore && threat.can_win {
                    Intent::Attack {
                        target_id: threat.target_id,
                    }
                } else {
                    Intent::Flee {
                        target_id: threat.target_id,
                    }
                }
            } else {
                Intent::Idle { duration: 1 }
            }
        }
        Instinct::Thirst => {
            if let Some(water) = view.waters.iter().max_by_key(|w| {
                (
                    w.amount.min(crate::rules::biology::DRINK_WATER_AMOUNT) as isize * 2
                        - w.steps.len() as isize,
                    std::cmp::Reverse(w.steps.len()),
                )
            }) {
                Intent::Drink {
                    at: water.steps.clone(),
                }
            } else {
                explore(life, &view, rng)
            }
        }
        Instinct::Survival | Instinct::Hunger => {
            let food = view
                .foods
                .iter()
                .filter(|f| {
                    if f.is_corpse {
                        species.eats_meat()
                    } else {
                        species.eats_plants()
                    }
                })
                .max_by_key(|f| {
                    (
                        f.amount.min(crate::rules::biology::EAT_ENERGY_AMOUNT) as isize * 2
                            - f.steps.len() as isize,
                        std::cmp::Reverse(f.steps.len()),
                    )
                });
            if let Some(food) = food {
                return Intent::Eat {
                    at: food.steps.clone(),
                    corpse_id: food.corpse_id,
                };
            }
            if let Some(prey) = view
                .entities
                .iter()
                .filter(|e| species.hunts(&e.species))
                .min_by_key(|e| (e.steps.len(), e.power, e.id))
            {
                if prey.steps.len() <= 1 {
                    Intent::Attack { target_id: prey.id }
                } else {
                    Intent::Move {
                        steps: prey.steps.clone(),
                    }
                }
            } else {
                explore(life, &view, rng)
            }
        }
        Instinct::Mating => {
            if let Some(mate) = view
                .entities
                .iter()
                .filter(|e| e.species == species && e.reproductive)
                .min_by_key(|e| (e.steps.len(), e.id))
            {
                if mate.steps.len() <= 1 {
                    Intent::Mate { target_id: mate.id }
                } else {
                    Intent::Move {
                        steps: Steps(mate.steps.0.iter().take(life.speed).copied().collect()),
                    }
                }
            } else if rng.range(1, 100) <= 30 {
                explore(life, &view, rng)
            } else {
                Intent::Idle { duration: 1 }
            }
        }
        Instinct::Idle => {
            if rng.range(1, 100) <= 30 {
                explore(life, &view, rng)
            } else {
                Intent::Idle { duration: 1 }
            }
        }
    }
}
