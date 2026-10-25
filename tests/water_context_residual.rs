#[path = "../examples/support/water_context_residual_candidate.rs"]
mod water_context_residual_candidate;

use water_context_residual_candidate::{
    choose_water_candidate, compute_actual_score, ResidualStatus, WaterAttemptOutcome,
    WaterCandidateInput, WaterCensorReason, WaterContextKey, WaterContextResidualTable,
    MAX_CONTEXT_ROWS, MAX_WINDOW_SAMPLES, WATER_RESIDUAL_TTL_TICKS,
};

#[test]
fn test_compute_actual_score_formula() {
    assert_eq!(WATER_RESIDUAL_TTL_TICKS, 100);
    // water=20, movement=10, ticks=5 => ((40 - 15)*100)/5 = 2500/5 = 500
    assert_eq!(compute_actual_score(20, 10, 5), 500);
    // water=0, movement=10, ticks=5 => ((-15)*100)/5 = -300
    assert_eq!(compute_actual_score(0, 10, 5), -300);
}

#[test]
fn test_adaptation_to_dry_scheduled_source() {
    let mut table = WaterContextResidualTable::new(true, true);

    let pos_a = (10, 0);
    let pos_b = (0, 10);
    let speed = 2;
    let cost = 1;
    let key_a = WaterContextKey::new(pos_a, speed, cost).unwrap();

    let candidates = vec![
        WaterCandidateInput {
            source_pos: pos_a,
            choice_baseline: 400,
            steps_len: 5,
        },
        WaterCandidateInput {
            source_pos: pos_b,
            choice_baseline: 450,
            steps_len: 5,
        },
    ];

    // Initially, Candidate B (450) beats Candidate A (400)
    assert_eq!(choose_water_candidate(&table, &candidates, speed, cost, 1), Some(1));

    // 6 successful drinks at Source A:
    // water=20, move_energy=10, ticks=5 => actual = ((20*2 - 10 - 5)*100)/5 = 500.
    // residual = 500 - 400 = +100.
    for i in 1..=6 {
        let tick = i * 10;
        let outcome = WaterAttemptOutcome::Reached {
            water_gained: 20,
            movement_energy: 10,
            ticks: 5,
            choice_baseline: 400,
        };
        assert!(table.record_outcome(key_a, outcome, tick));
    }

    // Now Source A has residual +100, total = 400 + 100 = 500 > 450. Preference shifts to Source A.
    assert_eq!(choose_water_candidate(&table, &candidates, speed, cost, 65), Some(0));

    // Now Source A dries up (scheduled resource exhausted).
    // Creature visits Source A twice and finds missing resource (verified).
    // water=0, move_energy=10, ticks=5 => actual = ((0 - 10 - 5)*100)/5 = -300.
    // residual = -300 - 400 = -700.
    for i in 7..=8 {
        let tick = i * 10;
        let outcome = WaterAttemptOutcome::FailedMissingResource {
            movement_energy: 10,
            ticks: 5,
            choice_baseline: 400,
            target_verified: true,
        };
        assert!(table.record_outcome(key_a, outcome, tick));
    }

    // Recency window has 4 of +100 and 2 of -700.
    // window sum = 400 - 1400 = -1000.
    // window mean = -1000 / 6 = -166.
    // Source A total = 400 + (-166) = 234 < 450.
    // The recency-windowed learner quickly shifts preference back to Source B!
    assert_eq!(choose_water_candidate(&table, &candidates, speed, cost, 85), Some(1));

    // Compare with an unweighted lifetime average:
    // If we had averaged all 8 visits equally: (6 * 100 + 2 * (-700)) / 8 = -800 / 8 = -100.
    // And if there had been, say, 16 prior visits of +100: (1600 - 1400) / 18 = +11 (still positive!).
    // In our bounded recency window (size 8), after 8 failures, all old +100 visits are flushed.
    for i in 9..=14 {
        let tick = i * 10;
        let outcome = WaterAttemptOutcome::FailedMissingResource {
            movement_energy: 10,
            ticks: 5,
            choice_baseline: 400,
            target_verified: true,
        };
        assert!(table.record_outcome(key_a, outcome, tick));
    }
    let row = table.rows.get(&key_a).unwrap();
    assert_eq!(row.window.len(), MAX_WINDOW_SAMPLES);
    assert!(row.window.iter().all(|&r| r == -700));
    assert_eq!(row.window_mean(), Some(-700));
}

