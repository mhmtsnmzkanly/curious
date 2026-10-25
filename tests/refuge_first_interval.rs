#[path = "../examples/support/refuge_first_interval_candidate.rs"]
mod refuge_first_interval_candidate;

use refuge_first_interval_candidate::*;

fn sample_key(endpoint: (i32, i32), first_step_sector: u8) -> FirstIntervalKey {
    FirstIntervalKey {
        subject_species: Species::Herbivore,
        threat_species: Species::Carnivore,
        threat_bearing_sector: 1,
        threat_distance_steps: 1,
        first_step_sector,
        endpoint,
        subject_speed: 3,
    }
}

#[test]
fn test_1_completed_first_interval_positive_damage_ranks_even_if_later_censored() {
    let mut learner = FirstIntervalLearner::with_enabled(42, true, true);
    let key = sample_key((5, 5), 2);

    let outcome = FirstIntervalOutcome::CompletedFirstInterval {
        threat_damage: 20,
        paid_movement_energy: 3,
        later_full_route_censored: true, // Route was interrupted later on tick 2+
    };

    let result = learner.record_episode(42, key, outcome, 10);
    assert_eq!(
        result,
        RecordResult::Applied {
            updated_existing: false
        }
    );

    assert_eq!(learner.rows.len(), 1);
    let row = &learner.rows[0];
    assert_eq!(row.completed_count, 1);
    assert_eq!(row.total_threat_damage, 20);
    assert_eq!(row.total_movement_energy, 3);
    assert_eq!(row.later_censored_count, 1);
    assert_eq!(row.last_updated_tick, 10);

    // Candidate ranking should use this known first-interval support
    let ranking = learner.rank_first_interval_candidates(&[key], 12);
    assert_eq!(ranking.estimates.len(), 1);
    assert_eq!(ranking.estimates[0].status, EstimateStatus::Known);
    assert_eq!(ranking.estimates[0].completed_samples, 1);
    assert_eq!(ranking.estimates[0].expected_threat_damage(), Some(20.0));
    assert_eq!(ranking.estimates[0].expected_movement_energy(), Some(3.0));
    assert_eq!(ranking.estimates[0].later_censored_samples, 1);
    assert_eq!(ranking.ranked_indices, Some(vec![0]));
}

#[test]
fn test_2_first_interval_censoring_never_updates_or_fabricates_zero_damage() {
    let mut learner = FirstIntervalLearner::with_enabled(42, true, true);
    let key = sample_key((10, 10), 0);

    let censor_reasons = [
        FirstIntervalCensorReason::ThirdPartyDamage,
        FirstIntervalCensorReason::UnattributedHealthLoss,
        FirstIntervalCensorReason::PreResolutionInterrupt,
        FirstIntervalCensorReason::MissingThreatAttribution,
    ];

    for (tick, reason) in censor_reasons.iter().enumerate() {
        let outcome = FirstIntervalOutcome::CensoredFirstInterval(*reason);
        let result = learner.record_episode(42, key, outcome, tick as u64);
        assert_eq!(result, RecordResult::Censored(*reason));
        assert!(
            learner.rows.is_empty(),
            "Censored first intervals must never create or update rows"
        );
    }

    // Query must return Unknown; zero damage must NOT be fabricated
    let ranking = learner.rank_first_interval_candidates(&[key], 5);
    assert_eq!(ranking.estimates[0].status, EstimateStatus::Unknown);
    assert_eq!(ranking.estimates[0].expected_threat_damage(), None);
    assert_eq!(ranking.ranked_indices, None);
}

