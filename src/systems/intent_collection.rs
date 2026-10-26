use crate::{
    entity::{
        controller::EntityController,
        decision::{Decision, DecisionTrace, Reason},
        intent::Intent,
    },
    systems::perception::PerceptionSystem,
    world::World,
};
pub struct IntentCollectionSystem;
impl IntentCollectionSystem {
    pub fn collect(world: &mut World, log: &mut Vec<String>) -> Vec<(usize, Intent)> {
        let index = super::spatial::SpatialIndex::new(&world.entities);
        let mut intents = Vec::new();
        let mut traces = Vec::new();
        for slot_index in 0..world.entities.len() {
            let slot = &world.entities[slot_index];
            if !slot.phase.is_active() {
                continue;
            }
            let view = PerceptionSystem::with_index(world, slot, &index);
            let rng = world.rng.state();
            let mut learner = slot.learner.clone();
            let mut hunt_learner = slot.hunt_learner.clone();
            let mut memory = slot.memory.clone();
            let mut water_meal_override = None;
            memory.observe(slot.pos, &view, world.tick_counter, slot.base.species());
            let mut decision = if let Some(intent) = world.command_inputs.get(&slot.id) {
                Decision::new(intent.clone(), Reason::ExternalCommand)
            } else {
                match slot.controller {
                    EntityController::AI => memory.recall(
                        slot.pos,
                        slot.base.life(),
                        &view,
                        slot.base.decide(view.clone(), &mut world.rng),
                    ),
                    EntityController::Player(_) => {
                        Decision::new(Intent::Idle { duration: 1 }, Reason::PlayerWaiting)
                    }
                }
            };
            if memory.enabled
                && memory.spatial_curiosity_enabled
                && matches!(slot.controller, EntityController::AI)
                && !world.command_inputs.contains_key(&slot.id)
                && decision.reason == Reason::Explore
            {
                if let Some(dir) = memory.curiosity_direction(
                    slot.pos,
                    &view,
                    slot.base.life(),
                    world.tick_counter,
                ) {
                    decision = Decision::new(
                        Intent::Move {
                            steps: crate::map::movement::Steps(vec![dir]),
                        },
                        Reason::Explore,
                    );
                }
            }
            let decision = if world.policy.mate_rendezvous
                && slot.base.species() == crate::entity::species::Species::Herbivore
                && memory.enabled
                && memory.companions_enabled
                && matches!(slot.controller, EntityController::AI)
                && !world.command_inputs.contains_key(&slot.id)
            {
                memory.social.decide(
                    slot.pos,
                    slot.base.life(),
                    &view,
                    &memory.companions,
                    &memory.dangers,
                    world.tick_counter,
                    decision,
                )
            } else {
                memory.social = Default::default();
                decision
            };
            let decision = if world.policy.reserve_opportunities
                && matches!(slot.controller, EntityController::AI)
                && !world.command_inputs.contains_key(&slot.id)
            {
                crate::policy::prepare_reserve(
                    slot.base.species(),
                    slot.base.life(),
                    &view,
                    decision,
                )
            } else {
                decision
            };
            let decision = if world.policy.food_search_reserve_enabled
                && matches!(slot.controller, EntityController::AI)
                && !world.command_inputs.contains_key(&slot.id)
            {
                crate::policy::bound_food_search_move(
                    slot.base.life(),
                    decision,
                    world.policy.injury_capacity_penalty,
                )
            } else {
                decision
            };
            let mut decision = if matches!(slot.controller, EntityController::AI)
                && !world.command_inputs.contains_key(&slot.id)
            {
                learner.choose_with_residual_and_context(
                    slot.pos,
                    slot.base.life(),
                    &view,
                    decision,
                    &mut world.rng,
                    world.policy.water_residual_enabled,
                    world.tick_counter,
                    1u16 + slot.genome.mobility as u16,
                )
            } else {
                learner.interrupt(crate::entity::learning::WaterEnd::Interrupted);
                decision
            };
            if world.policy.water_meal_priority_enabled
                && matches!(slot.controller, EntityController::AI)
                && !world.command_inputs.contains_key(&slot.id)
            {
                if let Some(meal) = crate::policy::prefer_adjacent_meal_before_unpayable_water(
                    slot.base.species(),
                    slot.base.life(),
                    &view,
                    decision.clone(),
                    world.policy.injury_capacity_penalty,
                ) {
                    // Close at decision time so this tick's Eat cannot be attributed as
                    // water evidence by the post-resolver outcome pass.
                    learner.interrupt(crate::entity::learning::WaterEnd::Interrupted);
                    let replaced_decision = decision.clone();
                    let visible_water_amount = match &replaced_decision.intent {
                        Intent::Drink { at } => view
                            .waters
                            .iter()
                            .find(|water| water.amount > 0 && water.steps == *at)
                            .map(|water| water.amount)
                            .unwrap_or(0),
                        _ => 0,
                    };
                    let visible_plant_food_amount = match &meal.intent {
                        Intent::Eat {
                            at,
                            corpse_id: None,
                        } => view
                            .foods
                            .iter()
                            .find(|food| {
                                !food.is_corpse && food.corpse_id.is_none() && food.steps == *at
                            })
                            .map(|food| food.amount)
                            .unwrap_or(0),
                        _ => 0,
                    };
                    water_meal_override = Some(crate::entity::decision::WaterMealOverride {
                        replaced_decision,
                        visible_water_amount,
                        visible_plant_food_amount,
                    });
                    decision = meal;
                }
            }
            let decision = hunt_learner.choose(
                slot.base.species(),
                slot.base.life(),
                &view,
                decision,
                &mut world.rng,
            );
            let decision = memory.choose_refuge_for_species(
                slot.base.species(),
                slot.pos,
                slot.base.life(),
                &view,
                decision,
                world.tick_counter,
            );
            let full = world.observation.full_all || world.observation.full_for.contains(&slot.id);
            log.push(format!(
                "[Decision] @{} {:?} => {:?}",
                slot.id, decision.reason, decision.intent
            ));
            intents.push((slot.id, decision.intent.clone()));
            let mut trace = DecisionTrace::new(
                world.tick_counter,
                slot.id,
                slot.base.species(),
                slot.pos,
                slot.base.life().clone(),
                rng,
                view,
                decision,
                full,
            );
            trace.water_meal_override = water_meal_override;
            traces.push(trace);
            world.entities[slot_index].memory = memory;
            world.entities[slot_index].learner = learner;
            world.entities[slot_index].hunt_learner = hunt_learner;
        }
        world.last_decisions = traces;
        intents
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        creatures::herbivore::HerbivoreEntity,
        entity::{
            Entity,
            learning::{WaterEnd, WaterEpisode},
            phase::EntityPhase,
        },
        map::{cell::Cell, movement::Position},
        world::{EntitySlot, World},
    };

