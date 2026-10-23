#[path = "../examples/support/refuge_candidate.rs"]
mod refuge_candidate;

use refuge_candidate::*;

fn gate() -> CompatibilityGate {
    CompatibilityGate {
        max_situation_band_differences: 0,
        max_route_length_difference: 0,
        max_turn_difference: 0,
    }
}

fn key(threat: ThreatKind, habitat: u8, route_length: u16) -> CandidateKey {
    CandidateKey {
        situation: Situation {
            threat,
            bearing_sector: 1,
            distance_band: 1,
            energy_band: 2,
            stamina_band: 2,
            habitat_class: habitat,
        },
        route: Route {
            refuge_kind: 0,
            length: route_length,
            turns: 1,
            approach_sector: 1,
            terrain: vec![0, 1],
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

#[test]
fn context_aliasing_is_possible_without_threat_identity() {
    let observed = key(ThreatKind::Carnivore, 3, 4);
    let current = key(ThreatKind::Carnivore, 3, 4);
    assert!(compatible(&current, &observed, gate()));
}

#[test]
fn incompatible_species_habitat_and_route_are_rejected() {
    let observed = key(ThreatKind::Carnivore, 3, 4);
    assert!(!compatible(
        &key(ThreatKind::Omnivore, 3, 4),
        &observed,
        gate()
    ));
    assert!(!compatible(
        &key(ThreatKind::Carnivore, 4, 4),
        &observed,
        gate()
    ));
    assert!(!compatible(
        &key(ThreatKind::Carnivore, 3, 5),
        &observed,
        gate()
    ));
    let mut changed_terrain = observed.clone();
    changed_terrain.route.terrain[1] = 9;
    assert!(!compatible(&observed, &changed_terrain, gate()));
}

#[test]
fn caller_must_explicitly_allow_band_generalization() {
    let observed = key(ThreatKind::Carnivore, 3, 4);
    let mut current = observed.clone();
    current.situation.energy_band += 1;
    assert!(!compatible(&current, &observed, gate()));
    assert!(compatible(
        &current,
        &observed,
        CompatibilityGate {
            max_situation_band_differences: 1,
            ..gate()
        }
    ));
}

#[test]
fn censored_episode_does_not_create_or_update_safety_evidence() {
    let mut rows = vec![];
    assert!(!record_outcome(
        &mut rows,
        key(ThreatKind::Carnivore, 3, 4),
        EpisodeEvidence::Censored(CensorReason::OtherAttacker),
        10,
        4,
    ));
    assert!(rows.is_empty());
}

#[test]
fn missing_candidate_support_falls_back_instead_of_assuming_zero_risk() {
    let known = key(ThreatKind::Carnivore, 3, 4);
    let unknown = key(ThreatKind::Carnivore, 3, 5);
    let mut rows = vec![];
    record_outcome(&mut rows, known.clone(), completed(0, 4), 1, 4);
    assert!(rank_candidates(&[known, unknown], &rows, gate(), 1).is_none());
}

#[test]
fn one_observation_cannot_be_reused_for_two_distinct_current_routes() {
    let short = key(ThreatKind::Carnivore, 3, 3);
    let long = key(ThreatKind::Carnivore, 3, 5);
    let mut rows = vec![];
    record_outcome(
        &mut rows,
        key(ThreatKind::Carnivore, 3, 4),
        completed(0, 4),
        1,
        4,
    );
    assert!(
        rank_candidates(
            &[short, long],
            &rows,
            CompatibilityGate {
                max_route_length_difference: 1,
                ..gate()
            },
            1,
        )
        .is_none()
    );
}

#[test]
fn evidence_ranks_safety_before_cost_with_stable_ties() {
    let safe_but_long = key(ThreatKind::Carnivore, 3, 4);
    let damaged_but_short = key(ThreatKind::Carnivore, 3, 3);
    let mut rows = vec![];
    record_outcome(&mut rows, safe_but_long.clone(), completed(0, 40), 1, 4);
    record_outcome(&mut rows, damaged_but_short.clone(), completed(3, 1), 1, 4);
    assert_eq!(
        rows.iter()
            .find(|row| row.key == safe_but_long)
            .unwrap()
            .safe_arrivals,
        1
    );
    assert_eq!(
        rows.iter()
            .find(|row| row.key == damaged_but_short)
            .unwrap()
            .safe_arrivals,
        0
    );
    assert_eq!(
        rank_candidates(&[damaged_but_short, safe_but_long], &rows, gate(), 1),
        Some(vec![1, 0])
    );
}

#[test]
fn retention_is_bounded_and_uses_a_stable_key_tiebreak() {
    let low = key(ThreatKind::Carnivore, 2, 3);
    let high = key(ThreatKind::Carnivore, 4, 3);
    let newer = key(ThreatKind::Carnivore, 3, 3);
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
    let a = key(ThreatKind::Carnivore, 2, 3);
    let b = key(ThreatKind::Carnivore, 4, 3);
    let c = key(ThreatKind::Carnivore, 3, 3);
    let mut rows = vec![
        outcome(a.clone(), 1),
        outcome(b.clone(), 2),
        outcome(c.clone(), 3),
    ];
    assert!(record_outcome(&mut rows, b.clone(), completed(0, 1), 4, 1));
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].key, b);
    assert_eq!(rows[0].completed, 2);
}

#[test]
fn zero_capacity_clears_preloaded_rows() {
    let mut rows = vec![outcome(key(ThreatKind::Carnivore, 3, 3), 1)];
    assert!(!record_outcome(
        &mut rows,
        key(ThreatKind::Carnivore, 3, 4),
        completed(0, 1),
        2,
        0,
    ));
    assert!(rows.is_empty());
}

#[test]
fn aggregate_overflow_freezes_the_entire_row() {
    let key = key(ThreatKind::Carnivore, 3, 3);
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
