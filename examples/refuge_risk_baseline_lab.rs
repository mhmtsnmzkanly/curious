//! Fixed 1,000-tick observational capture for the refuge effort/risk baseline.
//!
//! This source is frozen only after root review. It accepts no seed or horizon
//! arguments and never enables a refuge learner or changes an action.
#[path = "support/refuge_risk_baseline.rs"]
mod baseline;

use curious::{
    entity::{
        decision::{DecisionTrace, Reason},
        intent::Intent,
        species::Species,
    },
    map::{cell::Cell, movement::Position},
    scenario::Scenario,
    simulation::SimulationEvent,
    world::World,
};
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    io::{self, BufWriter, Write},
};

const SCENARIO_PATH: &str = "scenarios/mixed-refuge.json";
const SCENARIO_SHA256: &str = "9641a7a60157edcd9b3d379210b26aead5ed57df56ec242dca5c604a74d478f2";
const ENGINEERING_SEED: u64 = 123;
const WORLD_TICKS: usize = 1_000;
const MAX_RAW_BYTES: u64 = 128 * 1024 * 1024;
const MAX_EPISODE_TICKS: u8 = 20;
const MAX_ACCOUNT: u64 = 1_000_000_000_000;
const ID_HOLDOUT_FOLDS: u64 = 5;
const PROTOCOL: &str = "refuge_risk_baseline_v1_engineering";

#[derive(Debug, Clone, Serialize)]
struct SubjectObservation {
    id: usize,
    health: usize,
    max_health: usize,
    energy: usize,
    max_energy: usize,
    stamina: usize,
    max_stamina: usize,
    speed: usize,
    move_energy_cost: usize,
    vision_range: usize,
}