#[test]
fn test_ttl_expiration_to_unknown() {
    let mut table = WaterContextResidualTable::new(true, true);
    let key = WaterContextKey::new((5, 5), 2, 1).unwrap();

    let outcome = WaterAttemptOutcome::Reached {
        water_gained: 20,
        movement_energy: 10,
        ticks: 5,
        choice_baseline: 400,
    };
    assert!(table.record_outcome(key, outcome, 10));

    // At tick 50 (elapsed 40 <= 100): fresh
    match table.query_residual(&key, 50) {
        ResidualStatus::Known { window_mean, samples } => {
            assert_eq!(window_mean, 100);
            assert_eq!(samples, 1);
        }
        ResidualStatus::Unknown => panic!("Expected fresh residual"),
    }

    // At tick 110 (elapsed 100 <= 100): still valid
    assert!(matches!(table.query_residual(&key, 110), ResidualStatus::Known { .. }));

    // At tick 111 (elapsed 101 > 100): expired!
    assert_eq!(table.query_residual(&key, 111), ResidualStatus::Unknown);

    // Candidate ranking at tick 111 falls back to choice_baseline without adjustment
    let candidates = vec![
        WaterCandidateInput {
            source_pos: (5, 5),
            choice_baseline: 400,
            steps_len: 5,
        },
        WaterCandidateInput {
            source_pos: (20, 20),
            choice_baseline: 450,
            steps_len: 5,
        },
    ];
    // With expired residual, Candidate 1 (450) wins over Candidate 0 (400 + 0)
    assert_eq!(choose_water_candidate(&table, &candidates, 2, 1, 111), Some(1));

    // Next update at tick 120 prunes the expired row
    let key_b = WaterContextKey::new((20, 20), 2, 1).unwrap();
    assert!(table.record_outcome(key_b, outcome, 120));
    assert!(!table.rows.contains_key(&key));
    assert!(table.rows.contains_key(&key_b));
}

#[test]
fn test_body_context_isolation() {
    let mut table = WaterContextResidualTable::new(true, true);
    let pos = (8, 8);
    let key_fast = WaterContextKey::new(pos, 3, 1).unwrap();
    let key_heavy = WaterContextKey::new(pos, 4, 2).unwrap();

    let outcome = WaterAttemptOutcome::Reached {
        water_gained: 20,
        movement_energy: 10,
        ticks: 5,
        choice_baseline: 300,
    };
    assert!(table.record_outcome(key_fast, outcome, 10));

    // Query under key_fast is Known
    assert!(matches!(table.query_residual(&key_fast, 20), ResidualStatus::Known { .. }));

    // Query under key_heavy is Unknown (isolated body context)
    assert_eq!(table.query_residual(&key_heavy, 20), ResidualStatus::Unknown);

    // Verify key bounds validation
    assert!(WaterContextKey::new(pos, 0, 1).is_none());
    assert!(WaterContextKey::new(pos, 33, 1).is_none());
    assert!(WaterContextKey::new(pos, 3, 0).is_none());
    assert!(WaterContextKey::new(pos, 3, 33).is_none());
    assert!(WaterContextKey::new((1_000_001, 0), 3, 1).is_none());
    assert!(WaterContextKey::new((0, -1_000_001), 3, 1).is_none());
}