#[test]
fn test_3_missing_candidate_support_returns_unknown_and_fails_closed() {
    let mut learner = FirstIntervalLearner::with_enabled(100, true, true);
    let key_known = sample_key((1, 1), 0);
    let key_unknown = sample_key((2, 2), 1);

    // Record only key_known
    learner.record_episode(
        100,
        key_known,
        FirstIntervalOutcome::CompletedFirstInterval {
            threat_damage: 15,
            paid_movement_energy: 2,
            later_full_route_censored: false,
        },
        1,
    );

    // Query both candidates
    let candidates = [key_known, key_unknown];
    let ranking = learner.rank_first_interval_candidates(&candidates, 2);

    assert_eq!(ranking.estimates[0].status, EstimateStatus::Known);
    assert_eq!(ranking.estimates[0].expected_threat_damage(), Some(15.0));

    assert_eq!(ranking.estimates[1].status, EstimateStatus::Unknown);
    assert_eq!(ranking.estimates[1].expected_threat_damage(), None);

    // When any candidate is Unknown, ranking must fail closed (None) rather than guessing
    assert_eq!(
        ranking.ranked_indices, None,
        "Ranking must return None when any candidate support is missing"
    );
}

#[test]
fn test_4_strict_owner_id_isolation_rejects_foreign_episodes() {
    let mut learner = FirstIntervalLearner::with_enabled(10, true, true);
    let key = sample_key((3, 3), 4);

    let foreign_result = learner.record_episode(
        99, // Foreign owner
        key,
        FirstIntervalOutcome::CompletedFirstInterval {
            threat_damage: 10,
            paid_movement_energy: 1,
            later_full_route_censored: false,
        },
        5,
    );

    assert_eq!(foreign_result, RecordResult::WrongOwner);
    assert!(learner.rows.is_empty());
    assert_eq!(learner.total_samples_recorded, 0);

    // Matching owner succeeds
    let valid_result = learner.record_episode(
        10,
        key,
        FirstIntervalOutcome::CompletedFirstInterval {
            threat_damage: 10,
            paid_movement_energy: 1,
            later_full_route_censored: false,
        },
        5,
    );

    assert_eq!(
        valid_result,
        RecordResult::Applied {
            updated_existing: false
        }
    );
    assert_eq!(learner.rows.len(), 1);
}

#[test]
fn test_5_disabled_and_frozen_contracts() {
    let key = sample_key((0, 5), 2);

    // Disabled mode
    let mut disabled_learner = FirstIntervalLearner::new(1); // default-off: enabled=false, update_enabled=false
    assert!(!disabled_learner.enabled);
    assert!(!disabled_learner.update_enabled);

    let rec_res = disabled_learner.record_episode(
        1,
        key,
        FirstIntervalOutcome::CompletedFirstInterval {
            threat_damage: 5,
            paid_movement_energy: 1,
            later_full_route_censored: false,
        },
        1,
    );
    assert_eq!(rec_res, RecordResult::Disabled);

    let rank_res = disabled_learner.rank_first_interval_candidates(&[key], 1);
    assert_eq!(rank_res.estimates[0].status, EstimateStatus::Unknown);
    assert_eq!(rank_res.ranked_indices, None);

    // Frozen mode: enabled=true, update_enabled=false
    let mut frozen_learner = FirstIntervalLearner::with_enabled(1, true, true);
    frozen_learner.record_episode(
        1,
        key,
        FirstIntervalOutcome::CompletedFirstInterval {
            threat_damage: 5,
            paid_movement_energy: 1,
            later_full_route_censored: false,
        },
        1,
    );

    // Freeze updates
    frozen_learner.update_enabled = false;

    // Further record attempts rejected
    let freeze_rec = frozen_learner.record_episode(
        1,
        key,
        FirstIntervalOutcome::CompletedFirstInterval {
            threat_damage: 50,
            paid_movement_energy: 10,
            later_full_route_censored: false,
        },
        2,
    );
    assert_eq!(freeze_rec, RecordResult::Frozen);

    // But query and ranking still function with frozen knowledge
    let freeze_rank = frozen_learner.rank_first_interval_candidates(&[key], 2);
    assert_eq!(freeze_rank.estimates[0].status, EstimateStatus::Known);
    assert_eq!(freeze_rank.estimates[0].completed_samples, 1);
    assert_eq!(freeze_rank.estimates[0].expected_threat_damage(), Some(5.0));
    assert_eq!(freeze_rank.ranked_indices, Some(vec![0]));
}

