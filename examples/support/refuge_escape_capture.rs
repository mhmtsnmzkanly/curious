//! Bounded, read-only capture helpers for the continuous-life refuge lab.
//!
//! This module reconstructs candidate keys and eligibility from the actual
//! production DecisionTrace plus the table that existed before that tick. It
//! never calls a decision function or advances a World.

use curious::{
    entity::{
        decision::{DecisionTrace, Reason},
        intent::Intent,
        memory::{GeneralizedRefugeKey, GeneralizedRefugeOutcome, RefugeEpisode},
        perception::PerceivedEntity,
        species::Species,
    },
    map::movement::{Position, Steps},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const FOCAL_ID: usize = 1;

#[derive(Clone, Debug, Serialize)]
pub struct CandidateWitness {
    pub route: Steps,
    pub endpoint: Position,
    pub key: Option<GeneralizedRefugeKey>,
    pub supported_unique_fresh: bool,
    pub different_threat_id_supported: bool,
    pub support_sources: Vec<usize>,
    pub support_completed: u32,
    pub support_last_completed_tick: Option<usize>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ChoiceWitness {
    pub choice_tick: usize,
    pub final_reason: Reason,
    pub position: Position,
    pub life_before_choice: curious::entity::lifestate::LifeState,
    pub rng_before_choice: u64,
    pub perception_witness: Option<curious::entity::perception::Perception>,
    pub threat_id: Option<usize>,
    pub active_episode_before: Option<RefugeEpisode>,
    pub is_fresh: bool,
    pub selection_kind: &'static str,
    pub expected_supported_greedy_index: Option<usize>,
    pub expected_fallback_index: Option<usize>,
    pub selected_candidate_index: Option<usize>,
    pub reconstructed_selection_matches: Option<bool>,
    pub table_differs_from_fork_at_choice: Option<bool>,
    pub resolver_updates_before_choice: usize,
    pub candidates: Vec<CandidateWitness>,
    pub candidate_cap_exceeded: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateSource {
    pub key: GeneralizedRefugeKey,
    pub threat_id: usize,
    pub ended_tick: usize,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Provenance {
    sources: Vec<UpdateSource>,
}

impl Provenance {
    pub fn reset(&mut self) {
        self.sources.clear();
    }

    /// Account for table keys removed by ordinary retention or an explicit
    /// treatment. A replacement record for the same key then starts a new
    /// provenance sequence.
    pub fn retain_table_keys(&mut self, table: &[GeneralizedRefugeOutcome]) {
        self.sources
            .retain(|source| table.iter().any(|row| row.key == source.key));
    }

    pub fn record(&mut self, key: GeneralizedRefugeKey, threat_id: usize, ended_tick: usize) {
        self.sources.push(UpdateSource {
            key,
            threat_id,
            ended_tick,
        });
    }

    pub fn source_ids(&self, key: &GeneralizedRefugeKey) -> Vec<usize> {
        self.sources
            .iter()
            .filter(|source| source.key == *key)
            .map(|source| source.threat_id)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    pub fn has_different_id_support(&self, key: &GeneralizedRefugeKey, current_id: usize) -> bool {
        self.sources
            .iter()
            .any(|source| source.key == *key && source.threat_id != current_id)
    }

    pub fn rows(&self) -> &[UpdateSource] {
        &self.sources
    }
}

pub fn retain_fresh_rows(rows: &mut Vec<GeneralizedRefugeOutcome>, tick: usize) {
    rows.retain(|row| {
        tick.saturating_sub(row.last_completed_tick)
            <= curious::entity::refuge_generalization::GENERALIZED_REFUGE_TTL_TICKS
    });
}

fn threat_for<'a>(trace: &'a DecisionTrace) -> Option<&'a PerceivedEntity> {
    trace.perception.as_ref()?.entities.iter().find(|entity| {
        entity.species.hunts(&trace.species)
            && entity.steps.len() <= 2
            && !entity.steps.0.is_empty()
    })
}

fn endpoint(route: &Steps, origin: Position) -> Position {
    route
        .0
        .iter()
        .fold(origin, |position, direction| position + *direction)
}

fn fallback_index(candidates: &[CandidateWitness]) -> Option<usize> {
    candidates
        .iter()
        .enumerate()
        .min_by_key(|(order, candidate)| (candidate.route.0.len(), *order))
        .map(|(index, _)| index)
}

fn enforce_candidate_cap(candidates: &mut Vec<CandidateWitness>) -> bool {
    if candidates.len() > curious::entity::refuge_generalization::MAX_GENERALIZED_REFUGE_OUTCOMES {
        candidates.clear();
        true
    } else {
        false
    }
}

/// Measure one real production decision. `table` and `active_episode` are
/// cloned from the focal memory immediately before the single World tick.
pub fn capture_choice(
    trace: &DecisionTrace,
    table: &[GeneralizedRefugeOutcome],
    active_episode: Option<RefugeEpisode>,
    provenance: &Provenance,
    generalization_enabled: bool,
) -> ChoiceWitness {
    let threat = threat_for(trace);
    let Some(view) = trace.perception.as_ref() else {
        return ChoiceWitness {
            choice_tick: trace.tick,
            final_reason: trace.decision.reason,
            position: trace.pos,
            life_before_choice: trace.before.clone(),
            rng_before_choice: trace.rng_before,
            perception_witness: None,
            threat_id: threat.map(|entity| entity.id),
            active_episode_before: active_episode,
            is_fresh: false,
            selection_kind: "perception_unavailable",
            expected_supported_greedy_index: None,
            expected_fallback_index: None,
            selected_candidate_index: None,
            reconstructed_selection_matches: None,
            table_differs_from_fork_at_choice: None,
            resolver_updates_before_choice: 0,
            candidates: Vec::new(),
            candidate_cap_exceeded: false,
        };
    };

    let mut candidates = Vec::new();
    if let Some(threat) = threat {
        for route in &view.shelters {
            if route.0.is_empty() {
                continue;
            }
            let relative = endpoint(route, Position::new(0, 0));
            let occupied = view
                .entities
                .iter()
                .any(|entity| endpoint(&entity.steps, Position::new(0, 0)) == relative);
            if occupied {
                continue;
            }
            let absolute = endpoint(route, trace.pos);
            let key = threat.health.and_then(|health| {
                curious::entity::refuge_generalization::candidate_key(
                    trace.pos,
                    trace.species.clone(),
                    &trace.before,
                    view,
                    threat.species.clone(),
                    health,
                    threat.power,
                    &threat.steps,
                    route,
                )
            });
            candidates.push(CandidateWitness {
                route: route.clone(),
                endpoint: absolute,
                key,
                supported_unique_fresh: false,
                different_threat_id_supported: false,
                support_sources: Vec::new(),
                support_completed: 0,
                support_last_completed_tick: None,
            });
        }
    }
    let candidate_cap_exceeded = enforce_candidate_cap(&mut candidates);

    let keys = candidates
        .iter()
        .map(|candidate| candidate.key.clone())
        .collect::<Option<Vec<_>>>();
    let ranked = generalization_enabled
        .then_some(())
        .and_then(|_| keys.as_ref())
        .and_then(|keys| {
            curious::entity::refuge_generalization::rank_candidates(keys, table, trace.tick)
        });
    let expected_supported_greedy_index =
        ranked.as_ref().and_then(|indices| indices.first().copied());
    let expected_fallback_index = fallback_index(&candidates);

    if let Some(threat) = threat {
        for candidate in &mut candidates {
            if let Some(key) = &candidate.key {
                candidate.support_sources = provenance.source_ids(key);
                let matches = table
                    .iter()
                    .filter(|row| row.key == *key)
                    .collect::<Vec<_>>();
                if matches.len() == 1 {
                    let row = matches[0];
                    candidate.support_completed = row.completed;
                    candidate.support_last_completed_tick = Some(row.last_completed_tick);
                    candidate.supported_unique_fresh = row.completed > 0
                        && row.last_completed_tick <= trace.tick
                        && trace.tick.saturating_sub(row.last_completed_tick)
                            <= curious::entity::refuge_generalization::GENERALIZED_REFUGE_TTL_TICKS;
                }
                candidate.different_threat_id_supported =
                    provenance.has_different_id_support(key, threat.id);
            }
        }
    }

    let selected_steps = match &trace.decision.intent {
        Intent::Move { steps } => Some(steps),
        _ => None,
    };
    let selected_matches = selected_steps.map(|steps| {
        let matches = candidates
            .iter()
            .enumerate()
            .filter(|(_, candidate)| {
                candidate
                    .route
                    .0
                    .iter()
                    .take(trace.before.speed)
                    .copied()
                    .eq(steps.0.iter().copied())
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        if matches.len() == 1 {
            Some(matches[0])
        } else {
            None
        }
    });
    let selected_candidate_index = selected_matches.flatten();
    let is_fresh = trace.decision.reason == Reason::SeekShelter
        && !active_episode.as_ref().is_some_and(|episode| {
            episode.learning_mode == curious::entity::memory::RefugeLearningMode::Generalized
                && Some(episode.threat_id) == threat.map(|entity| entity.id)
                && threat.is_some_and(|entity| episode.threat_species == entity.species)
                && selected_candidate_index
                    .and_then(|index| candidates.get(index))
                    .is_some_and(|candidate| candidate.endpoint == episode.target)
        });
    let selection_kind = if !generalization_enabled {
        "rule_baseline"
    } else if trace.decision.reason != Reason::SeekShelter {
        "ordinary_rule_other_reason"
    } else if candidate_cap_exceeded {
        "production_no_candidates_over_cap"
    } else if candidates.is_empty() {
        "production_no_candidates"
    } else if keys.is_none() {
        "ordinary_fallback_missing_candidate_key"
    } else if ranked.is_some() {
        "generalized_supported_greedy"
    } else {
        "generalized_fallback_unsupported"
    };
    let expected_index = if ranked.is_some() {
        expected_supported_greedy_index
    } else {
        expected_fallback_index
    };

    ChoiceWitness {
        choice_tick: trace.tick,
        final_reason: trace.decision.reason,
        position: trace.pos,
        life_before_choice: trace.before.clone(),
        rng_before_choice: trace.rng_before,
        perception_witness: Some(view.clone()),
        threat_id: threat.map(|entity| entity.id),
        active_episode_before: active_episode,
        is_fresh,
        selection_kind,
        expected_supported_greedy_index,
        expected_fallback_index,
        selected_candidate_index,
        reconstructed_selection_matches: selected_candidate_index
            .map(|actual| Some(actual) == expected_index),
        table_differs_from_fork_at_choice: None,
        resolver_updates_before_choice: 0,
        candidates,
        candidate_cap_exceeded,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key() -> GeneralizedRefugeKey {
        GeneralizedRefugeKey {
            subject_species: Species::Herbivore,
            threat_species: Species::Carnivore,
            threat_health: 10,
            threat_power: 20,
            threat_bearing_sector: 0,
            threat_distance_steps: 1,
            subject_health: 40,
            subject_max_health: 40,
            subject_energy: 50,
            subject_max_energy: 50,
            subject_stamina: 10,
            subject_max_stamina: 10,
            subject_speed: 1,
            move_energy_cost: 1,
            origin_habitat: curious::entity::refuge_generalization::ObservedCellKind::Empty,
            endpoint_sector: 2,
            route: vec![curious::entity::refuge_generalization::ObservedStep {
                direction_sector: 2,
                cell_kind: curious::entity::refuge_generalization::ObservedCellKind::Burrow,
            }],
            refuge_kind: curious::entity::refuge_generalization::ObservedCellKind::Burrow,
            turns: 0,
            estimated_movement_energy: 1,
        }
    }

    #[test]
    fn provenance_round_trips_and_distinguishes_threat_ids() {
        let key = key();
        let mut provenance = Provenance::default();
        provenance.record(key.clone(), 2, 8);
        let encoded = serde_json::to_string(&provenance).unwrap();
        let decoded: Provenance = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded.source_ids(&key), vec![2]);
        assert!(decoded.has_different_id_support(&key, 3));
        assert!(!decoded.has_different_id_support(&key, 2));
    }

    #[test]
    fn fallback_uses_original_candidate_input_order_for_equal_lengths() {
        let candidates = vec![
            CandidateWitness {
                route: Steps(vec![curious::map::movement::Direction::Left]),
                endpoint: Position::new(10, 10),
                key: None,
                supported_unique_fresh: false,
                different_threat_id_supported: false,
                support_sources: vec![],
                support_completed: 0,
                support_last_completed_tick: None,
            },
            CandidateWitness {
                route: Steps(vec![curious::map::movement::Direction::Right]),
                endpoint: Position::new(-10, -10),
                key: None,
                supported_unique_fresh: false,
                different_threat_id_supported: false,
                support_sources: vec![],
                support_completed: 0,
                support_last_completed_tick: None,
            },
        ];
        assert_eq!(fallback_index(&candidates), Some(0));
    }

    #[test]
    fn candidate_cap_clears_ninth_option_instead_of_exposing_support() {
        let mut candidates = (0..9)
            .map(|x| CandidateWitness {
                route: Steps(vec![curious::map::movement::Direction::Right]),
                endpoint: Position::new(x, 0),
                key: None,
                supported_unique_fresh: false,
                different_threat_id_supported: false,
                support_sources: vec![],
                support_completed: 0,
                support_last_completed_tick: None,
            })
            .collect::<Vec<_>>();
        assert!(enforce_candidate_cap(&mut candidates));
        assert!(candidates.is_empty());
    }

    #[test]
    fn expired_same_key_row_recreation_discards_old_threat_provenance() {
        let key = key();
        let old = GeneralizedRefugeOutcome {
            key: key.clone(),
            last_completed_tick: 1,
            completed: 1,
            safe_arrivals: 1,
            threat_damage: 0,
            movement_energy: 2,
        };
        let new = GeneralizedRefugeOutcome {
            key: key.clone(),
            last_completed_tick: 102,
            completed: 1,
            safe_arrivals: 1,
            threat_damage: 0,
            movement_energy: 3,
        };
        let mut provenance = Provenance::default();
        provenance.record(key.clone(), 2, 1);
        let mut rows = vec![old];
        retain_fresh_rows(&mut rows, 102);
        provenance.retain_table_keys(&rows);
        assert!(provenance.source_ids(&key).is_empty());
        rows.push(new);
        provenance.record(key.clone(), 9, 102);
        assert_eq!(provenance.source_ids(&key), vec![9]);
    }
}
