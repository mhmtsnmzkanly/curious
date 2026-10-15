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
        let mut intents = Vec::new();
        let mut traces = Vec::new();
        for index in 0..world.entities.len() {
            let slot = &world.entities[index];
            if !slot.phase.is_active() {
                continue;
            }
            let view = PerceptionSystem::build(world, slot);
            let rng = world.rng.state();
            let mut learner = slot.learner.clone();
            let mut memory = slot.memory.clone();
            memory.observe(slot.pos, &view, world.tick_counter, slot.base.species());
            let decision = if let Some(intent) = world.command_inputs.get(&slot.id) {
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
            let decision = if matches!(slot.controller, EntityController::AI)
                && !world.command_inputs.contains_key(&slot.id)
            {
                learner.choose(slot.pos, &view, decision, &mut world.rng)
            } else {
                learner.pending = None;
                decision
            };
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
            world.entities[index].memory = memory;
            world.entities[index].learner = learner;
        }
        world.last_decisions = traces;
        intents
    }
}
