use curious::{
    creatures::herbivore::HerbivoreEntity,
    entity::{
        controller::EntityController,
        decision::{Decision, Reason},
        intent::Intent,
        learning::{
            Learner, MAX_RESIDUAL_SAMPLES, MAX_WATER_TICKS, ResidualValue, Value, WaterEnd,
            WaterEpisode,
        },
        perception::Perception,
        phase::EntityPhase,
    },
    map::{
        Map,
        cell::Cell,
        movement::{Direction, Position, Steps},
    },
    random::Random,
    simulation::{SimulationEvent, checkpoint::State, event::RejectionReason},
    world::{EntitySlot, World},
};
fn water_world() -> World {
    let mut herb = HerbivoreEntity::default();
    herb.life_state.speed = 1;
    herb.life_state.water = 10;
    let mut w = World::with_seed(
        0,
        6,
        0,
        6,
        vec![EntitySlot::new(
            1,
            Position::new(0, 0),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(herb),
        )],
        42,
    );
    w.logger = curious::logger::Logger::disabled();
    w.environment = Default::default();
    w.map = Map::new(0, 6, 0, 6);
    w.map
        .set_cell(Position::new(5, 0), Cell::Water { amount: 30 });
    w.map
        .set_cell(Position::new(0, 5), Cell::Water { amount: 30 });
    w
}
fn attempting() -> Learner {
    let mut learner = Learner::default();
    learner.episode = Some(WaterEpisode {
        target: Position::new(5, 0),
        ticks: 0,
        movement_energy: 0,
        incoming_damage: 0,
        choice_baseline: None,
        outcome_overflowed: false,
        candidate_censored: false,
    });
    learner
}
#[test]
fn slow_trip_is_one_experience_and_resumes_identically() {
    let mut w = water_world();
    w.policy.water_residual_enabled = true;
    let events = w.tick();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, SimulationEvent::EntityMoved { energy_cost: 1, .. }))
    );
    assert!(w.entities[0].learner.values.is_empty());
    let target = w.entities[0].learner.episode.as_ref().unwrap().target;
    let mut resumed = State::capture(&w).restore().unwrap();
    for _ in 0..3 {
        w.tick();
        resumed.tick();
        assert_eq!(
            serde_json::to_value(State::capture(&w)).unwrap(),
            serde_json::to_value(State::capture(&resumed)).unwrap()
        );
    }
    let l = &w.entities[0].learner;
    assert!(l.episode.is_none());
    assert_eq!(l.values.len(), 1);
    assert_eq!(l.values[0].visits, 1);
    assert_eq!(l.values[0].pos, target);
    let result = l.last_episode.as_ref().unwrap();
    assert_eq!(result.end, WaterEnd::Reached);
    assert_eq!(result.attempt.ticks, 4);
    assert_eq!(result.attempt.movement_energy, 4);
    assert_eq!(result.water, 10);
    assert_eq!(result.reward, Some(88));
    assert_eq!(l.residuals.len(), 1);
    assert_eq!(l.residuals[0].visits, 1);
    let mut resumed_after_learning = State::capture(&w).restore().unwrap();
    for _ in 0..3 {
        w.tick();
        resumed_after_learning.tick();
        assert_eq!(
            serde_json::to_value(State::capture(&w)).unwrap(),
            serde_json::to_value(State::capture(&resumed_after_learning)).unwrap()
        );
    }
}
#[test]
fn accounting_uses_actor_and_target_events_not_net_body_deltas() {
    let mut learner = attempting();
    learner.finish(
        1,
        true,
        &[
            SimulationEvent::EntityMoved {
                id: 1,
                from: Position::new(0, 0),
                to: Position::new(1, 0),
                energy_cost: 3,
            },
            SimulationEvent::EntityMoved {
                id: 9,
                from: Position::new(0, 0),
                to: Position::new(1, 0),
                energy_cost: 100,
            },
            SimulationEvent::EntityAttacked {
                energy_cost: 1,
                killed: false,
                attacker: 9,
                target: 1,
                damage: 7,
            },
            SimulationEvent::EntityDrank {
                id: 1,
                pos: Position::new(0, 5),
                amount: 10,
            },
        ],
    );
    assert!(learner.values.is_empty());
    learner.finish(
        1,
        true,
        &[
            SimulationEvent::EntityMoved {
                id: 1,
                from: Position::new(1, 0),
                to: Position::new(2, 0),
                energy_cost: 2,
            },
            SimulationEvent::EntityDrank {
                id: 1,
                pos: Position::new(5, 0),
                amount: 10,
            },
        ],
    );
    let r = learner.last_episode.unwrap();
    assert_eq!(r.reward, Some(88));
    assert_eq!(r.attempt.incoming_damage, 7);
}
#[test]
fn threats_and_external_commands_do_not_poison_source_values() {
    let mut l = attempting();
    l.choose(
        Position::new(0, 0),
        &Perception::empty(),
        Decision::new(Intent::Idle { duration: 1 }, Reason::EscapeThreat),
        &mut Random::new(1),
    );
    assert!(l.values.is_empty());
    assert_eq!(l.last_episode.unwrap().end, WaterEnd::Interrupted);
    let mut w = water_world();
    w.tick();
    w.push_command(1, Intent::Idle { duration: 1 });
    w.tick();
    assert!(w.entities[0].learner.episode.is_none());
    assert!(w.entities[0].learner.values.is_empty());
}
#[test]
fn retries_are_bounded_and_failure_is_not_approach_delay() {
    let mut l = attempting();
    for _ in 1..MAX_WATER_TICKS {
        l.finish(1, true, &[]);
        assert!(l.values.is_empty());
    }
    l.finish(1, true, &[]);
    assert_eq!(l.last_episode.as_ref().unwrap().end, WaterEnd::TimedOut);
    assert_eq!(l.values[0].value, -30);
    let mut l = attempting();
    l.finish(
        1,
        true,
        &[SimulationEvent::ActionRejected {
            id: 1,
            action: "Approach".into(),
            target: None,
            reason: RejectionReason::OutOfRange,
        }],
    );
    assert!(l.values.is_empty());
    l.finish(
        1,
        true,
        &[SimulationEvent::ActionRejected {
            id: 1,
            action: "Drink".into(),
            target: None,
            reason: RejectionReason::MissingResource,
        }],
    );
    assert_eq!(l.last_episode.unwrap().end, WaterEnd::Failed);
    assert_eq!(l.values[0].value, -12);
}
#[test]
fn disabling_or_death_closes_without_false_training() {
    let mut l = attempting();
    l.enabled = false;
    l.finish(1, true, &[]);
    assert!(l.values.is_empty());
    assert_eq!(l.last_episode.unwrap().end, WaterEnd::Disabled);
    let mut w = water_world();
    w.tick();
    w.entities[0].base.life_mut().health = 0;
    w.tick();
    assert!(w.entities[0].learner.episode.is_none());
    assert!(w.entities[0].learner.values.is_empty());
    assert_eq!(
        w.entities[0].learner.last_episode.as_ref().unwrap().end,
        WaterEnd::Died
    );
}