    fn world(controller: EntityController) -> World {
        let mut herb = HerbivoreEntity::default();
        herb.life_mut().energy = 5;
        herb.life_mut().speed = 3;
        let slot = EntitySlot::new(
            1,
            Position::new(0, 0),
            EntityPhase::Active,
            controller,
            Box::new(herb),
        );
        let mut world = World::with_seed(-5, 5, -5, 5, vec![slot], 7);
        world.policy.food_search_reserve_enabled = true;
        for y in world.map.min_y..=world.map.max_y {
            for x in world.map.min_x..=world.map.max_x {
                world.map.set_cell(Position::new(x, y), Cell::Empty);
            }
        }
        world
    }

    #[test]
    fn food_search_reserve_applies_to_ai_discretionary_search() {
        let mut world = world(EntityController::AI);
        let intents = IntentCollectionSystem::collect(&mut world, &mut Vec::new());
        assert_eq!(
            world.last_decisions[0].decision.reason,
            Reason::FoodSearchReserve
        );
        let Intent::Move { steps } = &intents[0].1 else {
            panic!("energy budget permits a bounded search move");
        };
        assert_eq!(steps.len(), 2);
    }

    #[test]
    fn food_search_reserve_does_not_override_external_commands_or_player_waiting() {
        let mut commanded = world(EntityController::AI);
        let command = Intent::Move {
            steps: crate::map::movement::Steps(vec![crate::map::movement::Direction::Right; 3]),
        };
        commanded.push_command(1, command.clone());
        let intents = IntentCollectionSystem::collect(&mut commanded, &mut Vec::new());
        assert_eq!(intents[0].1, command);
        assert_eq!(
            commanded.last_decisions[0].decision.reason,
            Reason::ExternalCommand
        );

        let mut player = world(EntityController::Player(3));
        let intents = IntentCollectionSystem::collect(&mut player, &mut Vec::new());
        assert!(matches!(intents[0].1, Intent::Idle { duration: 1 }));
        assert_eq!(
            player.last_decisions[0].decision.reason,
            Reason::PlayerWaiting
        );
    }