#[test]
fn test_6_ttl_expiration_and_bounded_row_eviction() {
    let mut learner = FirstIntervalLearner::with_enabled(1, true, true);
    let key1 = sample_key((1, 0), 0);

    // Record at tick 10
    learner.record_episode(
        1,
        key1,
        FirstIntervalOutcome::CompletedFirstInterval {
            threat_damage: 10,
            paid_movement_energy: 1,
            later_full_route_censored: false,
        },
        10,
    );

    // Tick 110 (delta = 100 <= TTL) -> valid
    let r1 = learner.rank_first_interval_candidates(&[key1], 110);
    assert_eq!(r1.estimates[0].status, EstimateStatus::Known);

    // Tick 111 (delta = 101 > TTL) -> expired
    let r2 = learner.rank_first_interval_candidates(&[key1], 111);
    assert_eq!(r2.estimates[0].status, EstimateStatus::Unknown);
    assert_eq!(r2.ranked_indices, None);

    // Fill table up to MAX_FIRST_INTERVAL_ROWS (8 rows)
    let mut keys = vec![key1];
    for i in 1..MAX_FIRST_INTERVAL_ROWS {
        let k = sample_key((i as i32 + 2, 0), 0);
        keys.push(k);
        learner.record_episode(
            1,
            k,
            FirstIntervalOutcome::CompletedFirstInterval {
                threat_damage: 5,
                paid_movement_energy: 1,
                later_full_route_censored: false,
            },
            20 + i as u64,
        );
    }
    assert_eq!(learner.rows.len(), MAX_FIRST_INTERVAL_ROWS);

    // Insert 9th key at tick 120 (key1 is expired at tick 120 because 120 - 10 = 110 > 100)
    let key9 = sample_key((99, 99), 0);
    learner.record_episode(
        1,
        key9,
        FirstIntervalOutcome::CompletedFirstInterval {
            threat_damage: 2,
            paid_movement_energy: 1,
            later_full_route_censored: false,
        },
        120,
    );

    // Size must remain bounded to exactly MAX_FIRST_INTERVAL_ROWS
    assert_eq!(learner.rows.len(), MAX_FIRST_INTERVAL_ROWS);
    // key1 (expired) was evicted
    assert!(!learner.rows.iter().any(|r| r.key == key1));
    assert!(learner.rows.iter().any(|r| r.key == key9));
}

#[test]
fn test_7_counter_overflow_atomic_refusal() {
    let mut learner = FirstIntervalLearner::with_enabled(1, true, true);
    let key = sample_key((4, 4), 3);

    // Single value exceeding MAX_INTERVAL_ACCOUNT rejected immediately
    let huge_res = learner.record_episode(
        1,
        key,
        FirstIntervalOutcome::CompletedFirstInterval {
            threat_damage: MAX_INTERVAL_ACCOUNT + 1,
            paid_movement_energy: 1,
            later_full_route_censored: false,
        },
        1,
    );
    assert_eq!(huge_res, RecordResult::InvalidOutcomeValue);

    // Setup an existing row near boundary
    let setup_res = learner.record_episode(
        1,
        key,
        FirstIntervalOutcome::CompletedFirstInterval {
            threat_damage: MAX_INTERVAL_ACCOUNT - 10,
            paid_movement_energy: 5,
            later_full_route_censored: false,
        },
        1,
    );
    assert_eq!(
        setup_res,
        RecordResult::Applied {
            updated_existing: false
        }
    );
    assert_eq!(learner.rows.len(), 1);

    // Update that would overflow damage limit is atomically refused
    let overflow_res = learner.record_episode(
        1,
        key,
        FirstIntervalOutcome::CompletedFirstInterval {
            threat_damage: 20, // (MAX - 10) + 20 > MAX
            paid_movement_energy: 1,
            later_full_route_censored: false,
        },
        2,
    );
    assert_eq!(overflow_res, RecordResult::AccountLimitExceeded);

    // Verify row was NOT corrupted by failed update
    let row = &learner.rows[0];
    assert_eq!(row.completed_count, 1);
    assert_eq!(row.total_threat_damage, MAX_INTERVAL_ACCOUNT - 10);
    assert_eq!(row.last_updated_tick, 1);
}

