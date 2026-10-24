#[path = "support/refuge_residual_candidate.rs"]
mod refuge_residual_candidate;

use refuge_residual_candidate::{
    BaselineCalibration, Closure, CompletedEpisode, MAX_ROUTE_ROWS, MAX_UPDATES, ObservableContext,
    PhysicalVector, PredictionStatus, RefugeKind, ResidualModel, RouteCandidate, RouteKey,
    UpdateResult,
};

fn context() -> ObservableContext {
    ObservableContext {
        subject_health: 70,
        subject_max_health: 100,
        subject_energy: 60,
        subject_max_energy: 100,
        subject_stamina: 50,
        subject_max_stamina: 100,
        threat_distance_steps: 2,
        subject_vision_range: 8,
        threat_power: 10,
        threat_health: Some(40),
        route_steps: 3,
        route_turns: 1,
        estimated_movement_energy: 12,
    }
}

fn key(x: i32) -> RouteKey {
    RouteKey {
        endpoint_x: x,
        endpoint_y: 0,
        refuge_kind: RefugeKind::Burrow,
    }
}

fn baseline(damage: u32, movement: u32, time: u32) -> PhysicalVector {
    PhysicalVector {
        threat_damage: damage,
        paid_movement_energy: movement,
        elapsed_ticks: time,
    }
}

fn candidate(x: i32, order: usize, expected: PhysicalVector) -> RouteCandidate {
    RouteCandidate::from_observation(
        key(x),
        context(),
        Some(expected),
        BaselineCalibration::Uncalibrated,
        order,
    )
}

fn episode(
    owner_id: u64,
    route: RouteKey,
    expected: PhysicalVector,
    actual: PhysicalVector,
    threat_id: u64,
    ended_tick: u64,
) -> CompletedEpisode {
    CompletedEpisode {
        owner_id,
        route,
        features: context().features(),
        baseline: Some(expected),
        actual,
        closure: Closure::Arrived,
        threat_id: Some(threat_id),
        other_damage: 0,
        unexplained_health_loss: 0,
        ended_tick,
    }
}

#[test]
fn default_off_preserves_baseline_choice_and_values_exactly() {
    let model = ResidualModel::new(15);
    let routes = [
        candidate(1, 0, baseline(3, 4, 2)),
        candidate(2, 1, baseline(1, 8, 5)),
    ];
    let result = model.query(15, &routes);
    assert_eq!(result.selected_index, Some(0));
    assert_eq!(result.baseline_index, Some(0));
    assert!(!result.model_enabled);
    for (row, expected) in result.predictions.iter().zip(&routes) {
        assert_eq!(row.baseline, expected.baseline);
        assert_eq!(row.prediction, expected.baseline);
        assert_eq!(row.residual_q16, None);
        assert_eq!(row.status, PredictionStatus::Unknown);
    }
}

#[test]
fn enabled_untrained_and_zero_residual_keep_baseline_order() {
    let owner = 15;
    let expected_a = baseline(9, 9, 9);
    let expected_b = baseline(0, 0, 0);
    let routes = [candidate(1, 0, expected_a), candidate(2, 1, expected_b)];
    let mut model = ResidualModel::new(owner);
    model.set_enabled(true);

    // The supplied baseline order deliberately conflicts with lexicographic
    // physical vector order. An empty model has no evidence to reorder it.
    let empty = model.query(owner, &routes);
    assert_eq!(empty.baseline_index, Some(0));
    assert_eq!(empty.selected_index, Some(0));
    assert_eq!(empty.predictions[0].prediction, Some(expected_a));
    assert_eq!(empty.predictions[1].prediction, Some(expected_b));
    assert!(
        empty
            .predictions
            .iter()
            .all(|row| row.status == PredictionStatus::Unknown)
    );

    // Own completed episodes with exactly baseline outcomes produce zero
    // residuals and still must not reorder the current rule's choice.
    for tick in 1..=4 {
        assert!(matches!(
            model.update(episode(
                owner,
                key(1),
                expected_a,
                expected_a,
                50 + tick,
                tick
            )),
            UpdateResult::Applied { .. }
        ));
        assert!(matches!(
            model.update(episode(
                owner,
                key(2),
                expected_b,
                expected_b,
                60 + tick,
                tick
            )),
            UpdateResult::Applied { .. }
        ));
    }
    let zero = model.query(owner, &routes);
    assert_eq!(zero.baseline_index, Some(0));
    assert_eq!(zero.selected_index, Some(0));
    assert_eq!(zero.predictions[0].prediction, Some(expected_a));
    assert_eq!(zero.predictions[1].prediction, Some(expected_b));
}