    fn thirsty_world(with_episode: bool) -> World {
        let mut world = world(EntityController::AI);
        world.policy.food_search_reserve_enabled = false;
        world.policy.water_meal_priority_enabled = true;
        let slot = &mut world.entities[0];
        slot.base.life_mut().energy = 0;
        slot.base.life_mut().water = 0;
        world
            .map
            .set_cell(Position::new(0, 1), Cell::Food { amount: 3 });
        world
            .map
            .set_cell(Position::new(3, 0), Cell::Water { amount: 5 });
        world
            .map
            .set_cell(Position::new(-3, 0), Cell::Water { amount: 5 });
        if with_episode {
            world.entities[0]
                .learner
                .values
                .push(crate::entity::learning::Value {
                    pos: Position::new(4, 0),
                    value: 12,
                    visits: 3,
                });
            world.entities[0].learner.episode = Some(WaterEpisode {
                target: Position::new(3, 0),
                ticks: 2,
                movement_energy: 2,
                incoming_damage: 0,
                choice_baseline: Some(10),
                outcome_overflowed: false,
                candidate_censored: false,
            });
        }
        world
    }

    #[test]
    fn unaffordable_water_continuation_closes_before_adjacent_plant_eat() {
        let mut world = thirsty_world(true);
        let original_values = serde_json::to_value(&world.entities[0].learner.values).unwrap();
        let intents = IntentCollectionSystem::collect(&mut world, &mut Vec::new());
        assert_eq!(
            world.last_decisions[0].decision.reason,
            Reason::EatBeforeWater
        );
        let override_evidence = world.last_decisions[0]
            .water_meal_override
            .as_ref()
            .unwrap();
        assert!(matches!(
            &override_evidence.replaced_decision.intent,
            Intent::Drink { .. }
        ));
        assert!(override_evidence.visible_water_amount > 0);
        assert!(override_evidence.visible_plant_food_amount > 0);
        assert!(matches!(
            replaced.reason,
            Reason::LearnedWater | Reason::SeekWater | Reason::RecallWater
        ));
        assert!(matches!(intents[0].1, Intent::Eat { .. }));
        assert!(world.entities[0].learner.episode.is_none());
        let closed = world.entities[0].learner.last_episode.as_ref().unwrap();
        assert_eq!(closed.end, WaterEnd::Interrupted);
        assert_eq!(closed.reward, None);
        assert_eq!(
            serde_json::to_value(&world.entities[0].learner.values).unwrap(),
            original_values
        );

        let mut occupied = std::collections::HashMap::from([(Position::new(0, 0), 1)]);
        let mut events = Vec::new();
        crate::systems::interaction::InteractionSystem::resolve_eats(
            &mut world.entities,
            &mut world.map,
            &[crate::world::EatPlan {
                path: Vec::new(),
                id: 1,
                new_pos: Position::new(0, 0),
                cost: 0,
                resource_pos: Position::new(0, 1),
                corpse_id: None,
            }],
            &mut occupied,
            &mut events,
            false,
        );
        assert_eq!(world.entities[0].base.life().energy, 3);
        assert!(events.iter().any(|event| matches!(
            event,
            crate::simulation::SimulationEvent::EntityAte {
                id: 1,
                amount: 3,
                ..
            }
        )));

        // Post-resolver accounting must ignore this tick because no water episode is open.
        crate::systems::water_outcomes::finish(
            &mut world.entities,
            &world.last_decisions,
            &intents,
            &events,
        );
        assert_eq!(
            world.entities[0].learner.last_episode.as_ref().unwrap().end,
            WaterEnd::Interrupted
        );
        assert_eq!(
            serde_json::to_value(&world.entities[0].learner.values).unwrap(),
            original_values
        );
    }