fn candidate_attempt(target: Position, baseline: i64) -> Learner {
    let mut l = Learner::default();
    l.episode = Some(WaterEpisode {
        target,
        ticks: 0,
        movement_energy: 0,
        incoming_damage: 0,
        choice_baseline: Some(baseline),
        outcome_overflowed: false,
        candidate_censored: false,
    });
    l
}

#[test]
fn residual_uses_own_resolver_currency_once_and_requires_own_source() {
    let source = Position::new(5, 0);
    let mut l = candidate_attempt(source, 1_000);
    l.finish_with_evidence(
        1,
        true,
        &[
            SimulationEvent::EntityMoved {
                id: 1,
                from: Position::new(0, 0),
                to: Position::new(1, 0),
                energy_cost: 3,
            },
            SimulationEvent::EntityMoved {
                id: 2,
                from: Position::new(0, 0),
                to: Position::new(1, 0),
                energy_cost: 10_000,
            },
            SimulationEvent::EntityDrank {
                id: 2,
                pos: source,
                amount: 10,
            },
            SimulationEvent::EntityDrank {
                id: 1,
                pos: source,
                amount: 10,
            },
        ],
        false,
    );
    assert_eq!((l.residuals[0].sum, l.residuals[0].visits), (600, 1));
    assert_eq!(l.last_episode.as_ref().unwrap().reward, Some(93));
    assert_eq!(l.last_episode.as_ref().unwrap().attempt.movement_energy, 3);

    let mut l = candidate_attempt(source, 1_000);
    l.finish_with_evidence(
        1,
        true,
        &[SimulationEvent::EntityDrank {
            id: 1,
            pos: Position::new(0, 5),
            amount: 10,
        }],
        false,
    );
    assert!(l.episode.as_ref().unwrap().candidate_censored);
    l.finish_with_evidence(
        1,
        true,
        &[SimulationEvent::EntityDrank {
            id: 1,
            pos: source,
            amount: 10,
        }],
        false,
    );
    assert!(
        l.residuals.is_empty(),
        "wrong-source own evidence censors the whole episode"
    );
}