#[test]
fn test_censoring_never_updates() {
    let mut table = WaterContextResidualTable::new(true, true);
    let key = WaterContextKey::new((1, 1), 2, 1).unwrap();

    let censor_reasons = [
        WaterCensorReason::Interrupted,
        WaterCensorReason::TimedOut,
        WaterCensorReason::Died,
        WaterCensorReason::MismatchedDrink,
        WaterCensorReason::Overflowed,
    ];

    for reason in censor_reasons {
        assert!(!table.record_outcome(key, WaterAttemptOutcome::Censored(reason), 10));
    }

    // Unverified FailedMissingResource is treated as censored
    assert!(!table.record_outcome(
        key,
        WaterAttemptOutcome::FailedMissingResource {
            movement_energy: 10,
            ticks: 5,
            choice_baseline: 100,
            target_verified: false,
        },
        10
    ));

    // Out-of-bounds parameters are rejected
    assert!(!table.record_outcome(
        key,
        WaterAttemptOutcome::Reached {
            water_gained: 101, // exceeds MAX_WATER_AMOUNT = 100
            movement_energy: 10,
            ticks: 5,
            choice_baseline: 100,
        },
        10
    ));
    assert!(!table.record_outcome(
        key,
        WaterAttemptOutcome::Reached {
            water_gained: 10,
            movement_energy: 1_000_001, // exceeds MAX_MOVEMENT_ENERGY
            ticks: 5,
            choice_baseline: 100,
        },
        10
    ));
    assert!(!table.record_outcome(
        key,
        WaterAttemptOutcome::Reached {
            water_gained: 10,
            movement_energy: 10,
            ticks: 0, // ticks must be >= 1
            choice_baseline: 100,
        },
        10
    ));
    assert!(!table.record_outcome(
        key,
        WaterAttemptOutcome::Reached {
            water_gained: 10,
            movement_energy: 10,
            ticks: 31, // exceeds MAX_EPISODE_TICKS = 30
            choice_baseline: 100,
        },
        10
    ));
    assert!(!table.record_outcome(
        key,
        WaterAttemptOutcome::Reached {
            water_gained: 10,
            movement_energy: 10,
            ticks: 5,
            choice_baseline: 105_000, // exceeds MAX_CHOICE_BASELINE = 104_000
        },
        10
    ));

    // Entire table remains completely empty
    assert!(table.rows.is_empty());
}

#[test]
fn test_disabled_and_frozen_modes() {
    let mut table = WaterContextResidualTable::default(); // enabled = false, update_enabled = true
    let key = WaterContextKey::new((3, 3), 2, 1).unwrap();
    let outcome = WaterAttemptOutcome::Reached {
        water_gained: 20,
        movement_energy: 10,
        ticks: 5,
        choice_baseline: 400,
    };

    // When disabled, record_outcome returns false and table stays empty
    assert!(!table.record_outcome(key, outcome, 10));
    assert!(table.rows.is_empty());
    assert_eq!(table.query_residual(&key, 10), ResidualStatus::Unknown);

    let candidates = vec![
        WaterCandidateInput {
            source_pos: (3, 3),
            choice_baseline: 400,
            steps_len: 5,
        },
        WaterCandidateInput {
            source_pos: (4, 4),
            choice_baseline: 420,
            steps_len: 5,
        },
    ];
    // Disabled is a strict passthrough of baseline ranking
    assert_eq!(choose_water_candidate(&table, &candidates, 2, 1, 10), Some(1));

    // Enable table and record an outcome
    table.enabled = true;
    assert!(table.record_outcome(key, outcome, 10));
    assert_eq!(table.rows.len(), 1);

    // Freeze updates
    table.update_enabled = false;
    let new_outcome = WaterAttemptOutcome::Reached {
        water_gained: 30,
        movement_energy: 5,
        ticks: 3,
        choice_baseline: 400,
    };
    assert!(!table.record_outcome(key, new_outcome, 20));

    // Existing row was not updated
    let row = table.rows.get(&key).unwrap();
    assert_eq!(row.total_visits, 1);
    assert_eq!(row.window.len(), 1);
    assert_eq!(row.last_observed_tick, 10);

    // Frozen residual is still queried and used
    assert!(matches!(table.query_residual(&key, 20), ResidualStatus::Known { .. }));
}

