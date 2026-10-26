//! Explicit experimental policy options, separate from physical eligibility.
use crate::entity::{
    decision::{Decision, Reason},
    intent::Intent,
    lifestate::LifeState,
    perception::Perception,
    species::Species,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySettings {
    /// Unqualified herbivore-only rendezvous; opt in independently of reserves.
    #[serde(default)]
    pub mate_rendezvous: bool,
    /// Not ecologically qualified: always opt in, including after legacy restore.
    #[serde(default)]
    pub reserve_opportunities: bool,
    /// Bound discretionary food-search movement by current energy and next-tick body maintenance.
    #[serde(default, skip_serializing_if = "is_false")]
    pub food_search_reserve_enabled: bool,
    /// Prefer an immediately executable plant meal when a water approach cannot pay its first step.
    #[serde(default, skip_serializing_if = "is_false")]
    pub water_meal_priority_enabled: bool,
    /// Experimental current-foraging baseline plus own-source residual choice.
    #[serde(default, skip_serializing_if = "is_false")]
    pub water_residual_enabled: bool,
    /// Experimental K2 physical injury capacity model.
    #[serde(default, skip_serializing_if = "is_false")]
    pub injury_capacity_penalty: bool,
    /// Experimental K2/K6 juvenile parental care feeding and guarding.
    #[serde(default, skip_serializing_if = "is_false")]
    pub juvenile_care_enabled: bool,
    /// Experimental K2/K5 ambush predation stealth and first-strike bonus.
    #[serde(default, skip_serializing_if = "is_false")]
    pub ambush_predation_enabled: bool,
    /// Experimental K4 local social alarm call and coordination.
    #[serde(default, skip_serializing_if = "is_false")]
    pub local_coordination_enabled: bool,
}

/// Switch a narrow, energy-blocked water approach to an already actionable plant meal.
/// `life.energy` is already post-metabolism for this decision. All candidates come from
/// current perception; the resolver still owns movement and eating.
pub fn prefer_adjacent_meal_before_unpayable_water(
    species: Species,
    life: &LifeState,
    view: &Perception,
    decision: Decision,
    injury_penalty_enabled: bool,
) -> Option<Decision> {
    if !matches!(
        decision.reason,
        Reason::SeekWater | Reason::LearnedWater | Reason::RecallWater
    ) {
        return None;
    }
    let Intent::Drink { at } = &decision.intent else {
        return None;
    };
    view.waters
        .iter()
        .any(|w| w.amount > 0 && w.steps == *at)
        .then_some(())?;
    let approach_len = at.len().saturating_sub(1);
    if approach_len == 0
        || life.move_energy_cost == 0
        || life.energy >= life.move_energy_cost
        || life.moves_used >= life.effective_speed(injury_penalty_enabled)
        || life.stamina < life.effective_move_stamina_cost(1, injury_penalty_enabled)
    {
        return None;
    }
    let food = view
        .foods
        .iter()
        .filter(|food| {
            !food.is_corpse
                && food.corpse_id.is_none()
                && species.eats_plants()
                && food.amount > 0
                && food.steps.len() <= 1
                && food
                    .amount
                    .min(life.max_meal)
                    .min(life.max_energy.saturating_sub(life.energy))
                    > 0
        })
        .max_by_key(|food| {
            (
                crate::entity::foraging::value(food.amount, food.steps.len(), life, false),
                std::cmp::Reverse(food.steps.len()),
            )
        })?;
    Some(Decision::new(
        Intent::Eat {
            at: food.steps.clone(),
            corpse_id: food.corpse_id,
        },
        Reason::EatBeforeWater,
    ))
}

fn is_false(value: &bool) -> bool {
    !value
}

/// Bound an already-selected random food-search move by a simple energy budget.
/// The budget reserves the known next-tick body maintenance and enough energy
/// for an equal number of movement steps after the current move. It does not
/// predict a route, resource, or environmental cost.
pub fn bound_food_search_move(
    life: &LifeState,
    decision: Decision,
    injury_penalty_enabled: bool,
) -> Decision {
    if decision.reason != Reason::SearchFood {
        return decision;
    }
    let Intent::Move { steps } = &decision.intent else {
        return decision;
    };
    if steps.0.is_empty() {
        return decision;
    }

    let maintenance = crate::rules::biology::BMR_ENERGY_COST
        .saturating_add(life.vision_metabolic_cost)
        .saturating_add(life.stamina_metabolic_cost);
    let movement_budget = life.energy.saturating_sub(maintenance);
    let round_trip_cap = if life.move_energy_cost == 0 {
        0
    } else {
        movement_budget / life.move_energy_cost / 2
    };
    let speed_cap = life
        .effective_speed(injury_penalty_enabled)
        .saturating_sub(life.moves_used);
    let cap = steps.0.len().min(round_trip_cap).min(speed_cap);

    if cap == steps.0.len() {
        return decision;
    }
    if cap == 0 {
        return Decision::new(Intent::Idle { duration: 1 }, Reason::FoodSearchReserve);
    }
    let mut bounded = steps.clone();
    bounded.0.truncate(cap);
    Decision::new(Intent::Move { steps: bounded }, Reason::FoodSearchReserve)
}

/// One immediately executable meal, not an abandoned multi-tick journey.
/// Carnivores can prepare for a visible adult before the pair is adjacent.
/// Never overrides survival, thirst, threats, existing hunger, or custom policies.
pub fn prepare_reserve(
    species: Species,
    life: &LifeState,
    view: &Perception,
    base: Decision,
) -> Decision {
    if !matches!(
        base.reason,
        Reason::Rest | Reason::Explore | Reason::NoEligibleMate
    ) || life.reproduction_blocker()
        != Some(crate::simulation::event::RejectionReason::InsufficientEnergy)
        || life.health != life.max_health
        || life.water < life.max_water * 3 / 4
        // Preparing requires a potential adult partner, not an already ready
        // one: otherwise two under-reserved adults wait for each other forever.
        // A carnivore meal can be brief and sparse: eat in place while an adult
        // is visible, rather than requiring adjacency before preparing to meet.
        // This does not authorize travel to food or chasing an unseen partner.
        || !view
            .entities
            .iter()
            .any(|mate| {
                mate.species == species
                    && mate.mature
                    && (species == Species::Carnivore || mate.steps.len() <= 1)
            })
    {
        return base;
    }
    // Leave one unit for next tick's basal cost. Do not count hoped-for future meals.
    let target = life.reproduction_energy_required().saturating_add(1);
    if target > life.max_energy || life.energy >= target {
        return base;
    }
    let needed = target - life.energy;
    let food = view
        .foods
        .iter()
        .filter(|food| {
            food.steps.len() <= 1
                && (if food.is_corpse {
                    species.eats_meat()
                } else {
                    species.eats_plants()
                })
                && food
                    .amount
                    .min(life.max_meal)
                    .min(life.max_energy - life.energy)
                    >= needed
        })
        .max_by_key(|food| food.amount.min(life.max_meal));
    match food {
        Some(food) => Decision::new(
            Intent::Eat {
                at: food.steps.clone(),
                corpse_id: food.corpse_id,
            },
            Reason::PrepareReproduction,
        ),
        None => base,
    }
}

#[cfg(test)]
mod food_search_reserve_tests {
    use super::*;
    use crate::{
        creatures::herbivore::HerbivoreEntity,
        entity::Entity,
        map::movement::{Direction, Steps},
    };

    fn life() -> LifeState {
        HerbivoreEntity::default().life().clone()
    }

    fn search(steps: usize) -> Decision {
        Decision::new(
            Intent::Move {
                steps: Steps(vec![Direction::DownRight; steps]),
            },
            Reason::SearchFood,
        )
    }

    #[test]
    fn food_search_reserve_caps_move_after_next_known_maintenance() {
        let mut life = life();
        life.energy = 5;
        life.move_energy_cost = 1;
        // BMR 1 leaves an energy budget of 4; half is reserved for return movement.
        let decision = bound_food_search_move(&life, search(3), false);
        assert_eq!(decision.reason, Reason::FoodSearchReserve);
        let Intent::Move { steps } = decision.intent else {
            panic!("positive reserve should retain a bounded move");
        };
        assert_eq!(steps.len(), 2);

        life.energy = 9;
        life.move_energy_cost = 2;
        let decision = bound_food_search_move(&life, search(3), false);
        let Intent::Move { steps } = decision.intent else {
            panic!("positive reserve should retain a bounded move");
        };
        assert_eq!(steps.len(), 2);
    }

    #[test]
    fn food_search_reserve_uses_trait_costs_speed_and_overflow_safe_subtraction() {
        let mut life = life();
        life.energy = 8;
        life.vision_metabolic_cost = 1;
        life.stamina_metabolic_cost = 1;
        life.speed = 2;
        life.moves_used = 1;
        // Known maintenance is 3, energy permits 2 round-trip steps, speed permits 1.
        let decision = bound_food_search_move(&life, search(4), false);
        let Intent::Move { steps } = decision.intent else {
            panic!("one remaining speed slot should permit one step");
        };
        assert_eq!(steps.len(), 1);

        life.energy = 0;
        let stopped = bound_food_search_move(&life, search(1), false);
        assert_eq!(stopped.reason, Reason::FoodSearchReserve);
        assert!(matches!(stopped.intent, Intent::Idle { duration: 1 }));
    }

    #[test]
    fn food_search_reserve_obeys_injury_speed_and_preserves_other_reasons() {
        let mut life = life();
        life.energy = 20;
        life.health = life.max_health / 2;
        life.speed = 4;
        let injured = bound_food_search_move(&life, search(4), true);
        let Intent::Move { steps } = injured.intent else {
            panic!("injury-adjusted speed should retain a move");
        };
        assert_eq!(steps.len(), 2);

        let water = Decision::new(
            Intent::Move {
                steps: Steps(vec![Direction::Up; 3]),
            },
            Reason::SearchWater,
        );
        let protected = bound_food_search_move(&life, water, true);
        assert_eq!(protected.reason, Reason::SearchWater);
        let Intent::Move { steps } = protected.intent else {
            panic!("non-food search decision must be preserved");
        };
        assert_eq!(steps.len(), 3);
    }

    #[test]
    fn food_search_reserve_does_not_change_affordable_move_or_non_move_search() {
        let mut life = life();
        life.energy = 80;
        let unchanged = bound_food_search_move(&life, search(3), false);
        assert_eq!(unchanged.reason, Reason::SearchFood);
        let resting = Decision::new(Intent::Idle { duration: 1 }, Reason::SearchFood);
        let unchanged = bound_food_search_move(&life, resting, false);
        assert_eq!(unchanged.reason, Reason::SearchFood);
    }

    #[test]
    fn food_search_reserve_preserves_urgent_and_non_food_decisions() {
        let life = life();
        let protected = [
            Reason::RecoverStamina,
            Reason::HuntLowEnergy,
            Reason::HuntTooCostly,
            Reason::EscapeThreat,
            Reason::SeekShelter,
            Reason::CorneredDefense,
            Reason::Defend,
            Reason::SeekWater,
            Reason::SearchWater,
            Reason::RecallWater,
            Reason::LearnedWater,
            Reason::RecallFood,
            Reason::RecallMate,
            Reason::SeekCompanion,
            Reason::WaitForMate,
            Reason::PauseMateSearch,
            Reason::PrepareReproduction,
            Reason::Mate,
            Reason::ApproachMate,
            Reason::SeekFood,
            Reason::HuntPrey,
            Reason::LearnedHunt,
            Reason::NoEligibleMate,
            Reason::Explore,
            Reason::Rest,
            Reason::ExternalCommand,
            Reason::PlayerWaiting,
            Reason::CustomPolicy,
        ];
        for reason in protected {
            let decision = Decision::new(
                Intent::Move {
                    steps: Steps(vec![Direction::DownRight; 3]),
                },
                reason,
            );
            let result = bound_food_search_move(&life, decision, false);
            assert_eq!(result.reason, reason);
            let Intent::Move { steps } = result.intent else {
                panic!("protected decision must retain its intent");
            };
            assert_eq!(steps.len(), 3);
        }
    }

    #[test]
    fn food_search_reserve_setting_is_default_off_and_omitted_when_false() {
        let settings = PolicySettings::default();
        assert!(!settings.food_search_reserve_enabled);
        let json = serde_json::to_value(settings).unwrap();
        assert!(json.get("food_search_reserve_enabled").is_none());
        let decoded: PolicySettings = serde_json::from_value(serde_json::json!({})).unwrap();
        assert!(!decoded.food_search_reserve_enabled);
    }

    #[test]
    fn water_meal_priority_is_default_off_and_omitted_when_false() {
        let settings = PolicySettings::default();
        assert!(!settings.water_meal_priority_enabled);
        let json = serde_json::to_value(settings).unwrap();
        assert!(json.get("water_meal_priority_enabled").is_none());
        let decoded: PolicySettings = serde_json::from_value(serde_json::json!({})).unwrap();
        assert!(!decoded.water_meal_priority_enabled);
    }

    #[test]
    fn water_meal_priority_requires_unpayable_first_step_and_executable_plant_meal() {
        use crate::entity::perception::{PerceivedFood, PerceivedWater};

        let mut life = life();
        life.energy = 0;
        life.water = 0;
        let drink = Decision::new(
            Intent::Drink {
                at: Steps(vec![Direction::Right; 3]),
            },
            Reason::LearnedWater,
        );
        let mut view = Perception::empty();
        view.waters.push(PerceivedWater {
            amount: 2,
            steps: Steps(vec![Direction::Right; 3]),
        });
        view.foods.push(PerceivedFood {
            corpse_id: None,
            amount: 3,
            is_corpse: false,
            steps: Steps(vec![Direction::Up]),
        });
        let switched = prefer_adjacent_meal_before_unpayable_water(
            Species::Herbivore,
            &life,
            &view,
            drink.clone(),
            false,
        )
        .unwrap();
        assert_eq!(switched.reason, Reason::EatBeforeWater);
        assert!(matches!(switched.intent, Intent::Eat { .. }));

        // One payable movement step is useful progress on a longer multi-tick route.
        life.energy = life.move_energy_cost;
        assert!(
            prefer_adjacent_meal_before_unpayable_water(
                Species::Herbivore,
                &life,
                &view,
                drink.clone(),
                false,
            )
            .is_none()
        );
        life.energy = 0;

        // An already adjacent drink has no approach movement to fund.
        let adjacent_drink = Decision::new(
            Intent::Drink {
                at: Steps(vec![Direction::Right]),
            },
            Reason::SeekWater,
        );
        view.waters[0].steps = Steps(vec![Direction::Right]);
        assert!(
            prefer_adjacent_meal_before_unpayable_water(
                Species::Herbivore,
                &life,
                &view,
                adjacent_drink,
                false,
            )
            .is_none()
        );
    }

    #[test]
    fn water_meal_priority_rejects_invalid_food_and_nonlearner_drink_reasons() {
        use crate::entity::perception::{PerceivedFood, PerceivedWater};
        let mut life = life();
        life.energy = 0;
        let drink = Decision::new(
            Intent::Drink {
                at: Steps(vec![Direction::Right; 2]),
            },
            Reason::LearnedWater,
        );
        let mut view = Perception::empty();
        view.waters.push(PerceivedWater {
            amount: 1,
            steps: Steps(vec![Direction::Right; 2]),
        });
        for food in [
            PerceivedFood {
                corpse_id: Some(9),
                amount: 5,
                is_corpse: true,
                steps: Steps(vec![Direction::Up]),
            },
            PerceivedFood {
                corpse_id: None,
                amount: 5,
                is_corpse: false,
                steps: Steps(vec![Direction::Up; 2]),
            },
            PerceivedFood {
                corpse_id: None,
                amount: 0,
                is_corpse: false,
                steps: Steps(vec![Direction::Up]),
            },
        ] {
            view.foods = vec![food];
            assert!(
                prefer_adjacent_meal_before_unpayable_water(
                    Species::Herbivore,
                    &life,
                    &view,
                    drink.clone(),
                    false,
                )
                .is_none()
            );
        }
        view.foods.push(PerceivedFood {
            corpse_id: None,
            amount: 5,
            is_corpse: false,
            steps: Steps(vec![Direction::Up]),
        });
        for reason in [Reason::CustomPolicy, Reason::EscapeThreat, Reason::SeekFood] {
            let protected = Decision::new(drink.intent.clone(), reason);
            assert!(
                prefer_adjacent_meal_before_unpayable_water(
                    Species::Herbivore,
                    &life,
                    &view,
                    protected,
                    false,
                )
                .is_none()
            );
        }
        assert!(
            prefer_adjacent_meal_before_unpayable_water(
                Species::Carnivore,
                &life,
                &view,
                drink.clone(),
                false,
            )
            .is_none()
        );

        let mut no_movement = life.clone();
        no_movement.move_energy_cost = 0;
        assert!(
            prefer_adjacent_meal_before_unpayable_water(
                Species::Herbivore,
                &no_movement,
                &view,
                Decision::new(drink.intent.clone(), Reason::LearnedWater),
                false,
            )
            .is_none()
        );
        let mut no_meal_capacity = life.clone();
        no_meal_capacity.max_energy = 0;
        assert!(
            prefer_adjacent_meal_before_unpayable_water(
                Species::Herbivore,
                &no_meal_capacity,
                &view,
                Decision::new(drink.intent.clone(), Reason::LearnedWater),
                false,
            )
            .is_none()
        );
        let mut no_stamina = life.clone();
        no_stamina.stamina = 0;
        assert!(
            prefer_adjacent_meal_before_unpayable_water(
                Species::Herbivore,
                &no_stamina,
                &view,
                Decision::new(drink.intent.clone(), Reason::LearnedWater),
                false,
            )
            .is_none()
        );
        let mut no_speed_left = life.clone();
        no_speed_left.moves_used = no_speed_left.speed;
        assert!(
            prefer_adjacent_meal_before_unpayable_water(
                Species::Herbivore,
                &no_speed_left,
                &view,
                Decision::new(drink.intent.clone(), Reason::LearnedWater),
                false,
            )
            .is_none()
        );
    }
}