impl SubjectObservation {
    fn from_trace(trace: &DecisionTrace) -> Self {
        Self {
            id: trace.id,
            health: trace.before.health,
            max_health: trace.before.max_health,
            energy: trace.before.energy,
            max_energy: trace.before.max_energy,
            stamina: trace.before.stamina,
            max_stamina: trace.before.max_stamina,
            speed: trace.before.speed,
            move_energy_cost: trace.before.move_energy_cost,
            vision_range: trace.before.vision_range,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct EpisodeStart {
    owner_id: usize,
    threat_id: usize,
    threat_species: Species,
    started_tick: usize,
    start_position: Position,
    body: SubjectObservation,
    decision_reason: Reason,
    selected_intent: Intent,
    selected_route_index: usize,
    routes: Vec<baseline::RouteBaseline>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ClosureKind {
    Arrived,
    ThreatLost,
    Died,
    TimedOut,
    TargetChanged,
    PriorityInterrupted,
    OptionsInvalid,
    ObservationMissing,
    HorizonBoundary,
}

#[derive(Debug, Clone, Serialize)]
struct MovementWitness {
    from: Position,
    to: Position,
    energy_cost: usize,
}

#[derive(Debug, Clone, Serialize)]
struct AttackWitness {
    attacker_id: usize,
    target_id: usize,
    damage: usize,
}

#[derive(Debug, Clone, Serialize)]
struct AttemptTick {
    owner_id: usize,
    threat_id: usize,
    tick: usize,
    position_before_resolution: Position,
    health_before_resolution: usize,
    position_after_resolution: Position,
    health_after_resolution: usize,
    movement_events: Vec<MovementWitness>,
    attack_events: Vec<AttackWitness>,
    cumulative_threat_damage: u64,
    cumulative_other_damage: u64,
    cumulative_unattributed_health_loss: u64,
    cumulative_paid_movement_energy: u64,
    elapsed_ticks: u8,
}

#[derive(Debug, Clone, Serialize)]
struct EpisodeClosure {
    start: EpisodeStart,
    ended_tick: usize,
    final_position: Position,
    final_health: usize,
    closure: ClosureKind,
    completed_arrival: bool,
    attribution_known: bool,
    threat_damage: u64,
    other_damage: u64,
    unattributed_health_loss: u64,
    paid_movement_energy: u64,
    elapsed_ticks: u8,
}

#[derive(Debug, Serialize)]
#[serde(tag = "record_type", rename_all = "snake_case")]
enum Record {
    Header {
        protocol: &'static str,
        scenario: &'static str,
        scenario_sha256: &'static str,
        seed: u64,
        physical_tick_limit: usize,
        raw_byte_limit: u64,
        observation_full_all_set_at_tick_zero: bool,
        refuge_feature_flags_changed: bool,
        evidence_role: &'static str,
        damage_baseline_status: &'static str,
    },
    Tick {
        tick: usize,
        active_herbivores: usize,
        seek_shelter_decisions: usize,
        active_attempts: usize,
        resolver_events: usize,
    },
    ResolverEvents {
        tick: usize,
        events: Vec<ResolverEventWitness>,
    },
    StateCheckpoint {
        when: &'static str,
        tick: usize,
        rng_state: u64,
        state: Value,
    },
    EpisodeStart(EpisodeStart),
    EpisodeTick(AttemptTick),
    EpisodeClosure(EpisodeClosure),
    Summary {
        completed_world_ticks: usize,
        refuge_decisions: u64,
        started_attempts: u64,
        closed_attempts: u64,
        completed_arrivals: u64,
        attribution_complete_arrivals: u64,
        censored_attempts: u64,
        ambiguous_or_unmatched_decisions: u64,
        completed_attributed_damage_samples: u64,
        id_grouped_constant_mean_holdout: Vec<HoldoutFold>,
        effort_time_diagnostics: EffortTimeDiagnostics,
        interpretation: &'static str,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ResolverEventWitness {
    Moved {
        event_index: usize,
        id: usize,
        from: Position,
        to: Position,
        energy_cost: usize,
    },
    Attacked {
        event_index: usize,
        attacker: usize,
        target: usize,
        damage: usize,
    },
}

struct CappedJsonl<W: Write> {
    writer: W,
    written: u64,
    limit: u64,
}

impl<W: Write> CappedJsonl<W> {
    fn new(writer: W, limit: u64) -> Self {
        Self {
            writer,
            written: 0,
            limit,
        }
    }

    fn emit<T: Serialize>(&mut self, record: &T) -> io::Result<()> {
        let mut bytes = serde_json::to_vec(record).map_err(io::Error::other)?;
        bytes.push(b'\n');
        let next = self
            .written
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| io::Error::other("raw byte accounting overflow"))?;
        if next > self.limit {
            return Err(io::Error::other("registered raw JSONL byte cap reached"));
        }
        self.writer.write_all(&bytes)?;
        self.written = next;
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct ActiveAttempt {
    start: EpisodeStart,
    endpoint: Position,
    last_health: usize,
    position_before_resolution: Position,
    health_before_resolution: usize,
    threat_damage: u64,
    other_damage: u64,
    unattributed_health_loss: u64,
    paid_movement_energy: u64,
    elapsed_ticks: u8,
}

#[derive(Debug, Clone)]
struct OutcomeSample {
    owner_id: usize,
    threat_damage: u64,
    paid_movement_energy: u64,
    elapsed_ticks: u8,
    nominal_energy: Option<u64>,
    nominal_ticks: Option<u64>,
    completed_arrival: bool,
    attribution_known: bool,
}

#[derive(Debug, Default)]
struct Observer {
    active: BTreeMap<usize, ActiveAttempt>,
    pending_closures: Vec<EpisodeClosure>,
    outcomes: Vec<OutcomeSample>,
    refuge_decisions: u64,
    started_attempts: u64,
    closed_attempts: u64,
    completed_arrivals: u64,
    attribution_complete_arrivals: u64,
    censored_attempts: u64,
    ambiguous_or_unmatched_decisions: u64,
}

#[derive(Debug, Serialize)]
struct HoldoutFold {
    fold: u64,
    training_agent_count: usize,
    training_episode_count: usize,
    heldout_agent_count: usize,
    heldout_episode_count: usize,
    training_mean_threat_damage: Option<f64>,
    heldout_mean_threat_damage: Option<f64>,
    heldout_mean_absolute_error: Option<f64>,
    heldout_mean_signed_error: Option<f64>,
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct EffortTimeDiagnostics {
    arrived_episodes: usize,
    energy_estimate_exact_count: usize,
    energy_absolute_error_mean: Option<f64>,
    nominal_ticks_met_or_exceeded_count: usize,
    nominal_ticks_comparable_count: usize,
    elapsed_ticks_mean: Option<f64>,
}

impl Observer {
    fn before_decisions<W: Write>(
        &mut self,
        traces: &[DecisionTrace],
        tick: usize,
        writer: &mut CappedJsonl<W>,
    ) -> Result<(), String> {
        for trace in traces {
            if trace.species != Species::Herbivore {
                continue;
            }
            let Some(view) = trace.perception.as_ref() else {
                return Err(format!(
                    "full perception missing for herbivore {} at {tick}",
                    trace.id
                ));
            };
            if trace.decision.reason == Reason::SeekShelter {
                self.refuge_decisions += 1;
            }
            let threat = baseline::select_current_threat(view);
            let routes = baseline::route_baselines(trace.pos, &trace.before, view);
            let route_index = baseline::baseline_choice(&routes);
            let endpoint = route_index.map(|index| routes[index].endpoint);

            // The core reconciles non-attack health loss before deciding
            // whether the prior attempt continues or is queued for closure.
            if let Some(active) = self.active.get_mut(&trace.id) {
                reconcile_health(active, trace.before.health, 0);
                active.position_before_resolution = trace.pos;
                active.health_before_resolution = trace.before.health;
            }

            let active_same = self.active.get(&trace.id).is_some_and(|active| {
                threat.as_ref().is_some_and(|threat| {
                    active.start.threat_id == threat.id
                        && active.start.threat_species == threat.species
                }) && endpoint == Some(active.endpoint)
                    && trace.decision.reason == Reason::SeekShelter
            });

            if self.active.contains_key(&trace.id) && !active_same {
                let closure = if threat.is_none() {
                    ClosureKind::ThreatLost
                } else if trace.decision.reason != Reason::SeekShelter {
                    ClosureKind::PriorityInterrupted
                } else if routes.is_empty() {
                    ClosureKind::OptionsInvalid
                } else {
                    ClosureKind::TargetChanged
                };
                self.close_before_resolution(
                    trace.id,
                    closure,
                    trace.pos,
                    trace.before.health,
                    tick,
                );
            }

            if active_same || trace.decision.reason != Reason::SeekShelter {
                continue;
            }
            let (Some(threat), Some(route_index)) = (threat, route_index) else {
                self.ambiguous_or_unmatched_decisions += 1;
                continue;
            };
            if routes[route_index].endpoint_kind != Some(baseline::ObservedCellKind::Burrow)
                || !intent_matches_route(
                    &trace.decision.intent,
                    &routes[route_index].path,
                    trace.before.speed,
                )
            {
                self.ambiguous_or_unmatched_decisions += 1;
                return Err(format!(
                    "baseline route does not match ordinary SeekShelter intent for {} at {tick}",
                    trace.id
                ));
            }
            let start = EpisodeStart {
                owner_id: trace.id,
                threat_id: threat.id,
                threat_species: threat.species,
                started_tick: tick,
                start_position: trace.pos,
                body: SubjectObservation::from_trace(trace),
                decision_reason: trace.decision.reason,
                selected_intent: trace.decision.intent.clone(),
                selected_route_index: route_index,
                routes,
            };
            writer
                .emit(&Record::EpisodeStart(start.clone()))
                .map_err(|error| error.to_string())?;
            self.started_attempts += 1;
            self.active.insert(trace.id, ActiveAttempt::new(start));
        }
        Ok(())
    }

    fn close_before_resolution(
        &mut self,
        id: usize,
        closure: ClosureKind,
        position: Position,
        health: usize,
        tick: usize,
    ) {
        if let Some(mut attempt) = self.active.remove(&id) {
            reconcile_health(&mut attempt, health, 0);
            self.pending_closures
                .push(closure_record(attempt, closure, tick, position, health));
        }
    }

    fn after_resolver<W: Write>(
        &mut self,
        events: &[SimulationEvent],
        world: &World,
        tick: usize,
        writer: &mut CappedJsonl<W>,
    ) -> Result<(), String> {
        // `refuge_outcomes::finish` emits decision-time pending closures before
        // processing any active episode's resolver events for this tick.
        for closure in std::mem::take(&mut self.pending_closures) {
            self.record_closure(closure, writer)?;
        }

        let mut completed_this_tick = Vec::new();
        for slot in &world.entities {
            let Some(mut attempt) = self.active.remove(&slot.id) else {
                continue;
            };
            let mut movement_events = Vec::new();
            let mut attack_events = Vec::new();
            let mut all_attack_damage = 0u64;
            for event in events {
                match event {
                    SimulationEvent::EntityMoved {
                        id,
                        energy_cost,
                        from,
                        to,
                    } if *id == slot.id => {
                        let energy = *energy_cost as u64;
                        attempt.paid_movement_energy = attempt
                            .paid_movement_energy
                            .saturating_add(energy)
                            .min(MAX_ACCOUNT);
                        movement_events.push(MovementWitness {
                            from: *from,
                            to: *to,
                            energy_cost: *energy_cost,
                        });
                    }
                    SimulationEvent::EntityAttacked {
                        attacker,
                        target,
                        damage,
                        ..
                    } if *target == slot.id => {
                        all_attack_damage = all_attack_damage.saturating_add(*damage as u64);
                        if *attacker == attempt.start.threat_id {
                            attempt.threat_damage = attempt
                                .threat_damage
                                .saturating_add(*damage as u64)
                                .min(MAX_ACCOUNT);
                        } else {
                            attempt.other_damage = attempt
                                .other_damage
                                .saturating_add(*damage as u64)
                                .min(MAX_ACCOUNT);
                        }
                        attack_events.push(AttackWitness {
                            attacker_id: *attacker,
                            target_id: *target,
                            damage: *damage,
                        });
                    }
                    _ => {}
                }
            }

            let after_health = slot.base.life().health;
            let after_position = slot.pos;
            reconcile_health(&mut attempt, after_health, all_attack_damage);
            attempt.elapsed_ticks = attempt
                .elapsed_ticks
                .saturating_add(1)
                .min(MAX_EPISODE_TICKS);
            writer
                .emit(&Record::EpisodeTick(AttemptTick {
                    owner_id: slot.id,
                    threat_id: attempt.start.threat_id,
                    tick,
                    position_before_resolution: attempt.position_before_resolution,
                    health_before_resolution: attempt.health_before_resolution,
                    position_after_resolution: after_position,
                    health_after_resolution: after_health,
                    movement_events,
                    attack_events,
                    cumulative_threat_damage: attempt.threat_damage,
                    cumulative_other_damage: attempt.other_damage,
                    cumulative_unattributed_health_loss: attempt.unattributed_health_loss,
                    cumulative_paid_movement_energy: attempt.paid_movement_energy,
                    elapsed_ticks: attempt.elapsed_ticks,
                }))
                .map_err(|error| error.to_string())?;

            let closure = closure_after_resolver(
                slot.base.life().is_alive(),
                slot.pos == attempt.endpoint
                    && world.map.cell(attempt.endpoint) == Some(&Cell::Burrow),
                attempt.elapsed_ticks,
            );
            if let Some(closure) = closure {
                completed_this_tick.push(closure_record(
                    attempt,
                    closure,
                    tick,
                    after_position,
                    after_health,
                ));
            } else {
                attempt.position_before_resolution = after_position;
                attempt.health_before_resolution = after_health;
                self.active.insert(slot.id, attempt);
            }
        }
        let missing = self
            .active
            .keys()
            .copied()
            .filter(|id| !world.entities.iter().any(|slot| slot.id == *id))
            .collect::<Vec<_>>();
        for id in missing {
            if let Some(attempt) = self.active.remove(&id) {
                let position = attempt.position_before_resolution;
                let health = attempt.health_before_resolution;
                let closure = closure_record(
                    attempt,
                    ClosureKind::ObservationMissing,
                    tick,
                    position,
                    health,
                );
                self.record_closure(closure, writer)?;
            }
        }
        for closure in completed_this_tick {
            self.record_closure(closure, writer)?;
        }
        Ok(())
    }

    fn record_closure<W: Write>(
        &mut self,
        closure: EpisodeClosure,
        writer: &mut CappedJsonl<W>,
    ) -> Result<(), String> {
        self.closed_attempts += 1;
        if closure.completed_arrival {
            self.completed_arrivals += 1;
        } else {
            self.censored_attempts += 1;
        }
        if closure.completed_arrival && closure.attribution_known {
            self.attribution_complete_arrivals += 1;
        }
        self.outcomes.push(OutcomeSample {
            owner_id: closure.start.owner_id,
            threat_damage: closure.threat_damage,
            paid_movement_energy: closure.paid_movement_energy,
            elapsed_ticks: closure.elapsed_ticks,
            nominal_energy: closure.start.routes[closure.start.selected_route_index]
                .effort
                .nominal_paid_movement_energy,
            nominal_ticks: closure.start.routes[closure.start.selected_route_index]
                .effort
                .nominal_elapsed_ticks,
            completed_arrival: closure.completed_arrival,
            attribution_known: closure.attribution_known,
        });
        writer
            .emit(&Record::EpisodeClosure(closure))
            .map_err(|error| error.to_string())
    }

    fn censor_at_horizon<W: Write>(
        &mut self,
        tick: usize,
        world: &World,
        writer: &mut CappedJsonl<W>,
    ) -> Result<(), String> {
        let ids = self.active.keys().copied().collect::<Vec<_>>();
        for id in ids {
            let Some(slot) = world.entities.iter().find(|slot| slot.id == id) else {
                return Err(format!("missing active owner {id} at horizon"));
            };
            self.close_before_resolution(
                id,
                ClosureKind::HorizonBoundary,
                slot.pos,
                slot.base.life().health,
                tick,
            );
        }
        for closure in std::mem::take(&mut self.pending_closures) {
            self.record_closure(closure, writer)?;
        }
        Ok(())
    }

    fn holdout_folds(&self) -> Vec<HoldoutFold> {
        (0..ID_HOLDOUT_FOLDS)
            .map(|fold| {
                let complete = self
                    .outcomes
                    .iter()
                    .filter(|row| row.completed_arrival && row.attribution_known)
                    .collect::<Vec<_>>();
                let train = complete
                    .iter()
                    .copied()
                    .filter(|row| row.owner_id as u64 % ID_HOLDOUT_FOLDS != fold)
                    .collect::<Vec<_>>();
                let test = complete
                    .iter()
                    .copied()
                    .filter(|row| row.owner_id as u64 % ID_HOLDOUT_FOLDS == fold)
                    .collect::<Vec<_>>();
                let train_ids = train
                    .iter()
                    .map(|row| row.owner_id)
                    .collect::<std::collections::BTreeSet<_>>();
                let test_ids = test
                    .iter()
                    .map(|row| row.owner_id)
                    .collect::<std::collections::BTreeSet<_>>();
                let mean = (!train.is_empty()).then(|| {
                    train
                        .iter()
                        .map(|row| row.threat_damage as f64)
                        .sum::<f64>()
                        / train.len() as f64
                });
                let (mae, signed_error, heldout_mean) = if let Some(prediction) = mean {
                    if test.is_empty() {
                        (None, None, None)
                    } else {
                        let observed_mean =
                            test.iter().map(|row| row.threat_damage as f64).sum::<f64>()
                                / test.len() as f64;
                        let mae = test
                            .iter()
                            .map(|row| (row.threat_damage as f64 - prediction).abs())
                            .sum::<f64>()
                            / test.len() as f64;
                        (
                            Some(mae),
                            Some(prediction - observed_mean),
                            Some(observed_mean),
                        )
                    }
                } else {
                    (None, None, None)
                };
                HoldoutFold {
                    fold,
                    training_agent_count: train_ids.len(),
                    training_episode_count: train.len(),
                    heldout_agent_count: test_ids.len(),
                    heldout_episode_count: test.len(),
                    training_mean_threat_damage: mean,
                    heldout_mean_threat_damage: heldout_mean,
                    heldout_mean_absolute_error: mae,
                    heldout_mean_signed_error: signed_error,
                    status: if mean.is_some() && !test.is_empty() {
                        "descriptive_id_grouped_holdout"
                    } else {
                        "unknown_insufficient_fold_samples"
                    },
                }
            })
            .collect()
    }

    fn effort_time_diagnostics(&self) -> EffortTimeDiagnostics {
        let arrived = self
            .outcomes
            .iter()
            .filter(|row| row.completed_arrival)
            .collect::<Vec<_>>();
        let energy_errors = arrived
            .iter()
            .filter_map(|row| {
                row.nominal_energy
                    .map(|expected| expected.abs_diff(row.paid_movement_energy) as f64)
            })
            .collect::<Vec<_>>();
        let ticks = arrived
            .iter()
            .filter_map(|row| {
                row.nominal_ticks
                    .map(|nominal| (nominal, row.elapsed_ticks as u64))
            })
            .collect::<Vec<_>>();
        EffortTimeDiagnostics {
            arrived_episodes: arrived.len(),
            energy_estimate_exact_count: arrived
                .iter()
                .filter(|row| row.nominal_energy == Some(row.paid_movement_energy))
                .count(),
            energy_absolute_error_mean: mean(&energy_errors),
            nominal_ticks_met_or_exceeded_count: ticks
                .iter()
                .filter(|(nominal, actual)| actual >= nominal)
                .count(),
            nominal_ticks_comparable_count: ticks.len(),
            elapsed_ticks_mean: mean(
                &arrived
                    .iter()
                    .map(|row| row.elapsed_ticks as f64)
                    .collect::<Vec<_>>(),
            ),
        }
    }
}

impl ActiveAttempt {
    fn new(start: EpisodeStart) -> Self {
        let endpoint = start.routes[start.selected_route_index].endpoint;
        let last_health = start.body.health;
        let position_before_resolution = start.start_position;
        Self {
            endpoint,
            last_health,
            position_before_resolution,
            health_before_resolution: last_health,
            start,
            threat_damage: 0,
            other_damage: 0,
            unattributed_health_loss: 0,
            paid_movement_energy: 0,
            elapsed_ticks: 0,
        }
    }
}

fn closure_after_resolver(
    alive: bool,
    arrived_at_burrow: bool,
    elapsed_ticks: u8,
) -> Option<ClosureKind> {
    if !alive {
        Some(ClosureKind::Died)
    } else if arrived_at_burrow {
        // Physical arrival precedes timeout in the core closure ordering.
        Some(ClosureKind::Arrived)
    } else if elapsed_ticks >= MAX_EPISODE_TICKS {
        Some(ClosureKind::TimedOut)
    } else {
        None
    }
}

fn intent_matches_route(
    intent: &Intent,
    full_path: &[curious::map::movement::Direction],
    speed: usize,
) -> bool {
    match intent {
        Intent::Move { steps } => {
            steps.0 == full_path.iter().take(speed).copied().collect::<Vec<_>>()
        }
        _ => false,
    }
}

fn reconcile_health(attempt: &mut ActiveAttempt, health: usize, attack_damage: u64) {
    let loss = attempt.last_health.saturating_sub(health) as u64;
    attempt.unattributed_health_loss = attempt
        .unattributed_health_loss
        .saturating_add(loss.saturating_sub(attack_damage))
        .min(MAX_ACCOUNT);
    attempt.last_health = health;
}

fn closure_record(
    attempt: ActiveAttempt,
    closure: ClosureKind,
    ended_tick: usize,
    final_position: Position,
    final_health: usize,
) -> EpisodeClosure {
    let completed_arrival = matches!(closure, ClosureKind::Arrived);
    let attribution_known = attempt.other_damage == 0 && attempt.unattributed_health_loss == 0;
    EpisodeClosure {
        start: attempt.start,
        ended_tick,
        final_position,
        final_health,
        closure,
        completed_arrival,
        attribution_known,
        threat_damage: attempt.threat_damage,
        other_damage: attempt.other_damage,
        unattributed_health_loss: attempt.unattributed_health_loss,
        paid_movement_energy: attempt.paid_movement_energy,
        elapsed_ticks: attempt.elapsed_ticks,
    }
}

fn mean(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

fn refuge_flags_unchanged(world: &World) -> bool {
    world.entities.iter().all(|slot| {
        !slot.memory.refuge_outcomes_enabled
            && slot.memory.refuge_outcomes.is_empty()
            && !slot.memory.refuge_generalization_enabled
            && slot.memory.refuge_generalized_outcomes.is_empty()
            && slot.memory.refuge_episode.is_none()
            && slot.memory.pending_refuge_closures.is_empty()
    })
}

fn resolver_event_witnesses(events: &[SimulationEvent]) -> Vec<ResolverEventWitness> {
    events
        .iter()
        .enumerate()
        .filter_map(|(event_index, event)| match event {
            SimulationEvent::EntityMoved {
                id,
                energy_cost,
                from,
                to,
            } => Some(ResolverEventWitness::Moved {
                event_index,
                id: *id,
                from: *from,
                to: *to,
                energy_cost: *energy_cost,
            }),
            SimulationEvent::EntityAttacked {
                attacker,
                target,
                damage,
                ..
            } => Some(ResolverEventWitness::Attacked {
                event_index,
                attacker: *attacker,
                target: *target,
                damage: *damage,
            }),
            _ => None,
        })
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let scenario_bytes = fs::read(SCENARIO_PATH)?;
    // The frozen external run manifest verifies the scenario bytes against
    // SCENARIO_SHA256 before launching this no-argument executable. This lab
    // has no home-grown hash implementation or added hash dependency.
    let scenario: Scenario = serde_json::from_slice(&scenario_bytes)?;
    let mut world = scenario.build(ENGINEERING_SEED)?;
    // Storage-only full trace capture is set once before tick 1; it is never an
    // AI input and no refuge learner flag is enabled.
    world.observation.full_all = true;
    if !refuge_flags_unchanged(&world) {
        return Err(
            "scenario starts with refuge outcome/learning state enabled or nonempty".into(),
        );
    }

    let stdout = io::stdout();
    let mut writer = CappedJsonl::new(BufWriter::new(stdout.lock()), MAX_RAW_BYTES);
    writer.emit(&Record::Header {
        protocol: PROTOCOL,
        scenario: SCENARIO_PATH,
        scenario_sha256: SCENARIO_SHA256,
        seed: ENGINEERING_SEED,
        physical_tick_limit: WORLD_TICKS,
        raw_byte_limit: MAX_RAW_BYTES,
        observation_full_all_set_at_tick_zero: true,
        refuge_feature_flags_changed: false,
        evidence_role: "seen-seed single-world observed-outcome measurement; no calibrated damage baseline emitted",
        damage_baseline_status: "unknown; no expected-damage value is emitted",
    })?;

    writer.emit(&Record::StateCheckpoint {
        when: "initial_after_storage_only_observation_setting",
        tick: world.tick_counter,
        rng_state: world.rng.state(),
        state: serde_json::to_value(curious::persistence::State::capture(&world))?,
    })?;

    let mut observer = Observer::default();
    for tick in 1..=WORLD_TICKS {
        let events = world.tick();
        let active_herbivores = world
            .entities
            .iter()
            .filter(|slot| {
                slot.base.species() == Species::Herbivore
                    && slot.phase.is_active()
                    && slot.base.life().is_alive()
            })
            .count();
        let refuge_decisions = world
            .last_decisions
            .iter()
            .filter(|trace| {
                trace.species == Species::Herbivore && trace.decision.reason == Reason::SeekShelter
            })
            .count();
        observer.before_decisions(&world.last_decisions, tick, &mut writer)?;
        observer.after_resolver(&events, &world, tick, &mut writer)?;
        writer.emit(&Record::ResolverEvents {
            tick,
            events: resolver_event_witnesses(&events),
        })?;
        if !refuge_flags_unchanged(&world) {
            return Err(format!("refuge learner state changed unexpectedly at tick {tick}").into());
        }
        writer.emit(&Record::Tick {
            tick,
            active_herbivores,
            seek_shelter_decisions: refuge_decisions,
            active_attempts: observer.active.len(),
            resolver_events: events.len(),
        })?;
    }
    observer.censor_at_horizon(WORLD_TICKS, &world, &mut writer)?;
    writer.emit(&Record::Summary {
        completed_world_ticks: world.tick_counter,
        refuge_decisions: observer.refuge_decisions,
        started_attempts: observer.started_attempts,
        closed_attempts: observer.closed_attempts,
        completed_arrivals: observer.completed_arrivals,
        attribution_complete_arrivals: observer.attribution_complete_arrivals,
        censored_attempts: observer.censored_attempts,
        ambiguous_or_unmatched_decisions: observer.ambiguous_or_unmatched_decisions,
        completed_attributed_damage_samples: observer
            .outcomes
            .iter()
            .filter(|row| row.completed_arrival && row.attribution_known)
            .count() as u64,
        id_grouped_constant_mean_holdout: observer.holdout_folds(),
        effort_time_diagnostics: observer.effort_time_diagnostics(),
        interpretation: "selected-route observed outcomes and descriptive grouped-mean holdout only; no calibrated risk prediction, route counterfactual, action, utility, safety, or formal claim",
    })?;
    writer.emit(&Record::StateCheckpoint {
        when: "final_after_tick_1000_and_observer_horizon_censoring",
        tick: world.tick_counter,
        rng_state: world.rng.state(),
        state: serde_json::to_value(curious::persistence::State::capture(&world))?,
    })?;
    writer.writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use curious::map::movement::{Direction, Steps};

    fn start() -> EpisodeStart {
        EpisodeStart {
            owner_id: 17,
            threat_id: 4,
            threat_species: Species::Carnivore,
            started_tick: 1,
            start_position: Position::new(0, 0),
            body: SubjectObservation {
                id: 17,
                health: 100,
                max_health: 100,
                energy: 80,
                max_energy: 100,
                stamina: 80,
                max_stamina: 100,
                speed: 2,
                move_energy_cost: 1,
                vision_range: 5,
            },
            decision_reason: Reason::SeekShelter,
            selected_intent: Intent::Move {
                steps: Steps(vec![Direction::Right]),
            },
            selected_route_index: 0,
            routes: Vec::new(),
        }
    }

    fn active() -> ActiveAttempt {
        ActiveAttempt {
            start: start(),
            endpoint: Position::new(1, 0),
            last_health: 100,
            position_before_resolution: Position::new(0, 0),
            health_before_resolution: 100,
            threat_damage: 0,
            other_damage: 0,
            unattributed_health_loss: 0,
            paid_movement_energy: 0,
            elapsed_ticks: 0,
        }
    }

    #[test]
    fn closure_order_is_death_then_arrival_then_timeout() {
        assert!(matches!(
            closure_after_resolver(false, true, MAX_EPISODE_TICKS),
            Some(ClosureKind::Died)
        ));
        assert!(matches!(
            closure_after_resolver(true, true, MAX_EPISODE_TICKS),
            Some(ClosureKind::Arrived)
        ));
        assert!(matches!(
            closure_after_resolver(true, false, MAX_EPISODE_TICKS),
            Some(ClosureKind::TimedOut)
        ));
        assert_eq!(closure_after_resolver(true, false, 1), None);
    }

    #[test]
    fn threat_only_resolver_damage_is_attributed_but_other_or_unexplained_loss_censors() {
        let mut only_threat = active();
        reconcile_health(&mut only_threat, 90, 10);
        only_threat.threat_damage = 10;
        let completed = closure_record(
            only_threat,
            ClosureKind::Arrived,
            2,
            Position::new(1, 0),
            90,
        );
        assert!(completed.completed_arrival);
        assert!(completed.attribution_known);

        let mut other_attacker = active();
        reconcile_health(&mut other_attacker, 90, 10);
        other_attacker.other_damage = 10;
        let censored = closure_record(
            other_attacker,
            ClosureKind::Arrived,
            2,
            Position::new(1, 0),
            90,
        );
        assert!(censored.completed_arrival);
        assert!(!censored.attribution_known);

        let mut unexplained = active();
        reconcile_health(&mut unexplained, 90, 0);
        let censored = closure_record(
            unexplained,
            ClosureKind::Arrived,
            2,
            Position::new(1, 0),
            90,
        );
        assert!(!censored.attribution_known);
    }

    #[test]
    fn censored_attempts_do_not_enter_arrival_holdout_and_ids_stay_in_one_fold() {
        let mut observer = Observer::default();
        for (owner_id, damage, completed_arrival) in [
            (1, 4, true),
            (6, 8, true),
            (2, 20_000, false),
            (7, 12, true),
        ] {
            observer.outcomes.push(OutcomeSample {
                owner_id,
                threat_damage: damage,
                paid_movement_energy: 1,
                elapsed_ticks: 1,
                nominal_energy: Some(1),
                nominal_ticks: Some(1),
                completed_arrival,
                attribution_known: true,
            });
        }
        let folds = observer.holdout_folds();
        let fold_one = &folds[1]; // owner_id % 5; owners 1 and 6 remain grouped.
        assert_eq!(fold_one.heldout_agent_count, 2);
        assert_eq!(fold_one.heldout_episode_count, 2);
        assert_eq!(fold_one.training_episode_count, 1);
        assert_eq!(fold_one.training_mean_threat_damage, Some(12.0));
        assert_eq!(fold_one.heldout_mean_threat_damage, Some(6.0));
        assert_eq!(fold_one.heldout_mean_absolute_error, Some(6.0));
    }
}
