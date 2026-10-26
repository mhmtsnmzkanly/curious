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
            let decision = if matches!(slot.controller, EntityController::AI)
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
            traces.push(DecisionTrace::new(
                world.tick_counter,
                slot.id,
                slot.base.species(),
                slot.pos,
                slot.base.life().clone(),
                rng,
                view,
                decision,
                full,
            ));
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
        entity::{phase::EntityPhase, Entity},
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
}
