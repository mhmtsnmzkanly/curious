//! Attributes water-choice outcomes to the active individual's selected source.
use crate::{
    entity::{decision::DecisionTrace, intent::Intent},
    simulation::{SimulationEvent, event::RejectionReason},
    world::EntitySlot,
};

pub fn finish(
    entities: &mut [EntitySlot],
    decisions: &[DecisionTrace],
    intents: &[(usize, Intent)],
    events: &[SimulationEvent],
) {
    // Include actors that died before intent collection, not only decision traces.
    for slot in entities {
        if slot.learner.episode.is_none() {
            continue;
        }
        let alive = slot.base.life().is_alive();
        if !alive && !decisions.iter().any(|trace| trace.id == slot.id) {
            slot.learner
                .interrupt(crate::entity::learning::WaterEnd::Died);
            continue;
        }
        let missing_resource_target_matches =
            slot.learner.episode.as_ref().is_some_and(|episode| {
                if episode.choice_baseline.is_none() {
                    return false;
                }
                let origin = decisions
                    .iter()
                    .find(|trace| trace.id == slot.id)
                    .map(|t| t.pos);
                let issued_target =
                    intents
                        .iter()
                        .find(|(id, _)| *id == slot.id)
                        .and_then(|(_, intent)| match intent {
                            Intent::Drink { at } => {
                                origin.map(|p| at.0.iter().fold(p, |p, d| p + *d))
                            }
                            _ => None,
                        });
                issued_target == Some(episode.target)
                    && events.iter().any(|event| {
                        matches!(event,
                            SimulationEvent::ActionRejected {
                                id, action, reason: RejectionReason::MissingResource, ..
                            } if *id == slot.id && action == "Drink"
                        )
                    })
            });
        slot.learner
            .finish_with_evidence(slot.id, alive, events, missing_resource_target_matches);
    }
}