#[test]
fn missing_resource_needs_resolved_issued_target_and_timeout_is_censored() {
    let mut l = candidate_attempt(Position::new(5, 0), 0);
    let missing = [SimulationEvent::ActionRejected {
        id: 1,
        action: "Drink".into(),
        target: None,
        reason: RejectionReason::MissingResource,
    }];
    l.finish_with_evidence(1, true, &missing, false);
    assert_eq!(l.last_episode.as_ref().unwrap().end, WaterEnd::Failed);
    assert!(l.residuals.is_empty());

    let mut l = candidate_attempt(Position::new(5, 0), 0);
    for _ in 1..MAX_WATER_TICKS {
        l.finish_with_evidence(1, true, &[], false);
    }
    l.finish_with_evidence(1, true, &[], false);
    assert_eq!(l.last_episode.as_ref().unwrap().end, WaterEnd::TimedOut);
    assert!(l.residuals.is_empty());
}

#[test]
fn an_overrun_on_an_earlier_tick_censors_a_later_valid_drink() {
    let source = Position::new(5, 0);
    let mut l = candidate_attempt(source, 0);
    l.episode.as_mut().unwrap().movement_energy = 1_000_000;
    l.finish_with_evidence(
        1,
        true,
        &[SimulationEvent::EntityMoved {
            id: 1,
            from: Position::new(0, 0),
            to: Position::new(1, 0),
            energy_cost: 1,
        }],
        false,
    );
    assert!(l.episode.as_ref().unwrap().outcome_overflowed);
    l.finish_with_evidence(
        1,
        true,
        &[SimulationEvent::EntityDrank {
            id: 1,
            pos: source,
            amount: 10,
        }],
        false,
    );
    assert_eq!(l.last_episode.as_ref().unwrap().end, WaterEnd::Reached);
    assert!(l.residuals.is_empty());
}

#[test]
fn candidate_snapshot_is_fresh_for_new_choice_and_fixed_for_continuation() {
    let origin = Position::new(0, 0);
    let mut life = HerbivoreEntity::default().life_state;
    life.water = life.max_water.saturating_sub(10);
    let first_path = Steps::new(vec![Direction::Right]);
    let second_path = Steps::new(vec![Direction::Down, Direction::Down]);
    let mut view = Perception::empty();
    view.add_water(10, first_path.clone());
    view.add_water(1, second_path);
    let mut l = Learner::default();
    let mut rng = Random::new(20); // greedy branch, as in the legacy policy
    let selected = l.choose_with_residual(
        origin,
        &life,
        &view,
        Decision::new(Intent::Idle { duration: 1 }, Reason::SeekWater),
        &mut rng,
        true,
    );
    assert!(matches!(selected.intent, Intent::Drink { .. }));
    let original = l.episode.as_ref().unwrap().choice_baseline.unwrap();
    assert_eq!(
        original,
        curious::entity::foraging::value(10, 1, &life, true) as i64
    );

    let mut changed = Perception::empty();
    changed.add_water(
        2,
        Steps::new(vec![Direction::Right, Direction::Right, Direction::Left]),
    );
    changed.add_water(1, Steps::new(vec![Direction::Down, Direction::Down]));
    let continued = l.choose_with_residual(
        origin,
        &life,
        &changed,
        Decision::new(Intent::Idle { duration: 1 }, Reason::SeekWater),
        &mut rng,
        true,
    );
    assert_eq!(l.episode.as_ref().unwrap().choice_baseline, Some(original));
    assert!(matches!(continued.intent, Intent::Drink { at: Steps(ref s) } if s.len() == 3));
}