#[test]
fn own_adverse_and_undamaged_episodes_shift_diagnostic_route_ranking() {
    let owner = 15;
    let route_a = key(1);
    let route_b = key(2);
    let expected = baseline(2, 4, 3);
    let mut model = ResidualModel::new(owner);
    model.set_enabled(true);

    // These updates enter only through completed own episodes; source threat
    // IDs are provenance and are not part of the feature vector.
    for tick in 1..=12 {
        assert!(matches!(
            model.update(episode(
                owner,
                route_a,
                expected,
                baseline(4, 4, 3),
                100 + tick,
                tick
            )),
            UpdateResult::Applied { .. }
        ));
        assert!(matches!(
            model.update(episode(
                owner,
                route_b,
                expected,
                baseline(0, 4, 3),
                200 + tick,
                tick
            )),
            UpdateResult::Applied { .. }
        ));
    }

    let routes = [candidate(1, 0, expected), candidate(2, 1, expected)];
    let result = model.query(owner, &routes);
    assert_eq!(result.baseline_index, Some(0));
    assert_eq!(result.selected_index, Some(1));
    assert!(matches!(
        result.predictions[0].status,
        PredictionStatus::Provisional { updates: 12 }
    ));
    assert!(matches!(
        result.predictions[1].status,
        PredictionStatus::Provisional { updates: 12 }
    ));
    assert!(
        result.predictions[0].prediction.unwrap().threat_damage
            > result.predictions[1].prediction.unwrap().threat_damage
    );
    assert_eq!(model.provenance().len(), 24);
    assert_ne!(
        model.provenance()[0].threat_id,
        model.provenance()[1].threat_id
    );
    assert_eq!(
        result.predictions[0].baseline_calibration,
        BaselineCalibration::Uncalibrated
    );
}

#[test]
fn unknown_route_uses_zero_residual_without_claiming_safety() {
    let mut model = ResidualModel::new(15);
    model.set_enabled(true);
    let routes = [candidate(1, 0, baseline(7, 5, 4))];
    let result = model.query(15, &routes);
    let row = &result.predictions[0];
    assert_eq!(row.prediction, row.baseline);
    assert_eq!(row.residual_q16, None);
    assert_eq!(row.status, PredictionStatus::Unknown);
}

#[test]
fn calibration_label_is_reported_without_claiming_validation() {
    let calibrated = BaselineCalibration::ExternallyCalibrated {
        identifier: "future-design-token".to_owned(),
    };
    let route = RouteCandidate::from_observation(
        key(1),
        context(),
        Some(baseline(2, 4, 3)),
        calibrated.clone(),
        0,
    );
    let result = ResidualModel::new(15).query(15, &[route]);
    assert_eq!(result.predictions[0].baseline_calibration, calibrated);
    assert_eq!(result.predictions[0].status, PredictionStatus::Unknown);
}

#[test]
fn owner_isolation_and_wrong_owner_updates_fail_closed() {
    let expected = baseline(2, 4, 3);
    let mut first = ResidualModel::new(15);
    first.set_enabled(true);
    let wrong = episode(16, key(1), expected, baseline(0, 4, 3), 37, 1);
    assert_eq!(first.update(wrong), UpdateResult::WrongOwner);
    assert_eq!(first.total_updates(), 0);

    assert!(matches!(
        first.update(episode(15, key(1), expected, baseline(0, 4, 3), 37, 1)),
        UpdateResult::Applied { .. }
    ));
    let second = ResidualModel::new(16);
    let result = second.query(16, &[candidate(1, 0, expected)]);
    assert_eq!(result.predictions[0].status, PredictionStatus::Unknown);
    assert_eq!(result.predictions[0].prediction, Some(expected));
}

#[test]
fn censored_and_frozen_episodes_do_not_mutate_model() {
    let expected = baseline(2, 4, 3);
    let mut model = ResidualModel::new(15);
    assert_eq!(
        model.update(episode(15, key(1), expected, baseline(0, 4, 3), 37, 1)),
        UpdateResult::Disabled
    );
    model.set_enabled(true);

    for (tick, closure) in [
        Closure::ThreatLost,
        Closure::Died,
        Closure::TimedOut,
        Closure::Interrupted,
        Closure::InvalidRoute,
        Closure::Ambiguous,
    ]
    .into_iter()
    .enumerate()
    {
        let mut censored = episode(15, key(1), expected, baseline(0, 4, 3), 37, 1 + tick as u64);
        censored.closure = closure;
        assert_eq!(model.update(censored), UpdateResult::Censored);
    }
    let mut missing_attribution = episode(15, key(1), expected, baseline(0, 4, 3), 37, 20);
    missing_attribution.threat_id = None;
    assert_eq!(model.update(missing_attribution), UpdateResult::Censored);
    let mut other_damage = episode(15, key(1), expected, baseline(0, 4, 3), 37, 2);
    other_damage.other_damage = 1;
    assert_eq!(model.update(other_damage), UpdateResult::Censored);
    let mut unexplained = episode(15, key(1), expected, baseline(0, 4, 3), 37, 3);
    unexplained.unexplained_health_loss = 1;
    assert_eq!(model.update(unexplained), UpdateResult::Censored);
    assert_eq!(model.total_updates(), 0);
    assert!(model.provenance().is_empty());

    model.set_frozen(true);
    let before = model.clone();
    assert_eq!(
        model.update(episode(15, key(1), expected, baseline(0, 4, 3), 37, 4)),
        UpdateResult::Frozen
    );
    assert_eq!(model, before);
}

