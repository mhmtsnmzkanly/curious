//! Resolve one individual's observed shelter attempt against physical outcomes.
use crate::{
    entity::memory::{
        MAX_REFUGE_EPISODES_PER_KEY, MAX_REFUGE_OUTCOMES, RefugeClosure, RefugeEpisode,
        RefugeOutcome,
    },
    map::{Map, cell::Cell, movement::Position},
    simulation::SimulationEvent,
    world::EntitySlot,
};

const MAX_ACCOUNT: u64 = 1_000_000_000_000;

fn record_episode(
    slot: &mut EntitySlot,
    episode: &RefugeEpisode,
    closure: RefugeClosure,
    tick: usize,
    final_pos: Position,
) -> SimulationEvent {
    let arrived = closure == RefugeClosure::Arrived;
    // Only resolver-verified arrival is a completed route. Losing sight of a
    // threat, an interruption, or an invalidated route censors the attempt.
    let completed = closure == RefugeClosure::Arrived;
    let attribution_known = episode.other_damage == 0 && episode.unattributed_health_loss == 0;
    let safe_arrival = completed && arrived && attribution_known && episode.threat_damage == 0;
    if completed && attribution_known {
        let memory = &mut slot.memory;
        let index = memory
            .refuge_outcomes
            .iter()
            .position(|row| row.key == episode.key);
        let index = match index {
            Some(index) => index,
            None if memory.refuge_outcomes.len() < MAX_REFUGE_OUTCOMES => {
                memory.refuge_outcomes.push(RefugeOutcome {
                    key: episode.key.clone(),
                    last_observed_tick: tick,
                    completed: 0,
                    safe_arrivals: 0,
                    threat_damage: 0,
                    movement_energy: 0,
                });
                memory.refuge_outcomes.len() - 1
            }
            None => {
                let oldest = memory
                    .refuge_outcomes
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, row)| {
                        (
                            row.last_observed_tick,
                            row.key.threat_id,
                            species_rank(&row.key.threat_species),
                            row.key.bearing,
                            row.key.threat_distance,
                            row.key.energy_quartile,
                            row.key.stamina_quartile,
                            row.key.endpoint.y,
                            row.key.endpoint.x,
                        )
                    })
                    .map(|(index, _)| index)
                    .unwrap();
                memory.refuge_outcomes.remove(oldest);
                memory.refuge_outcomes.push(RefugeOutcome {
                    key: episode.key.clone(),
                    last_observed_tick: tick,
                    completed: 0,
                    safe_arrivals: 0,
                    threat_damage: 0,
                    movement_energy: 0,
                });
                memory.refuge_outcomes.len() - 1
            }
        };
        let row = &mut memory.refuge_outcomes[index];
        row.last_observed_tick = tick;
        // Freeze all counters together at the preregistered cap. This keeps
        // rates and means on the same denominator.
        if row.completed < MAX_REFUGE_EPISODES_PER_KEY {
            row.completed += 1;
            row.safe_arrivals += u32::from(safe_arrival);
            row.threat_damage = row
                .threat_damage
                .saturating_add(episode.threat_damage)
                .min(MAX_ACCOUNT);
            row.movement_energy = row
                .movement_energy
                .saturating_add(episode.movement_energy)
                .min(MAX_ACCOUNT);
        }
    }
    SimulationEvent::RefugeEpisodeEnded {
        id: slot.id,
        threat_id: episode.threat_id,
        threat_species: episode.threat_species.clone(),
        context_key: episode.key.clone(),
        started_tick: episode.started_tick,
        ended_tick: tick,
        endpoint: episode.target,
        final_pos,
        closure,
        completed,
        safe_arrival,
        attribution_known,
        threat_damage: episode.threat_damage,
        other_damage: episode.other_damage,
        movement_energy: episode.movement_energy,
        unattributed_health_loss: episode.unattributed_health_loss,
    }
}

fn species_rank(species: &crate::entity::species::Species) -> u8 {
    use crate::entity::species::Species::*;
    match species {
        Herbivore => 0,
        Carnivore => 1,
        Omnivore => 2,
    }
}

pub fn finish(
    entities: &mut [EntitySlot],
    map: &Map,
    events: &[SimulationEvent],
    tick: usize,
) -> Vec<SimulationEvent> {
    let mut closed = Vec::new();
    for slot in entities {
        for pending in std::mem::take(&mut slot.memory.pending_refuge_closures) {
            closed.push(record_episode(
                slot,
                &pending.episode,
                pending.closure,
                pending.ended_tick,
                pending.final_pos,
            ));
        }
        let Some(mut episode) = slot.memory.refuge_episode.take() else {
            continue;
        };
        let health_before_events = episode.threat_damage.saturating_add(episode.other_damage);
        for event in events {
            match event {
                SimulationEvent::EntityMoved {
                    id, energy_cost, ..
                } if *id == slot.id => {
                    episode.movement_energy = episode
                        .movement_energy
                        .saturating_add(*energy_cost as u64)
                        .min(MAX_ACCOUNT);
                }
                SimulationEvent::EntityAttacked {
                    attacker,
                    target,
                    damage,
                    ..
                } if *target == slot.id => {
                    if *attacker == episode.threat_id {
                        episode.threat_damage = episode
                            .threat_damage
                            .saturating_add(*damage as u64)
                            .min(MAX_ACCOUNT);
                    } else {
                        episode.other_damage = episode
                            .other_damage
                            .saturating_add(*damage as u64)
                            .min(MAX_ACCOUNT);
                    }
                }
                _ => {}
            }
        }
        let attack_damage_this_tick = episode
            .threat_damage
            .saturating_add(episode.other_damage)
            .saturating_sub(health_before_events);
        crate::entity::refuge_learning::reconcile_refuge_health(
            &mut episode,
            slot.base.life().health,
            attack_damage_this_tick,
        );
        episode.elapsed_ticks = episode.elapsed_ticks.saturating_add(1).min(20);
        let closure = if !slot.base.life().is_alive() {
            Some(RefugeClosure::Died)
        } else if slot.pos == episode.target && map.cell(episode.target) == Some(&Cell::Burrow) {
            Some(RefugeClosure::Arrived)
        } else if episode.unseen_ticks >= 2 {
            Some(RefugeClosure::ThreatLost)
        } else if episode.elapsed_ticks >= 20 {
            Some(RefugeClosure::TimedOut)
        } else {
            None
        };
        if let Some(closure) = closure {
            closed.push(record_episode(slot, &episode, closure, tick, slot.pos));
        } else {
            slot.memory.refuge_episode = Some(episode);
        }
    }
    closed
}
