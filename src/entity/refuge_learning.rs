//! Opt-in, bounded learning from resolver-verified refuge outcomes.
use super::{
    decision::{Decision, Reason},
    intent::Intent,
    lifestate::LifeState,
    memory::Memory,
    perception::Perception,
    species::Species,
};
use crate::map::movement::{Direction, Position, Steps};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

pub const MAX_REFUGE_OUTCOMES: usize = 8;
pub const MAX_REFUGE_EPISODES_PER_KEY: u32 = 1_000_000;
pub(crate) const MAX_REFUGE_ACCOUNT: u64 = 1_000_000_000_000;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RefugeClosure {
    Arrived,
    ThreatLost,
    Died,
    TimedOut,
    TargetChanged,
    PriorityInterrupted,
    OptionsInvalid,
    FeatureDisabled,
    LabBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct RefugeKey {
    pub threat_id: usize,
    pub threat_species: Species,
    /// Direction discriminant in stable compass order: U, UR, R, DR, D, DL, L, UL.
    pub bearing: u8,
    /// 0=one step, 1=two steps, 2=three or more steps.
    pub threat_distance: u8,
    /// Quartiles: min(3, floor(4 * current / max(1, maximum))).
    pub energy_quartile: u8,
    pub stamina_quartile: u8,
    /// Sorted set of currently visible, valid alternatives for exact matching.
    pub refuge_options: Vec<Position>,
    pub endpoint: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RefugeOutcome {
    pub key: RefugeKey,
    pub last_observed_tick: usize,
    pub completed: u32,
    pub safe_arrivals: u32,
    pub threat_damage: u64,
    pub movement_energy: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RefugeEpisode {
    pub key: RefugeKey,
    pub started_tick: usize,
    pub target: Position,
    pub threat_id: usize,
    pub threat_species: Species,
    pub movement_energy: u64,
    pub threat_damage: u64,
    pub other_damage: u64,
    /// Net health loss not explained by resolver attack events.
    #[serde(default)]
    pub unattributed_health_loss: u64,
    /// Health at the last resolver boundary for separating later physiology.
    #[serde(default)]
    pub last_health: usize,
    pub elapsed_ticks: u8,
    pub unseen_ticks: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PendingRefugeClosure {
    pub episode: RefugeEpisode,
    pub closure: RefugeClosure,
    pub ended_tick: usize,
    pub final_pos: Position,
}

pub(crate) fn choose_refuge(
    memory: &mut Memory,
    origin: Position,
    life: &LifeState,
    view: &Perception,
    decision: Decision,
    tick: usize,
) -> Decision {
    if !memory.enabled {
        return decision;
    }
    if !memory.refuge_outcomes_enabled {
        close(
            memory,
            RefugeClosure::FeatureDisabled,
            origin,
            life.health,
            tick,
        );
        return decision;
    }
    if let Some(episode) = &mut memory.refuge_episode {
        reconcile_refuge_health(episode, life.health, 0);
    }
    let Some(threat) = view.entities.iter().find(|entity| {
        entity.species.hunts(&Species::Herbivore)
            && entity.steps.len() <= 2
            && !entity.steps.0.is_empty()
    }) else {
        close(memory, RefugeClosure::ThreatLost, origin, life.health, tick);
        return decision;
    };
    if decision.reason != Reason::SeekShelter {
        close(
            memory,
            RefugeClosure::PriorityInterrupted,
            origin,
            life.health,
            tick,
        );
        return decision;
    }
    let candidates = refuge_candidates(origin, life, view, threat);
    if candidates.is_empty() {
        close(
            memory,
            RefugeClosure::OptionsInvalid,
            origin,
            life.health,
            tick,
        );
        return decision;
    }

    let any_observed = candidates.iter().any(|candidate| {
        memory
            .refuge_outcomes
            .iter()
            .any(|record| record.key == candidate.key && record.completed > 0)
    });
    let selected = if any_observed {
        let mut ranked = candidates.iter().collect::<Vec<_>>();
        ranked.sort_by(|a, b| compare_refuge_candidate(a, b, &memory.refuge_outcomes));
        ranked[0]
    } else {
        candidates
            .iter()
            .min_by_key(|candidate| (candidate.steps.0.len(), candidate.order))
            .unwrap()
    };

    let target = selected.endpoint;
    let same_episode = memory.refuge_episode.as_ref().is_some_and(|episode| {
        episode.target == target
            && episode.threat_id == threat.id
            && episode.threat_species == threat.species
    });
    if !same_episode {
        close(
            memory,
            RefugeClosure::TargetChanged,
            origin,
            life.health,
            tick,
        );
        memory.refuge_episode = Some(RefugeEpisode {
            key: selected.key.clone(),
            started_tick: tick,
            target,
            threat_id: threat.id,
            threat_species: threat.species.clone(),
            movement_energy: 0,
            threat_damage: 0,
            other_damage: 0,
            unattributed_health_loss: 0,
            last_health: life.health,
            elapsed_ticks: 0,
            unseen_ticks: 0,
        });
    }
    Decision::new(
        Intent::Move {
            steps: Steps(selected.steps.0.iter().take(life.speed).copied().collect()),
        },
        Reason::SeekShelter,
    )
}

pub(crate) fn censor_at_boundary(
    memory: &mut Memory,
    origin: Position,
    health: usize,
    tick: usize,
) {
    close(memory, RefugeClosure::LabBoundary, origin, health, tick);
}

fn close(
    memory: &mut Memory,
    closure: RefugeClosure,
    origin: Position,
    health: usize,
    tick: usize,
) {
    if let Some(mut episode) = memory.refuge_episode.take() {
        reconcile_refuge_health(&mut episode, health, 0);
        memory.pending_refuge_closures.push(PendingRefugeClosure {
            episode,
            closure,
            ended_tick: tick,
            final_pos: origin,
        });
    }
}

pub(crate) fn reconcile_refuge_health(
    episode: &mut RefugeEpisode,
    health: usize,
    observed_attack_damage: u64,
) {
    let health_loss = episode.last_health.saturating_sub(health) as u64;
    episode.unattributed_health_loss = episode
        .unattributed_health_loss
        .saturating_add(health_loss.saturating_sub(observed_attack_damage))
        .min(MAX_REFUGE_ACCOUNT);
    episode.last_health = health;
}

#[derive(Debug)]
struct RefugeCandidate {
    steps: Steps,
    endpoint: Position,
    order: usize,
    key: RefugeKey,
}

fn bearing_bucket(direction: Direction) -> u8 {
    use Direction::*;
    match direction {
        Up => 0,
        UpRight => 1,
        Right => 2,
        DownRight => 3,
        Down => 4,
        DownLeft => 5,
        Left => 6,
        UpLeft => 7,
    }
}

fn quartile(current: usize, maximum: usize) -> u8 {
    ((current.min(maximum) as u128 * 4) / maximum.max(1) as u128).min(3) as u8
}

fn refuge_candidates(
    origin: Position,
    life: &LifeState,
    view: &Perception,
    threat: &super::perception::PerceivedEntity,
) -> Vec<RefugeCandidate> {
    let direction = threat.steps.0[0];
    let threat_distance = match threat.steps.len() {
        1 => 0,
        2 => 1,
        _ => 2,
    };
    let mut candidates: Vec<_> = view
        .shelters
        .iter()
        .enumerate()
        .filter_map(|(order, steps)| {
            if steps.0.is_empty() {
                return None;
            }
            let relative_endpoint = steps.0.iter().fold(Position::new(0, 0), |p, d| p + *d);
            if view.entities.iter().any(|entity| {
                entity
                    .steps
                    .0
                    .iter()
                    .fold(Position::new(0, 0), |p, d| p + *d)
                    == relative_endpoint
            }) {
                return None;
            }
            let endpoint = Position::new(
                origin.x + relative_endpoint.x,
                origin.y + relative_endpoint.y,
            );
            Some(RefugeCandidate {
                steps: steps.clone(),
                endpoint,
                order,
                key: RefugeKey {
                    threat_id: threat.id,
                    threat_species: threat.species.clone(),
                    bearing: bearing_bucket(direction),
                    threat_distance,
                    energy_quartile: quartile(life.energy, life.max_energy),
                    stamina_quartile: quartile(life.stamina, life.max_stamina),
                    refuge_options: Vec::new(),
                    endpoint,
                },
            })
        })
        .collect();
    let mut options = candidates
        .iter()
        .map(|candidate| candidate.endpoint)
        .collect::<Vec<_>>();
    options.sort_by_key(|p| (p.y, p.x));
    if options.len() > 8 {
        return Vec::new();
    }
    for candidate in &mut candidates {
        candidate.key.refuge_options = options.clone();
    }
    candidates
}

fn candidate_record<'a>(
    candidate: &RefugeCandidate,
    records: &'a [RefugeOutcome],
) -> Option<&'a RefugeOutcome> {
    records
        .iter()
        .find(|record| record.key == candidate.key && record.completed > 0)
}

fn compare_refuge_candidate(
    a: &RefugeCandidate,
    b: &RefugeCandidate,
    records: &[RefugeOutcome],
) -> Ordering {
    match (candidate_record(a, records), candidate_record(b, records)) {
        (Some(a_record), Some(b_record)) => {
            let safe_rate = (b_record.safe_arrivals as u64 * a_record.completed as u64)
                .cmp(&(a_record.safe_arrivals as u64 * b_record.completed as u64));
            if safe_rate != Ordering::Equal {
                return safe_rate;
            }
            let damage = (a_record.threat_damage * b_record.completed as u64)
                .cmp(&(b_record.threat_damage * a_record.completed as u64));
            if damage != Ordering::Equal {
                return damage;
            }
            let movement = (a_record.movement_energy * b_record.completed as u64)
                .cmp(&(b_record.movement_energy * a_record.completed as u64));
            if movement != Ordering::Equal {
                return movement;
            }
        }
        (Some(_), None) => return Ordering::Less,
        (None, Some(_)) => return Ordering::Greater,
        (None, None) => {}
    }
    (a.steps.0.len(), a.order).cmp(&(b.steps.0.len(), b.order))
}

pub(crate) fn validate(memory: &Memory) -> crate::Result<()> {
    let valid_key = |key: &RefugeKey| {
        key.threat_species.hunts(&Species::Herbivore)
            && key.bearing <= 7
            && key.threat_distance <= 2
            && key.energy_quartile <= 3
            && key.stamina_quartile <= 3
            && !key.refuge_options.is_empty()
            && key.refuge_options.len() <= 8
            && !key
                .refuge_options
                .windows(2)
                .any(|w| (w[0].y, w[0].x) >= (w[1].y, w[1].x))
            && key
                .refuge_options
                .iter()
                .all(|p| p.x.unsigned_abs() <= 1_000_000 && p.y.unsigned_abs() <= 1_000_000)
            && key.refuge_options.contains(&key.endpoint)
            && key.endpoint.x.unsigned_abs() <= 1_000_000
            && key.endpoint.y.unsigned_abs() <= 1_000_000
    };
    let valid_episode = |e: &RefugeEpisode| {
        valid_key(&e.key)
            && e.key.threat_id == e.threat_id
            && e.key.threat_species == e.threat_species
            && e.key.endpoint == e.target
            && e.target.x.unsigned_abs() <= 1_000_000
            && e.target.y.unsigned_abs() <= 1_000_000
            && e.elapsed_ticks <= 20
            && e.unseen_ticks <= 20
            && e.movement_energy <= MAX_REFUGE_ACCOUNT
            && e.threat_damage <= MAX_REFUGE_ACCOUNT
            && e.other_damage <= MAX_REFUGE_ACCOUNT
            && e.unattributed_health_loss <= MAX_REFUGE_ACCOUNT
            && (e.last_health as u128) <= MAX_REFUGE_ACCOUNT as u128
    };
    let mut keys = std::collections::HashSet::new();
    if memory.refuge_outcomes.len() > MAX_REFUGE_OUTCOMES
        || memory.pending_refuge_closures.len() > 1
        || memory.refuge_outcomes.iter().any(|r| {
            r.completed == 0
                || r.completed > MAX_REFUGE_EPISODES_PER_KEY
                || r.safe_arrivals > r.completed
                || r.threat_damage > MAX_REFUGE_ACCOUNT
                || r.movement_energy > MAX_REFUGE_ACCOUNT
                || !valid_key(&r.key)
                || !keys.insert(&r.key)
        })
        || memory
            .refuge_episode
            .as_ref()
            .is_some_and(|e| !valid_episode(e))
        || memory.pending_refuge_closures.iter().any(|closure| {
            !valid_episode(&closure.episode)
                || closure.final_pos.x.unsigned_abs() > 1_000_000
                || closure.final_pos.y.unsigned_abs() > 1_000_000
        })
    {
        return Err("invalid refuge learning state".into());
    }
    Ok(())
}
