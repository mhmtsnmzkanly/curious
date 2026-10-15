//! Shared rule-based policy. Reason codes are produced in the same branch as the intent.
use super::{
    decision::{Decision, Reason},
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
    decide(species, life, view, rng).intent
}
pub fn decide(species: Species, life: &LifeState, view: Perception, rng: &mut Random) -> Decision {
    let selected = InstinctEvaluator::evaluate(life, &view, species.clone());
    match selected.instinct {
        Instinct::Threat => {
            if let Some(threat) = selected.threat {
                let combat = species.combat_profile();
                let no_escape = view.directions.values().all(|d| *d == 0);
                if (species != Species::Herbivore && threat.can_win || no_escape)
                    && life.energy >= combat.energy_cost
                    && life.stamina >= combat.stamina_cost
                {
                    Decision::new(
                        Intent::Attack {
                            target_id: threat.target_id,
                        },
                        if no_escape {
                            Reason::CorneredDefense
                        } else {
                            Reason::Defend
                        },
                    )
                } else {
                    Decision::new(
                        Intent::Flee {
                            target_id: threat.target_id,
                        },
                        Reason::EscapeThreat,
                    )
                }
            } else {
                Decision::new(Intent::Idle { duration: 1 }, Reason::Rest)
            }
        }
        Instinct::Thirst => {
            if let Some(w) = view.waters.iter().max_by_key(|w| {
                (
                    w.amount.min(crate::rules::biology::DRINK_WATER_AMOUNT) as isize * 2
                        - w.steps.len() as isize,
                    std::cmp::Reverse(w.steps.len()),
                )
            }) {
                Decision::new(
                    Intent::Drink {
                        at: w.steps.clone(),
                    },
                    Reason::SeekWater,
                )
            } else {
                Decision::new(explore(life, &view, rng), Reason::SearchWater)
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
            if let Some(f) = food {
                return Decision::new(
                    Intent::Eat {
                        at: f.steps.clone(),
                        corpse_id: f.corpse_id,
                    },
                    Reason::SeekFood,
                );
            }
            if let Some(prey) = view
                .entities
                .iter()
                .filter(|e| species.hunts(&e.species))
                .min_by_key(|e| (e.steps.len(), e.power, e.id))
            {
                let combat = species.combat_profile();
                // Do not reserve an arbitrary energy fraction: an adjacent injured
                // prey may still be worth a final affordable attack.
                let minimum_cost = combat
                    .energy_cost
                    .saturating_add(prey.steps.len().saturating_sub(1));
                if life.energy < minimum_cost {
                    return Decision::new(Intent::Idle { duration: 1 }, Reason::HuntLowEnergy);
                }
                if life.stamina <= life.max_stamina / 4 || life.stamina < combat.stamina_cost {
                    return Decision::new(Intent::Idle { duration: 1 }, Reason::RecoverStamina);
                }
                let intent = if prey.steps.len() <= 1 {
                    Intent::Attack { target_id: prey.id }
                } else {
                    Intent::Move {
                        steps: prey.steps.clone(),
                    }
                };
                Decision::new(intent, Reason::HuntPrey)
            } else {
                Decision::new(explore(life, &view, rng), Reason::SearchFood)
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
                    Decision::new(Intent::Mate { target_id: mate.id }, Reason::Mate)
                } else {
                    Decision::new(
                        Intent::Move {
                            steps: Steps(mate.steps.0.iter().take(life.speed).copied().collect()),
                        },
                        Reason::ApproachMate,
                    )
                }
            } else {
                let intent = if rng.range(1, 100) <= 30 {
                    explore(life, &view, rng)
                } else {
                    Intent::Idle { duration: 1 }
                };
                Decision::new(intent, Reason::NoEligibleMate)
            }
        }
        Instinct::Idle => {
            if rng.range(1, 100) <= 30 {
                Decision::new(explore(life, &view, rng), Reason::Explore)
            } else {
                Decision::new(Intent::Idle { duration: 1 }, Reason::Rest)
            }
        }
    }
}