#[test]
fn test_bounded_eviction_32_rows() {
    let mut table = WaterContextResidualTable::new(true, true);

    // Insert 32 distinct keys at ticks 1..=32
    for i in 1..=MAX_CONTEXT_ROWS {
        let key = WaterContextKey::new((i as i32, 0), 2, 1).unwrap();
        let outcome = WaterAttemptOutcome::Reached {
            water_gained: 20,
            movement_energy: 10,
            ticks: 5,
            choice_baseline: 400,
        };
        assert!(table.record_outcome(key, outcome, i));
    }
    assert_eq!(table.rows.len(), MAX_CONTEXT_ROWS);

    // Key 1 (inserted at tick 1) is currently the oldest
    let oldest_key = WaterContextKey::new((1, 0), 2, 1).unwrap();
    assert!(table.rows.contains_key(&oldest_key));

    // Insert 33rd key at tick 33
    let key_33 = WaterContextKey::new((33, 0), 2, 1).unwrap();
    let outcome = WaterAttemptOutcome::Reached {
        water_gained: 20,
        movement_energy: 10,
        ticks: 5,
        choice_baseline: 400,
    };
    assert!(table.record_outcome(key_33, outcome, 33));

    // Length is still bounded at 32
    assert_eq!(table.rows.len(), MAX_CONTEXT_ROWS);
    // Key 1 was evicted
    assert!(!table.rows.contains_key(&oldest_key));
    // Key 33 is present
    assert!(table.rows.contains_key(&key_33));
}

#[test]
fn test_bounded_window_ring_buffer_8_samples() {
    let mut table = WaterContextResidualTable::new(true, true);
    let key = WaterContextKey::new((2, 2), 2, 1).unwrap();

    // Push 12 outcomes for the same key
    for i in 1..=12 {
        let outcome = WaterAttemptOutcome::Reached {
            water_gained: 20,
            movement_energy: 10,
            ticks: 5,
            choice_baseline: 500 - (i as i64 * 10),
        };
        assert!(table.record_outcome(key, outcome, i));
    }

    let row = table.rows.get(&key).unwrap();
    assert_eq!(row.window.len(), MAX_WINDOW_SAMPLES);
    assert_eq!(row.total_visits, 12);

    // Actual score for water=20, move_energy=10, ticks=5 is:
    // ((20*2 - 10 - 5)*100)/5 = 500.
    // Residual for visit i is: 500 - (500 - i*10) = i * 10.
    // The 8 remaining samples in the ring buffer must be visits 5..=12:
    let expected: Vec<i64> = (5..=12).map(|i| i * 10).collect();
    let actual: Vec<i64> = row.window.iter().copied().collect();
    assert_eq!(actual, expected);

    let expected_sum: i64 = expected.iter().sum();
    assert_eq!(row.window_mean(), Some(expected_sum / 8));
}

#[test]
fn test_candidate_ranking_and_tie_breaking() {
    let mut table = WaterContextResidualTable::new(true, true);
    let pos_a = (1, 1);
    let pos_b = (2, 2);
    let key_b = WaterContextKey::new(pos_b, 2, 1).unwrap();

    // Give pos_b a +100 residual
    let outcome = WaterAttemptOutcome::Reached {
        water_gained: 20,
        movement_energy: 10,
        ticks: 5,
        choice_baseline: 400,
    };
    assert!(table.record_outcome(key_b, outcome, 10));

    // Candidate 0: baseline 500, steps 10, residual 0 => total 500
    // Candidate 1: baseline 400, steps 5, residual +100 => total 500
    // Shorter path (steps 5 < 10) wins!
    let candidates = vec![
        WaterCandidateInput {
            source_pos: pos_a,
            choice_baseline: 500,
            steps_len: 10,
        },
        WaterCandidateInput {
            source_pos: pos_b,
            choice_baseline: 400,
            steps_len: 5,
        },
    ];
    assert_eq!(choose_water_candidate(&table, &candidates, 2, 1, 20), Some(1));

    // When total_score and steps_len match, smaller index wins (Reverse(index) in max_by_key)
    let pos_c = (3, 3);
    let key_c = WaterContextKey::new(pos_c, 2, 1).unwrap();
    assert!(table.record_outcome(key_c, outcome, 10));

    let tied_candidates = vec![
        WaterCandidateInput {
            source_pos: pos_b,
            choice_baseline: 400,
            steps_len: 5,
        },
        WaterCandidateInput {
            source_pos: pos_c,
            choice_baseline: 400,
            steps_len: 5,
        },
    ];
    assert_eq!(choose_water_candidate(&table, &tied_candidates, 2, 1, 20), Some(0));
}
