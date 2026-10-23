#[path = "../examples/support/refuge_candidate.rs"]
mod refuge_candidate;

use refuge_candidate::*;

fn key() -> CandidateKey {
    CandidateKey {
        situation: Situation {
            subject_species: Some(Species::Herbivore),
            threat_species: Some(Species::Carnivore),
            bearing_sector: Some(1),
            distance_band: Some(1),
            energy_band: Some(2),
            stamina_band: Some(2),
            habitat_class: Some(3),
            subject_speed: Some(2),
        },
        route: Route {
            identity: RouteIdentity {
                endpoint_sector: 4,
                steps: vec![
                    ObservedStep {
                        direction_sector: 2,
                        terrain_class: 0,
                    },
                    ObservedStep {
                        direction_sector: 3,
                        terrain_class: 1,
                    },
                ],
            },
            refuge_kind: 0,
            turns: 1,
            estimated_movement_energy: Some(8),
        },
    }
}

fn completed(damage: u64, energy: u64) -> EpisodeEvidence {
    EpisodeEvidence::Completed {
        threat_damage: damage,
        movement_energy: energy,
    }
}

fn outcome(key: CandidateKey, tick: u64) -> Outcome {
    Outcome {
        key,
        last_completed_tick: tick,
        completed: 1,
        safe_arrivals: 1,
        threat_damage: 0,
        movement_energy: 1,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Difference {
    ThreatIdentityAndAlternativeSet,
    SubjectSpecies,
    ThreatSpecies,
    Bearing,
    Distance,
    Energy,
    Stamina,
    Habitat,
    Speed,
    EstimatedCost,
    RoutePath,
    RefugeKind,
    Turns,
}

#[derive(Debug, Clone)]
struct EpisodeContext {
    threat_id: u64,
    alternatives: Vec<RouteIdentity>,
}

struct FixturePair {
    label: Difference,
    current: CandidateKey,
    observed: CandidateKey,
    current_episode: EpisodeContext,
    observed_episode: EpisodeContext,
    expected: bool,
}

/// Fixed synthetic contract catalog. Positive cases change only external
/// episode metadata; all negative cases change one selected key feature.
fn synthetic_contract_pairs() -> Vec<FixturePair> {
    let base = key();
    let mut rows = Vec::new();
    let mut extra_route = base.route.identity.clone();
    extra_route.endpoint_sector = 5;
    rows.push(FixturePair {
        label: Difference::ThreatIdentityAndAlternativeSet,
        current: base.clone(),
        observed: base.clone(),
        current_episode: EpisodeContext {
            threat_id: 99,
            alternatives: vec![base.route.identity.clone()],
        },
        observed_episode: EpisodeContext {
            threat_id: 41,
            alternatives: vec![base.route.identity.clone(), extra_route],
        },
        expected: true,
    });
    macro_rules! negative {
        ($label:ident, $field:ident, $value:expr) => {{
            let mut changed = base.clone();
            changed.situation.$field = Some($value);
            rows.push(FixturePair {
                label: Difference::$label,
                current: base.clone(),
                observed: changed,
                current_episode: EpisodeContext {
                    threat_id: 41,
                    alternatives: vec![base.route.identity.clone()],
                },
                observed_episode: EpisodeContext {
                    threat_id: 41,
                    alternatives: vec![base.route.identity.clone()],
                },
                expected: false,
            });
        }};
    }
    negative!(SubjectSpecies, subject_species, Species::Omnivore);
    negative!(ThreatSpecies, threat_species, Species::Omnivore);
    negative!(Bearing, bearing_sector, 2);
    negative!(Distance, distance_band, 2);
    negative!(Energy, energy_band, 3);
    negative!(Stamina, stamina_band, 3);
    negative!(Habitat, habitat_class, 4);
    negative!(Speed, subject_speed, 3);
    let mut changed = base.clone();
    changed.route.estimated_movement_energy = Some(9);
    rows.push(FixturePair {
        label: Difference::EstimatedCost,
        current: base.clone(),
        observed: changed,
        current_episode: EpisodeContext {
            threat_id: 41,
            alternatives: vec![base.route.identity.clone()],
        },
        observed_episode: EpisodeContext {
            threat_id: 41,
            alternatives: vec![base.route.identity.clone()],
        },
        expected: false,
    });

    let mut changed = base.clone();
    changed.route.identity.steps[1].terrain_class = 9;
    rows.push(FixturePair {
        label: Difference::RoutePath,
        current: base.clone(),
        observed: changed,
        current_episode: EpisodeContext {
            threat_id: 41,
            alternatives: vec![base.route.identity.clone()],
        },
        observed_episode: EpisodeContext {
            threat_id: 41,
            alternatives: vec![base.route.identity.clone()],
        },
        expected: false,
    });
    let mut changed = base.clone();
    changed.route.refuge_kind = 1;
    rows.push(FixturePair {
        label: Difference::RefugeKind,
        current: base.clone(),
        observed: changed,
        current_episode: EpisodeContext {
            threat_id: 41,
            alternatives: vec![base.route.identity.clone()],
        },
        observed_episode: EpisodeContext {
            threat_id: 41,
            alternatives: vec![base.route.identity.clone()],
        },
        expected: false,
    });
    let mut changed = base.clone();
    changed.route.turns = 2;
    rows.push(FixturePair {
        label: Difference::Turns,
        current: base.clone(),
        observed: changed,
        current_episode: EpisodeContext {
            threat_id: 41,
            alternatives: vec![base.route.identity.clone()],
        },
        observed_episode: EpisodeContext {
            threat_id: 41,
            alternatives: vec![base.route.identity.clone()],
        },
        expected: false,
    });

    rows
}

#[test]
fn identity_and_alternative_metadata_are_outside_the_selected_feature_key() {
    // Those attribution/current-option facts live beside the active episode,
    // never in the persistent reusable feature key.
    let old_episode_metadata = (
        41_u64,
        vec![key().route.identity.clone(), key().route.identity],
    );
    let new_episode_metadata = (99_u64, vec![key().route.identity]);
    assert_ne!(old_episode_metadata, new_episode_metadata);
    assert!(compatible(&key(), &key()));
}

#[test]
fn fixed_synthetic_contract_catalog_has_exact_expected_categorization() {
    let pairs = synthetic_contract_pairs();
    assert_eq!(pairs.len(), 13);
    for pair in pairs {
        if pair.label == Difference::ThreatIdentityAndAlternativeSet {
            assert_ne!(
                pair.current_episode.threat_id,
                pair.observed_episode.threat_id
            );
            assert_ne!(
                pair.current_episode.alternatives,
                pair.observed_episode.alternatives
            );
        }
        assert_eq!(
            compatible(&pair.current, &pair.observed),
            pair.expected,
            "{:?}",
            pair.label
        );
    }
}

#[test]
fn different_threat_id_updates_the_same_row_and_aggregate_ranks_candidates() {
    let learned_route = key();
    let mut other_route = key();
    other_route.route.identity.endpoint_sector = 5;
    let episode_a = (41_u64, vec![learned_route.route.identity.clone()]);
    let episode_b = (
        99_u64,
        vec![
            learned_route.route.identity.clone(),
            other_route.route.identity.clone(),
        ],
    );
    assert_ne!(episode_a, episode_b);

    let mut rows = vec![];
    record_outcome(&mut rows, learned_route.clone(), completed(0, 20), 1, 4);
    // Attribution IDs are external to this pure reusable row, so the second
    // independently attributed episode aggregates into the same feature key.
    record_outcome(&mut rows, learned_route.clone(), completed(2, 20), 2, 4);
    record_outcome(&mut rows, other_route.clone(), completed(1, 1), 3, 4);

    assert_eq!(rows.len(), 2);
    let aggregate = rows.iter().find(|row| row.key == learned_route).unwrap();
    assert_eq!(aggregate.completed, 2);
    assert_eq!(aggregate.safe_arrivals, 1);
    assert_eq!(aggregate.threat_damage, 2);
    assert_eq!(
        rank_candidates(&[other_route, learned_route], &rows, 1),
        Some(vec![1, 0])
    );
}

#[test]
fn missing_physical_observable_fails_closed() {
    let mut current = key();
    current.situation.subject_speed = None;
    assert!(!compatible(&current, &key()));
    let mut observed = key();
    observed.route.estimated_movement_energy = None;
    assert!(!compatible(&key(), &observed));
}

#[test]
fn distinct_visible_routes_do_not_alias_even_with_equal_scalar_costs() {
    let short_left = key();
    let mut short_right = key();
    short_right.route.identity.endpoint_sector = 6;
    let mut rows = vec![];
    record_outcome(&mut rows, short_left.clone(), completed(0, 4), 1, 4);
    assert!(!compatible(&short_right, &short_left));
    assert!(rank_candidates(&[short_left, short_right], &rows, 1).is_none());
}

#[test]
fn censored_episode_does_not_create_or_update_safety_evidence() {
    let reasons = [
        CensorReason::OtherAttacker,
        CensorReason::UnattributedHealthLoss,
        CensorReason::Death,
        CensorReason::ThreatLost,
        CensorReason::Timeout,
        CensorReason::InvalidRoute,
        CensorReason::Interrupted,
        CensorReason::AmbiguousArrival,
        CensorReason::LabBoundary,
    ];
    let mut rows = vec![];
    for reason in reasons {
        assert!(!record_outcome(
            &mut rows,
            key(),
            EpisodeEvidence::Censored(reason),
            10,
            4,
        ));
        assert!(rows.is_empty());
    }
}

#[test]
fn missing_candidate_support_falls_back_instead_of_assuming_zero_risk() {
    let known = key();
    let mut unknown = key();
    unknown.route.identity.endpoint_sector = 5;
    let mut rows = vec![];
    record_outcome(&mut rows, known.clone(), completed(0, 4), 1, 4);
    assert!(rank_candidates(&[known, unknown], &rows, 1).is_none());
}

#[test]
fn one_history_row_cannot_be_reused_for_two_current_candidates() {
    let known = key();
    let same_physical_route = key();
    // A distinct currently valid choice with indistinguishable route features
    // is ambiguous: the same observation cannot stand in for both choices.
    // The schema intentionally has no endpoint coordinates; a duplicate
    // feature key in the current candidate list is detected through row reuse.
    let mut rows = vec![];
    record_outcome(&mut rows, known.clone(), completed(0, 4), 1, 4);
    assert!(rank_candidates(&[known, same_physical_route], &rows, 1).is_none());
}

#[test]
fn evidence_ranks_safety_before_cost_with_stable_ties() {
    let safe_but_costly = key();
    let mut damaged_but_cheap = key();
    damaged_but_cheap.route.identity.endpoint_sector = 5;
    let mut rows = vec![];
    record_outcome(&mut rows, safe_but_costly.clone(), completed(0, 40), 1, 4);
    record_outcome(&mut rows, damaged_but_cheap.clone(), completed(3, 1), 1, 4);
    assert_eq!(
        rank_candidates(&[damaged_but_cheap, safe_but_costly], &rows, 1),
        Some(vec![1, 0])
    );
}

#[test]
fn retention_is_bounded_and_uses_a_stable_key_tiebreak() {
    let low = key();
    let mut high = key();
    high.route.identity.endpoint_sector = 5;
    let mut newer = key();
    newer.route.identity.endpoint_sector = 6;
    let mut rows = vec![];
    record_outcome(&mut rows, high.clone(), completed(0, 1), 7, 2);
    record_outcome(&mut rows, low.clone(), completed(0, 1), 7, 2);
    record_outcome(&mut rows, newer.clone(), completed(0, 1), 8, 2);
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().any(|row| row.key == high));
    assert!(rows.iter().any(|row| row.key == newer));
    assert!(!rows.iter().any(|row| row.key == low));
}

#[test]
fn changed_capacity_trims_oversized_preloaded_table_before_update() {
    let a = key();
    let mut b = key();
    b.route.identity.endpoint_sector = 5;
    let mut c = key();
    c.route.identity.endpoint_sector = 6;
    let mut rows = vec![outcome(a.clone(), 1), outcome(b.clone(), 2), outcome(c, 3)];
    assert!(record_outcome(&mut rows, b.clone(), completed(0, 1), 4, 1));
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].key, b);
    assert_eq!(rows[0].completed, 2);
}

#[test]
fn zero_capacity_clears_preloaded_rows() {
    let mut rows = vec![outcome(key(), 1)];
    assert!(!record_outcome(&mut rows, key(), completed(0, 1), 2, 0));
    assert!(rows.is_empty());
}

#[test]
fn aggregate_overflow_freezes_the_entire_row() {
    let key = key();
    let mut row = outcome(key.clone(), 1);
    row.completed = u32::MAX;
    row.safe_arrivals = u32::MAX;
    let mut rows = vec![row.clone()];
    assert!(!record_outcome(
        &mut rows,
        key.clone(),
        completed(0, 1),
        2,
        1
    ));
    assert_eq!(rows[0], row);

    row.completed = 1;
    row.safe_arrivals = 1;
    row.threat_damage = u64::MAX;
    rows = vec![row.clone()];
    assert!(!record_outcome(
        &mut rows,
        key.clone(),
        completed(1, 1),
        2,
        1
    ));
    assert_eq!(rows[0], row);

    row.threat_damage = 0;
    row.movement_energy = u64::MAX;
    rows = vec![row.clone()];
    assert!(!record_outcome(&mut rows, key, completed(0, 1), 2, 1));
    assert_eq!(rows[0], row);
}