    #[test]
    fn fresh_unpayable_water_selection_can_switch_without_rng_or_movement_charge() {
        let mut with_policy = thirsty_world(false);
        let mut without_policy = thirsty_world(false);
        without_policy.policy.water_meal_priority_enabled = false;
        let mut on_intents = IntentCollectionSystem::collect(&mut with_policy, &mut Vec::new());
        let off_intents = IntentCollectionSystem::collect(&mut without_policy, &mut Vec::new());
        assert_eq!(with_policy.rng.state(), without_policy.rng.state());
        assert_eq!(with_policy.entities[0].base.life().energy, 0);
        assert_eq!(without_policy.entities[0].base.life().energy, 0);
        assert_eq!(
            with_policy.last_decisions[0].decision.reason,
            Reason::EatBeforeWater
        );
        assert!(matches!(on_intents.remove(0).1, Intent::Eat { .. }));
        assert!(matches!(off_intents[0].1, Intent::Drink { .. }));
        assert!(with_policy.entities[0].learner.episode.is_none());
        assert!(without_policy.entities[0].learner.episode.is_some());
    }

    #[test]
    fn paid_first_step_and_adjacent_water_are_never_overridden() {
        let mut partial = thirsty_world(false);
        partial.entities[0].base.life_mut().energy = 1;
        let intents = IntentCollectionSystem::collect(&mut partial, &mut Vec::new());
        assert_ne!(
            partial.last_decisions[0].decision.reason,
            Reason::EatBeforeWater
        );
        assert!(matches!(intents[0].1, Intent::Drink { .. }));

        let mut adjacent = thirsty_world(false);
        adjacent
            .map
            .set_cell(Position::new(1, 0), Cell::Water { amount: 5 });
        let intents = IntentCollectionSystem::collect(&mut adjacent, &mut Vec::new());
        assert_ne!(
            adjacent.last_decisions[0].decision.reason,
            Reason::EatBeforeWater
        );
        assert!(matches!(intents[0].1, Intent::Drink { .. }));
    }

    #[test]
    fn external_command_and_player_are_not_overridden() {
        let mut commanded = thirsty_world(false);
        let command = Intent::Drink {
            at: crate::map::movement::Steps(vec![crate::map::movement::Direction::Right; 3]),
        };
        commanded.push_command(1, command.clone());
        let intents = IntentCollectionSystem::collect(&mut commanded, &mut Vec::new());
        assert_eq!(intents[0].1, command);
        assert_eq!(
            commanded.last_decisions[0].decision.reason,
            Reason::ExternalCommand
        );

        let mut player = thirsty_world(false);
        player.entities[0].controller = EntityController::Player(3);
        let intents = IntentCollectionSystem::collect(&mut player, &mut Vec::new());
        assert!(matches!(intents[0].1, Intent::Idle { duration: 1 }));
    }
}