#[test]
fn missing_features_and_baseline_do_not_train() {
    let expected = baseline(2, 4, 3);
    let mut model = ResidualModel::new(15);
    model.set_enabled(true);
    let mut absent = episode(15, key(1), expected, baseline(0, 4, 3), 37, 1);
    absent.features = None;
    assert_eq!(
        model.update(absent),
        UpdateResult::MissingFeaturesOrBaseline
    );
    let mut no_baseline = episode(15, key(1), expected, baseline(0, 4, 3), 37, 2);
    no_baseline.baseline = None;
    assert_eq!(
        model.update(no_baseline),
        UpdateResult::MissingFeaturesOrBaseline
    );
    assert_eq!(model.total_updates(), 0);

    let mut candidate = candidate(1, 0, expected);
    candidate.features = None;
    let query = model.query(15, &[candidate]);
    assert_eq!(query.predictions[0].prediction, Some(expected));
    assert_eq!(query.predictions[0].status, PredictionStatus::Unknown);
}

#[test]
fn absent_perceived_threat_health_falls_back_to_unknown() {
    let mut observed = context();
    observed.threat_health = None;
    assert_eq!(observed.features(), None);
    let route = RouteCandidate::from_observation(
        key(1),
        observed,
        Some(baseline(2, 4, 3)),
        BaselineCalibration::Uncalibrated,
        0,
    );
    let mut model = ResidualModel::new(15);
    model.set_enabled(true);
    let result = model.query(15, &[route]);
    assert_eq!(result.predictions[0].status, PredictionStatus::Unknown);
    assert_eq!(result.predictions[0].prediction, Some(baseline(2, 4, 3)));
}

#[test]
fn route_and_update_caps_are_transactional() {
    let expected = baseline(0, 1, 1);
    let mut model = ResidualModel::new(15);
    model.set_enabled(true);
    for index in 0..MAX_ROUTE_ROWS {
        assert!(matches!(
            model.update(episode(
                15,
                key(index as i32),
                expected,
                expected,
                100 + index as u64,
                index as u64
            )),
            UpdateResult::Applied { .. }
        ));
    }
    assert_eq!(model.route_count(), MAX_ROUTE_ROWS);
    let before = model.clone();
    assert_eq!(
        model.update(episode(15, key(99), expected, expected, 999, 99)),
        UpdateResult::RouteLimit
    );
    assert_eq!(model, before);

    let mut capped = ResidualModel::new(16);
    capped.set_enabled(true);
    for tick in 0..MAX_UPDATES {
        assert!(matches!(
            capped.update(episode(16, key(1), expected, expected, 5, u64::from(tick),)),
            UpdateResult::Applied { .. }
        ));
    }
    let before = capped.clone();
    assert_eq!(capped.total_updates(), MAX_UPDATES);
    assert_eq!(
        capped.update(episode(
            16,
            key(1),
            expected,
            expected,
            5,
            u64::from(MAX_UPDATES),
        )),
        UpdateResult::UpdateLimit
    );
    assert_eq!(capped, before);
    assert_eq!(capped.provenance().len(), 32);
}

#[test]
fn numeric_clipping_and_invalid_values_stay_bounded() {
    let mut model = ResidualModel::new(15);
    model.set_enabled(true);
    let baseline = PhysicalVector::default();
    assert!(matches!(
        model.update(episode(
            15,
            key(1),
            baseline,
            PhysicalVector {
                threat_damage: 1_000_000,
                paid_movement_energy: 1_000_000,
                elapsed_ticks: 1_000_000,
            },
            37,
            1,
        )),
        UpdateResult::Applied {
            clipped_components: 3
        }
    ));
    let query = model.query(15, &[candidate(1, 0, baseline)]);
    let predicted = query.predictions[0].prediction.unwrap();
    assert!(predicted.threat_damage <= 256);
    assert!(predicted.paid_movement_energy <= 256);
    assert!(predicted.elapsed_ticks <= 256);

    let before = model.clone();
    let invalid = episode(
        15,
        key(1),
        baseline,
        PhysicalVector {
            threat_damage: 1_000_001,
            paid_movement_energy: 0,
            elapsed_ticks: 0,
        },
        37,
        2,
    );
    assert_eq!(model.update(invalid), UpdateResult::InvalidPhysicalValue);
    assert_eq!(model, before);
}