#[test]
fn disabling_policy_mid_episode_keeps_original_snapshot_and_can_finish_it() {
    let origin = Position::new(0, 0);
    let life = HerbivoreEntity::default().life_state;
    let mut view = Perception::empty();
    view.add_water(10, Steps::new(vec![Direction::Right]));
    view.add_water(1, Steps::new(vec![Direction::Down, Direction::Down]));
    let mut l = Learner::default();
    let mut rng = Random::new(20);
    l.choose_with_residual(
        origin,
        &life,
        &view,
        Decision::new(Intent::Idle { duration: 1 }, Reason::SeekWater),
        &mut rng,
        true,
    );
    let baseline = l.episode.as_ref().unwrap().choice_baseline;
    let target = l.episode.as_ref().unwrap().target;
    let _ = l.choose_with_residual(
        origin,
        &life,
        &view,
        Decision::new(Intent::Idle { duration: 1 }, Reason::SeekWater),
        &mut rng,
        false,
    );
    assert_eq!(l.episode.as_ref().unwrap().choice_baseline, baseline);
    l.finish_with_evidence(
        1,
        true,
        &[SimulationEvent::EntityDrank {
            id: 1,
            pos: target,
            amount: 10,
        }],
        false,
    );
    assert_eq!(l.residuals[0].pos, target);
    assert_eq!(l.residuals[0].visits, 1);
}

#[test]
fn fresh_current_foraging_context_can_reverse_a_stale_source_correction() {
    let origin = Position::new(0, 0);
    let mut life = HerbivoreEntity::default().life_state;
    life.water = life.max_water.saturating_sub(10);
    let mut l = Learner::default();
    l.residuals.push(ResidualValue {
        pos: Position::new(1, 0),
        sum: 1_000,
        visits: 1,
    });
    let mut rng = Random::new(20);
    let base = Decision::new(Intent::Idle { duration: 1 }, Reason::SeekWater);
    let mut fresh = Perception::empty();
    fresh.add_water(10, Steps::new(vec![Direction::Right]));
    fresh.add_water(10, Steps::new(vec![Direction::Down]));
    l.choose_with_residual(origin, &life, &fresh, base.clone(), &mut rng, true);
    assert_eq!(l.episode.as_ref().unwrap().target, Position::new(1, 0));

    l.interrupt(WaterEnd::Interrupted);
    rng = Random::new(20);
    let mut changed = Perception::empty();
    changed.add_water(
        1,
        Steps::new(vec![Direction::Right, Direction::Right, Direction::Left]),
    );
    changed.add_water(10, Steps::new(vec![Direction::Down]));
    l.choose_with_residual(origin, &life, &changed, base, &mut rng, true);
    assert_eq!(l.episode.as_ref().unwrap().target, Position::new(0, 1));
}

