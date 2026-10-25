#[path = "../examples/support/local_coordination_candidate.rs"]
mod local_coordination_candidate;

use local_coordination_candidate::*;

#[test]
fn unpaid_or_fraudulent_alarm_calls_are_rejected_by_receiver() {
    let receiver = LocalAgent::new(1, LocalCoord::new(0, 0), 100, 100, 5);

    // Case 1: is_paid = false
    let unpaid_alarm = VerifiedAlarmEvent {
        sender_id: 2,
        sender_pos: LocalCoord::new(2, 0),
        threat_sector: 2,
        tick: 1,
        paid_energy: 10,
        paid_stamina: 5,
        is_paid: false,
    };
    assert_eq!(
        process_local_alarm(&receiver, &unpaid_alarm, 1),
        AlarmPerception::IgnoredUnpaidOrFraudulent
    );

    // Case 2: is_paid = true but 0 energy and 0 stamina paid
    let zero_paid_alarm = VerifiedAlarmEvent {
        sender_id: 2,
        sender_pos: LocalCoord::new(2, 0),
        threat_sector: 2,
        tick: 1,
        paid_energy: 0,
        paid_stamina: 0,
        is_paid: true,
    };
    assert_eq!(
        process_local_alarm(&receiver, &zero_paid_alarm, 1),
        AlarmPerception::IgnoredUnpaidOrFraudulent
    );
}

#[test]
fn out_of_range_alarm_calls_are_unperceived() {
    let receiver = LocalAgent::new(1, LocalCoord::new(0, 0), 100, 100, 3);

    // Sender is at distance 5 > receiver vision range (3)
    let out_of_range_alarm = VerifiedAlarmEvent {
        sender_id: 2,
        sender_pos: LocalCoord::new(5, 0),
        threat_sector: 2,
        tick: 1,
        paid_energy: 10,
        paid_stamina: 5,
        is_paid: true,
    };

    assert_eq!(
        process_local_alarm(&receiver, &out_of_range_alarm, 1),
        AlarmPerception::IgnoredOutOfRange {
            distance: 5,
            vision_range: 3,
        }
    );
}

#[test]
fn stale_alarm_calls_are_ignored() {
    let receiver = LocalAgent::new(1, LocalCoord::new(0, 0), 100, 100, 5);

    let alarm = VerifiedAlarmEvent {
        sender_id: 2,
        sender_pos: LocalCoord::new(2, 0),
        threat_sector: 2,
        tick: 10,
        paid_energy: 10,
        paid_stamina: 5,
        is_paid: true,
    };

    // Current tick 10 (same tick) -> valid
    assert!(matches!(
        process_local_alarm(&receiver, &alarm, 10),
        AlarmPerception::Perceived { .. }
    ));

    // Current tick 11 (previous tick) -> valid
    assert!(matches!(
        process_local_alarm(&receiver, &alarm, 11),
        AlarmPerception::Perceived { .. }
    ));

    // Current tick 12 (two ticks later) -> stale
    assert_eq!(
        process_local_alarm(&receiver, &alarm, 12),
        AlarmPerception::IgnoredStaleTick {
            alarm_tick: 10,
            current_tick: 12,
        }
    );
}

#[test]
fn verified_local_alarm_pays_exact_costs_and_shifts_receiver_decision_away_from_threat_sector() {
    let mut sender = LocalAgent::new(2, LocalCoord::new(2, 0), 50, 40, 5);
    let mut receiver = LocalAgent::new(1, LocalCoord::new(0, 0), 100, 100, 5);

    let threat_sector = 2; // East
    let energy_cost = 12;
    let stamina_cost = 8;

    // 1. Resolver deducts exact costs
    let resolution = resolve_alarm_call(
        &mut sender,
        threat_sector,
        energy_cost,
        stamina_cost,
        4,
    );
    let paid_event = match resolution {
        AlarmResolution::Paid(evt) => evt,
        other => panic!("Expected paid resolution, got {:?}", other),
    };

    assert_eq!(sender.energy, 50 - 12);
    assert_eq!(sender.stamina, 40 - 8);
    assert_eq!(paid_event.paid_energy, 12);
    assert_eq!(paid_event.paid_stamina, 8);
    assert_eq!(paid_event.threat_sector, 2);

    // 2. Receiver processes alarm without receiving hidden threat identity or global threat coordinates
    let perception = process_local_alarm(&receiver, &paid_event, 4);
    assert_eq!(
        perception,
        AlarmPerception::Perceived {
            sender_id: 2,
            warned_sector: 2,
        }
    );

    // 3. Receiver shifts decision strictly away from warned sector (East -> West)
    let default_step = 2; // East
    let coordinated_step =
        choose_coordinated_step(&mut receiver, Some(&perception), default_step);
    assert_eq!(coordinated_step, (2 + 4) % 8); // 6 (West)
    assert_ne!(coordinated_step, default_step);
}

