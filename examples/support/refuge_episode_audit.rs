//! Observation-only episode measurement for both refuge-lab arms.
use curious::{
    entity::{
        decision::{DecisionTrace, Reason},
        intent::Intent,
        lifestate::LifeState,
        perception::Perception,
        species::Species,
    },
    map::movement::{Direction, Position, Steps},
    simulation::SimulationEvent,
};
use serde::Serialize;

const SUBJECT_ID: usize = 1;
const PREDATOR_ID: usize = 2;
const MAX_EPISODE_TICKS: u8 = 20;
const MAX_ACCOUNT: u64 = 1_000_000_000_000;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ContextKey {
    pub threat_id: usize,
    pub threat_species: Species,
    pub bearing: u8,
    pub threat_distance: u8,
    pub energy_quartile: u8,
    pub stamina_quartile: u8,
    pub refuge_options: Vec<Position>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditedEpisode {
    pub id: usize,
    pub threat_id: usize,
    pub threat_species: Species,
    pub context_key: ContextKey,
    pub started_tick: usize,
    pub ended_tick: usize,
    pub endpoint: Option<Position>,
    pub endpoint_resolution: &'static str,
    pub endpoint_candidates: Vec<Position>,
    pub final_pos: Position,
    pub closure: AuditClosure,
    pub completed: bool,
    pub safe_arrival: bool,
    pub attribution_known: bool,
    pub threat_damage: u64,
    pub other_damage: u64,
    pub unattributed_health_loss: u64,
    pub movement_energy: u64,
    pub starting_health: usize,
    pub final_health: usize,
    pub initial_reason: Reason,
    pub initial_intent: Intent,
    pub initial_selected_path: Steps,
    pub visible_threat_position: Position,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditClosure {
    Arrived,
    ThreatLost,
    Died,
    TimedOut,
    TargetChanged,
    PriorityInterrupted,
    OptionsInvalid,
    LabBoundary,
    EndpointAmbiguous,
}

#[derive(Debug, Clone)]
struct ActiveEpisode {
    context_key: ContextKey,
    started_tick: usize,
    endpoint: Option<Position>,
    endpoint_resolution: &'static str,
    endpoint_candidates: Vec<Position>,
    selected_path: Steps,
    reason: Reason,
    intent: Intent,
    visible_threat_position: Position,
    starting_health: usize,
    last_health: usize,
    elapsed: u8,
    threat_damage: u64,
    other_damage: u64,
    unattributed_health_loss: u64,
    movement_energy: u64,
}

#[derive(Debug, Default)]
pub struct Tracker {
    active: Option<ActiveEpisode>,
    pub closures: Vec<AuditedEpisode>,
}

fn threat(view: &Perception) -> Option<&curious::entity::perception::PerceivedEntity> {
    view.entities.iter().find(|entity| {
        entity.species.hunts(&Species::Herbivore)
            && entity.id == PREDATOR_ID
            && entity.steps.len() <= 2
            && !entity.steps.0.is_empty()
    })
}

fn bearing(direction: Direction) -> u8 {
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

fn endpoint(path: &Steps, origin: Position) -> Position {
    path.0.iter().fold(origin, |p, direction| p + *direction)
}

fn options(view: &Perception, origin: Position) -> Vec<(Position, Steps)> {
    let mut result = view
        .shelters
        .iter()
        .filter(|path| !path.0.is_empty())
        .filter_map(|path| {
            let relative = endpoint(path, Position::new(0, 0));
            let occupied = view
                .entities
                .iter()
                .any(|entity| endpoint(&entity.steps, Position::new(0, 0)) == relative);
            (!occupied).then(|| (endpoint(path, origin), path.clone()))
        })
        .collect::<Vec<_>>();
    result.sort_by_key(|(pos, _)| (pos.y, pos.x));
    result
}

fn selected_endpoint(
    view: &Perception,
    origin: Position,
    intent: &Intent,
) -> (Option<Position>, &'static str, Vec<Position>, Steps) {
    let paths = options(view, origin);
    let candidates = paths.iter().map(|(p, _)| *p).collect::<Vec<_>>();
    let Intent::Move { steps: chosen } = intent else {
        return (None, "no_move_path", candidates, Steps(vec![]));
    };
    let matches = paths
        .iter()
        .filter(|(_, path)| {
            chosen.0.len() <= path.0.len() && path.0.iter().take(chosen.0.len()).eq(chosen.0.iter())
        })
        .map(|(pos, _)| *pos)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [only] => (Some(*only), "unique", candidates, chosen.clone()),
        [] => (None, "unmatched", candidates, chosen.clone()),
        _ => (None, "ambiguous_prefix", candidates, chosen.clone()),
    }
}

fn context(trace: &DecisionTrace, view: &Perception) -> Option<(ContextKey, Position)> {
    let threat = threat(view)?;
    let threat_pos = endpoint(&threat.steps, trace.pos);
    let mut alternatives = options(view, trace.pos)
        .iter()
        .map(|(pos, _)| *pos)
        .collect::<Vec<_>>();
    alternatives.sort_by_key(|p| (p.y, p.x));
    let key = ContextKey {
        threat_id: threat.id,
        threat_species: threat.species.clone(),
        bearing: bearing(threat.steps.0[0]),
        threat_distance: match threat.steps.len() {
            1 => 0,
            2 => 1,
            _ => 2,
        },
        energy_quartile: quartile(trace.before.energy, trace.before.max_energy),
        stamina_quartile: quartile(trace.before.stamina, trace.before.max_stamina),
        refuge_options: alternatives,
    };
    Some((key, threat_pos))
}

impl Tracker {
    /// Consume the subject's current decision trace before attributing this tick's resolver events.
    pub fn before_resolver(&mut self, trace: Option<&DecisionTrace>, tick: usize) {
        let Some(trace) = trace.filter(|trace| trace.id == SUBJECT_ID) else {
            return;
        };
        let Some(view) = trace.perception.as_ref() else {
            return;
        };
        let observed = context(trace, view);
        let current_threat = observed.as_ref().map(|(key, _)| key);
        if let Some(active) = self.active.as_mut() {
            reconcile_health(active, trace.before.health, 0);
        }
        if current_threat.is_none() {
            self.close(
                AuditClosure::ThreatLost,
                trace.pos,
                trace.before.health,
                tick,
            );
            return;
        }
        if trace.decision.reason != Reason::SeekShelter {
            self.close(
                AuditClosure::PriorityInterrupted,
                trace.pos,
                trace.before.health,
                tick,
            );
            return;
        }
        let (context_key, threat_pos) = observed.unwrap();
        let (endpoint, endpoint_resolution, endpoint_candidates, selected_path) =
            selected_endpoint(view, trace.pos, &trace.decision.intent);
        if context_key.refuge_options.is_empty()
            || endpoint.is_none() && endpoint_candidates.is_empty()
        {
            if self.active.is_some() {
                self.close(
                    AuditClosure::OptionsInvalid,
                    trace.pos,
                    trace.before.health,
                    tick,
                );
            } else {
                self.active = Some(ActiveEpisode {
                    context_key,
                    started_tick: tick,
                    endpoint,
                    endpoint_resolution,
                    endpoint_candidates,
                    selected_path,
                    reason: trace.decision.reason,
                    intent: trace.decision.intent.clone(),
                    visible_threat_position: threat_pos,
                    starting_health: trace.before.health,
                    last_health: trace.before.health,
                    elapsed: 0,
                    threat_damage: 0,
                    other_damage: 0,
                    unattributed_health_loss: 0,
                    movement_energy: 0,
                });
                self.close(
                    AuditClosure::OptionsInvalid,
                    trace.pos,
                    trace.before.health,
                    tick,
                );
            }
            return;
        }
        let same = self.active.as_ref().is_some_and(|active| {
            active.context_key.threat_id == context_key.threat_id
                && active.context_key.threat_species == context_key.threat_species
                && active.endpoint == endpoint
                && (endpoint.is_some() || active.selected_path == selected_path)
        });
        if same {
            return;
        }
        if self.active.is_some() {
            self.close(
                AuditClosure::TargetChanged,
                trace.pos,
                trace.before.health,
                tick,
            );
        }
        self.active = Some(ActiveEpisode {
            context_key,
            started_tick: tick,
            endpoint,
            endpoint_resolution,
            endpoint_candidates,
            selected_path,
            reason: trace.decision.reason,
            intent: trace.decision.intent.clone(),
            visible_threat_position: threat_pos,
            starting_health: trace.before.health,
            last_health: trace.before.health,
            elapsed: 0,
            threat_damage: 0,
            other_damage: 0,
            unattributed_health_loss: 0,
            movement_energy: 0,
        });
    }

    /// Add actual resolver evidence and close only on physical outcome or registered censoring.
    pub fn after_resolver(
        &mut self,
        events: &[SimulationEvent],
        subject_pos: Position,
        subject_life: &LifeState,
        subject_alive: bool,
        burrows: &[Position],
        tick: usize,
    ) {
        let Some(active) = self.active.as_mut() else {
            return;
        };
        let mut attack_damage = 0u64;
        for event in events {
            match event {
                SimulationEvent::EntityMoved {
                    id, energy_cost, ..
                } if *id == SUBJECT_ID => {
                    active.movement_energy = active
                        .movement_energy
                        .saturating_add(*energy_cost as u64)
                        .min(MAX_ACCOUNT);
                }
                SimulationEvent::EntityAttacked {
                    attacker,
                    target,
                    damage,
                    ..
                } if *target == SUBJECT_ID => {
                    attack_damage = attack_damage.saturating_add(*damage as u64);
                    if *attacker == active.context_key.threat_id {
                        active.threat_damage = active
                            .threat_damage
                            .saturating_add(*damage as u64)
                            .min(MAX_ACCOUNT);
                    } else {
                        active.other_damage = active
                            .other_damage
                            .saturating_add(*damage as u64)
                            .min(MAX_ACCOUNT);
                    }
                }
                _ => {}
            }
        }
        reconcile_health(active, subject_life.health, attack_damage);
        active.elapsed = active.elapsed.saturating_add(1).min(MAX_EPISODE_TICKS);
        let closure = if !subject_alive {
            Some(AuditClosure::Died)
        } else if active
            .endpoint
            .is_some_and(|endpoint| endpoint == subject_pos && burrows.contains(&endpoint))
        {
            Some(AuditClosure::Arrived)
        } else if active.endpoint.is_none() && burrows.contains(&subject_pos) {
            Some(AuditClosure::EndpointAmbiguous)
        } else if active.elapsed >= MAX_EPISODE_TICKS {
            Some(AuditClosure::TimedOut)
        } else {
            None
        };
        if let Some(closure) = closure {
            self.close(closure, subject_pos, subject_life.health, tick);
        }
    }

    pub fn censor_at_boundary(&mut self, position: Position, health: usize, tick: usize) {
        self.close(AuditClosure::LabBoundary, position, health, tick);
    }

    pub fn take_closures(&mut self) -> Vec<AuditedEpisode> {
        std::mem::take(&mut self.closures)
    }

    fn close(&mut self, closure: AuditClosure, final_pos: Position, health: usize, tick: usize) {
        let Some(mut episode) = self.active.take() else {
            return;
        };
        reconcile_health(&mut episode, health, 0);
        let completed = closure == AuditClosure::Arrived;
        let attribution_known = episode.other_damage == 0 && episode.unattributed_health_loss == 0;
        let safe_arrival = completed && attribution_known && episode.threat_damage == 0;
        self.closures.push(AuditedEpisode {
            id: SUBJECT_ID,
            threat_id: episode.context_key.threat_id,
            threat_species: episode.context_key.threat_species.clone(),
            context_key: episode.context_key,
            started_tick: episode.started_tick,
            ended_tick: tick,
            endpoint: episode.endpoint,
            endpoint_resolution: episode.endpoint_resolution,
            endpoint_candidates: episode.endpoint_candidates,
            final_pos,
            closure,
            completed,
            safe_arrival,
            attribution_known,
            threat_damage: episode.threat_damage,
            other_damage: episode.other_damage,
            unattributed_health_loss: episode.unattributed_health_loss,
            movement_energy: episode.movement_energy,
            starting_health: episode.starting_health,
            final_health: health,
            initial_reason: episode.reason,
            initial_intent: episode.intent,
            initial_selected_path: episode.selected_path,
            visible_threat_position: episode.visible_threat_position,
        });
    }
}

fn reconcile_health(active: &mut ActiveEpisode, health: usize, attack_damage: u64) {
    let loss = active.last_health.saturating_sub(health) as u64;
    active.unattributed_health_loss = active
        .unattributed_health_loss
        .saturating_add(loss.saturating_sub(attack_damage))
        .min(MAX_ACCOUNT);
    active.last_health = health;
}

#[cfg(test)]
mod tests {
    use super::*;
    use curious::entity::perception::Perception;

    #[test]
    fn endpoint_resolution_refuses_ambiguous_move_prefixes() {
        let mut view = Perception::empty();
        view.shelters = vec![
            Steps(vec![Direction::Right]),
            Steps(vec![Direction::Right, Direction::Right]),
        ];
        let one_step = Intent::Move {
            steps: Steps(vec![Direction::Right]),
        };
        let (endpoint, status, candidates, _) =
            selected_endpoint(&view, Position::new(3, 3), &one_step);
        assert_eq!(endpoint, None);
        assert_eq!(status, "ambiguous_prefix");
        assert_eq!(candidates.len(), 2);

        let two_steps = Intent::Move {
            steps: Steps(vec![Direction::Right, Direction::Right]),
        };
        let (endpoint, status, _, _) = selected_endpoint(&view, Position::new(3, 3), &two_steps);
        assert_eq!(endpoint, Some(Position::new(5, 3)));
        assert_eq!(status, "unique");
    }
}