#[test]
fn frozen_updates_preserve_legacy_and_residual_tables_and_disabled_path_matches_legacy_golden() {
    let mut l = candidate_attempt(Position::new(1, 0), 0);
    l.values.push(Value {
        pos: Position::new(9, 9),
        value: 17,
        visits: 8,
    });
    l.residuals.push(ResidualValue {
        pos: Position::new(8, 8),
        sum: -123,
        visits: 7,
    });
    l.update_enabled = false;
    let before_values = (l.values[0].value, l.values[0].visits);
    let before_residual = (l.residuals[0].sum, l.residuals[0].visits);
    l.finish(
        1,
        true,
        &[SimulationEvent::EntityDrank {
            id: 1,
            pos: Position::new(1, 0),
            amount: 10,
        }],
    );
    assert_eq!((l.values[0].value, l.values[0].visits), before_values);
    assert_eq!((l.residuals[0].sum, l.residuals[0].visits), before_residual);

    // Golden from the pre-residual chooser: the 10% draw is consumed first,
    // then the legacy stored value/path/perception ordering selects the source.
    let origin = Position::new(0, 0);
    let mut view = Perception::empty();
    view.add_water(10, Steps::new(vec![Direction::Right]));
    view.add_water(10, Steps::new(vec![Direction::Down]));
    let mut legacy = Learner::default();
    legacy.values = vec![
        Value {
            pos: Position::new(1, 0),
            value: 20,
            visits: 2,
        },
        Value {
            pos: Position::new(0, 1),
            value: 40,
            visits: 2,
        },
    ];
    legacy.residuals.push(ResidualValue {
        pos: Position::new(0, 1),
        sum: 100_000,
        visits: 1,
    });
    let mut rng = Random::new(20);
    let got = legacy.choose_with_residual(
        origin,
        &HerbivoreEntity::default().life_state,
        &view,
        Decision::new(Intent::Idle { duration: 1 }, Reason::SeekWater),
        &mut rng,
        false,
    );
    assert_eq!(legacy.episode.as_ref().unwrap().target, Position::new(0, 1));
    assert!(matches!(got.intent, Intent::Drink { .. }));
    assert_eq!(
        rng.state(),
        Random::new(20)
            .next_u64()
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1)
    );
    assert_eq!(legacy.episode.as_ref().unwrap().choice_baseline, None);

    // Exploration golden: seed 9 takes the exploration branch and consumes the
    // second range draw; the pre-residual chooser's candidate 0 is retained.
    let mut explore = Learner::default();
    let mut rng = Random::new(9);
    explore.choose_with_residual(
        origin,
        &HerbivoreEntity::default().life_state,
        &view,
        Decision::new(Intent::Idle { duration: 1 }, Reason::SeekWater),
        &mut rng,
        false,
    );
    assert_eq!(
        explore.episode.as_ref().unwrap().target,
        Position::new(1, 0)
    );
    let first = 9u64.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    let expected_rng = first
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1);
    assert_eq!(rng.state(), expected_rng);
}

#[test]
fn residual_sample_limit_refuses_update_without_changing_exact_history() {
    let mut l = candidate_attempt(Position::new(1, 0), 0);
    l.residuals.push(ResidualValue {
        pos: Position::new(1, 0),
        sum: 42,
        visits: MAX_RESIDUAL_SAMPLES,
    });
    l.finish_with_evidence(
        1,
        true,
        &[SimulationEvent::EntityDrank {
            id: 1,
            pos: Position::new(1, 0),
            amount: 10,
        }],
        false,
    );
    assert_eq!(
        (l.residuals[0].sum, l.residuals[0].visits),
        (42, MAX_RESIDUAL_SAMPLES)
    );
    l.validate().unwrap();
}
#[test]
fn legacy_state_defaults_and_episode_limits_are_checked() {
    let mut old = serde_json::to_value(State::capture(&water_world())).unwrap();
    old["version"] = 9.into();
    let actor = &mut old["entities"][0]["learner"];
    actor.as_object_mut().unwrap().remove("episode");
    actor.as_object_mut().unwrap().remove("last_episode");
    actor["pending"] = serde_json::json!({"x":5,"y":0});
    let resumed = serde_json::from_value::<State>(old)
        .unwrap()
        .restore()
        .unwrap();
    assert!(resumed.entities[0].learner.episode.is_none());
    let mut w = water_world();
    w.entities[0].learner = attempting();
    w.entities[0].learner.episode.as_mut().unwrap().ticks = MAX_WATER_TICKS;
    assert!(State::capture(&w).restore().is_err());
}