#[test]
fn test_8_candidate_ranking_criteria_ordering_damage_then_energy_then_baseline() {
    let mut learner = FirstIntervalLearner::with_enabled(1, true, true);

    let key_high_dmg = sample_key((1, 0), 0); // 20 dmg, 5 energy
    let key_high_nrg = sample_key((2, 0), 1); // 10 dmg, 10 energy
    let key_best1 = sample_key((3, 0), 2);    // 10 dmg, 2 energy (candidate 2)
    let key_best2 = sample_key((4, 0), 3);    // 10 dmg, 2 energy (candidate 3, tied with 2)

    learner.record_episode(
        1,
        key_high_dmg,
        FirstIntervalOutcome::CompletedFirstInterval {
            threat_damage: 20,
            paid_movement_energy: 5,
            later_full_route_censored: false,
        },
        1,
    );
    learner.record_episode(
        1,
        key_high_nrg,
        FirstIntervalOutcome::CompletedFirstInterval {
            threat_damage: 10,
            paid_movement_energy: 10,
            later_full_route_censored: false,
        },
        1,
    );
    learner.record_episode(
        1,
        key_best1,
        FirstIntervalOutcome::CompletedFirstInterval {
            threat_damage: 10,
            paid_movement_energy: 2,
            later_full_route_censored: false,
        },
        1,
    );
    learner.record_episode(
        1,
        key_best2,
        FirstIntervalOutcome::CompletedFirstInterval {
            threat_damage: 10,
            paid_movement_energy: 2,
            later_full_route_censored: false,
        },
        1,
    );

    let candidates = [key_high_dmg, key_high_nrg, key_best1, key_best2];
    let ranking = learner.rank_first_interval_candidates(&candidates, 2);

    // Expected order:
    // candidate 2 (best1: dmg 10, nrg 2)
    // candidate 3 (best2: dmg 10, nrg 2, baseline tie-break 2 < 3)
    // candidate 1 (high_nrg: dmg 10, nrg 10)
    // candidate 0 (high_dmg: dmg 20, nrg 5)
    assert_eq!(ranking.ranked_indices, Some(vec![2, 3, 1, 0]));
}

#[test]
fn test_9_audit_visibility_tracks_later_censored_samples_in_aggregates() {
    let mut learner = FirstIntervalLearner::with_enabled(1, true, true);
    let key = sample_key((1, 1), 0);

    // Episode 1: completed, later full route arrived normally
    learner.record_episode(
        1,
        key,
        FirstIntervalOutcome::CompletedFirstInterval {
            threat_damage: 10,
            paid_movement_energy: 2,
            later_full_route_censored: false,
        },
        1,
    );

    // Episode 2: completed first interval, but route was interrupted later
    learner.record_episode(
        1,
        key,
        FirstIntervalOutcome::CompletedFirstInterval {
            threat_damage: 30,
            paid_movement_energy: 4,
            later_full_route_censored: true,
        },
        2,
    );

    let ranking = learner.rank_first_interval_candidates(&[key], 3);
    let est = &ranking.estimates[0];
    assert_eq!(est.completed_samples, 2);
    assert_eq!(est.later_censored_samples, 1);
    assert_eq!(est.total_threat_damage, 40);
    assert_eq!(est.expected_threat_damage(), Some(20.0));
    assert_eq!(est.expected_movement_energy(), Some(3.0));
}