#[test]
fn peer_alignment_only_occurs_within_receiver_vision_range() {
    let agent = LocalAgent::new(1, LocalCoord::new(0, 0), 100, 100, 4);

    let near_peer = LocalAgent::new(2, LocalCoord::new(3, 2), 100, 100, 5);
    let far_peer = LocalAgent::new(3, LocalCoord::new(6, 0), 100, 100, 5);

    // Near peer is at distance max(3, 2) = 3 <= 4 -> alignment succeeds
    assert_eq!(
        choose_peer_alignment_step(&agent, &near_peer, 1),
        Some(1)
    );

    // Far peer is at distance 6 > 4 -> alignment is None (unperceived)
    assert_eq!(
        choose_peer_alignment_step(&agent, &far_peer, 1),
        None
    );
}

#[test]
fn three_arm_comparison_measures_control_solo_and_group_separately() {
    let solo_metrics = run_coordination_harness(CoordinationArm::ControlSolo, 8);
    let unverified_metrics = run_coordination_harness(CoordinationArm::UnverifiedIgnored, 8);
    let verified_metrics = run_coordination_harness(CoordinationArm::VerifiedLocalCoordination, 8);

    // 1. ControlSolo: no sender, no alarm emitted or perceived; focal agent moves toward threat and dies
    assert_eq!(solo_metrics.alarms_emitted, 0);
    assert_eq!(solo_metrics.alarms_verified_paid, 0);
    assert_eq!(solo_metrics.alarms_perceived_by_focal, 0);
    assert!(!solo_metrics.focal_survived);
    assert!(solo_metrics.min_threat_distance_to_focal <= 1);
    assert_eq!(solo_metrics.total_energy_spent_sender, 0);

    // 2. UnverifiedIgnored: alarms are emitted but unpaid/unverified; focal ignores them and dies
    assert!(unverified_metrics.alarms_emitted > 0);
    assert_eq!(unverified_metrics.alarms_verified_paid, 0);
    assert_eq!(unverified_metrics.alarms_perceived_by_focal, 0);
    assert!(unverified_metrics.alarms_rejected_or_unperceived > 0);
    assert!(!unverified_metrics.focal_survived);
    assert!(unverified_metrics.min_threat_distance_to_focal <= 1);
    assert_eq!(unverified_metrics.total_energy_spent_sender, 0);

    // 3. VerifiedLocalCoordination: sender pays exact cost, focal receives verified warning and evades early
    assert!(verified_metrics.alarms_emitted > 0);
    assert!(verified_metrics.alarms_verified_paid > 0);
    assert!(verified_metrics.alarms_perceived_by_focal > 0);
    assert!(verified_metrics.focal_survived);
    assert!(verified_metrics.min_threat_distance_to_focal > 1);
    assert!(verified_metrics.total_energy_spent_sender >= 10);
    assert!(verified_metrics.total_stamina_spent_sender >= 5);

    // 4. Arms are strictly distinct and measured separately
    assert_ne!(solo_metrics, verified_metrics);
    assert_ne!(unverified_metrics, verified_metrics);
    assert_ne!(solo_metrics.alarms_emitted, unverified_metrics.alarms_emitted);
}

#[test]
fn finite_social_actions_structure_contract() {
    let alarm = LocalSocialAction::EmitAlarmCall {
        threat_sector: 2,
        energy_cost: 10,
        stamina_cost: 5,
    };
    let align = LocalSocialAction::AlignWithVisiblePeer {
        peer_id: 42,
        step_sector: 6,
    };
    assert!(matches!(
        alarm,
        LocalSocialAction::EmitAlarmCall {
            threat_sector: 2,
            ..
        }
    ));
    assert!(matches!(
        align,
        LocalSocialAction::AlignWithVisiblePeer {
            peer_id: 42,
            step_sector: 6
        }
    ));
}