#[test]
fn v16_migration_discards_injected_residual_policy_and_snapshots_only() {
    let mut w = water_world();
    w.entities[0].learner = candidate_attempt(Position::new(5, 0), 123);
    w.entities[0].learner.residuals.push(ResidualValue {
        pos: Position::new(5, 0),
        sum: 77,
        visits: 1,
    });
    let mut ended = candidate_attempt(Position::new(5, 0), 234).episode.unwrap();
    ended.outcome_overflowed = true;
    ended.candidate_censored = true;
    w.entities[0].learner.last_episode = Some(curious::entity::learning::WaterResult {
        attempt: ended,
        water: 0,
        end: WaterEnd::TimedOut,
        reward: None,
    });
    let active = w.entities[0].learner.episode.as_mut().unwrap();
    active.outcome_overflowed = true;
    active.candidate_censored = true;
    w.entities[0].learner.values.push(Value {
        pos: Position::new(9, 9),
        value: 42,
        visits: 3,
    });
    w.policy.water_residual_enabled = true;
    let mut old = serde_json::to_value(State::capture(&w)).unwrap();
    old["version"] = 16.into();
    let restored = serde_json::from_value::<State>(old)
        .unwrap()
        .restore()
        .unwrap();
    let learner = &restored.entities[0].learner;
    assert!(!restored.policy.water_residual_enabled);
    assert!(learner.residuals.is_empty());
    assert_eq!(learner.values[0].value, 42);
    assert_eq!(
        learner.episode.as_ref().unwrap().target,
        Position::new(5, 0)
    );
    assert_eq!(learner.episode.as_ref().unwrap().choice_baseline, None);
    assert!(!learner.episode.as_ref().unwrap().outcome_overflowed);
    assert!(!learner.episode.as_ref().unwrap().candidate_censored);
    assert!(
        !learner
            .last_episode
            .as_ref()
            .unwrap()
            .attempt
            .outcome_overflowed
    );
    assert!(
        !learner
            .last_episode
            .as_ref()
            .unwrap()
            .attempt
            .candidate_censored
    );
}

#[test]
fn malformed_candidate_checkpoint_state_and_future_versions_are_rejected() {
    let pos = Position::new(1, 0);
    let cases = [
        vec![ResidualValue {
            pos,
            sum: 0,
            visits: 0,
        }],
        vec![ResidualValue {
            pos,
            sum: 0,
            visits: MAX_RESIDUAL_SAMPLES + 1,
        }],
        vec![ResidualValue {
            pos,
            sum: 101_000_001,
            visits: 1,
        }],
        vec![
            ResidualValue {
                pos,
                sum: 0,
                visits: 1,
            },
            ResidualValue {
                pos,
                sum: 0,
                visits: 1,
            },
        ],
    ];
    for residuals in cases {
        let mut state = State::capture(&water_world());
        state.entities[0].learner.residuals = residuals;
        assert!(state.restore().is_err());
    }
    let mut state = State::capture(&water_world());
    state.entities[0].learner.episode = Some(candidate_attempt(pos, 104_001).episode.unwrap());
    assert!(state.restore().is_err());
    let mut state = State::capture(&water_world());
    state.version = 22;
    assert!(state.restore().is_err());
}

#[test]
fn a_visible_chosen_target_continues_even_when_only_one_source_remains() {
    use curious::map::movement::{Direction, Steps};
    let mut learner = attempting();
    let mut view = Perception::empty();
    view.add_water(10, Steps(vec![Direction::Right; 4]));
    let mut rng = Random::new(99);
    let before = rng.state();
    let decision = learner.choose(
        Position::new(1, 0),
        &view,
        Decision::new(
            Intent::Drink {
                at: Steps(vec![Direction::Right; 4]),
            },
            Reason::SeekWater,
        ),
        &mut rng,
    );
    assert_eq!(decision.reason, Reason::LearnedWater);
    assert_eq!(rng.state(), before);
    assert_eq!(
        learner.episode.as_ref().unwrap().target,
        Position::new(5, 0)
    );
    learner.choose(
        Position::new(1, 0),
        &Perception::empty(),
        Decision::new(Intent::Idle { duration: 1 }, Reason::SearchWater),
        &mut rng,
    );
    assert!(learner.episode.is_none());
    assert!(learner.values.is_empty());
}

#[test]
fn movement_energy_is_actual_traversal_cost_even_for_a_loop() {
    use curious::map::movement::{Direction, Steps};
    let mut w = water_world();
    w.entities[0].base.life_mut().speed = 4;
    w.entities[0].base.life_mut().move_energy_cost = 2;
    w.push_command(
        1,
        Intent::Move {
            steps: Steps(vec![
                Direction::Right,
                Direction::Down,
                Direction::Left,
                Direction::Up,
            ]),
        },
    );
    let events = w.tick();
    assert_eq!(w.entities[0].pos, Position::new(0, 0));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, SimulationEvent::EntityMoved { energy_cost: 8, .. }))
    );
}
