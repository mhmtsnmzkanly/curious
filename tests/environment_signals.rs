use curious::{
    environment::{DayNightCycle, Environment, SignalKind, SignalTrace, MAX_ENVIRONMENT_SIGNALS},
    map::movement::Position,
    persistence::State,
    scenario::Scenario,
    systems::perception::PerceptionSystem,
};

#[test]
fn default_off_preserves_state_and_rng_identity_and_clean_json_serialization() {
    let mut w = serde_json::from_str::<Scenario>(include_str!("../scenarios/meadow.json"))
        .unwrap()
        .build(42)
        .unwrap();

    // Verify default-off state on fresh world
    assert!(w.environment.day_night.is_none());
    assert!(!w.environment.signals_enabled);
    assert!(w.environment.signals.is_empty());

    // Clean JSON serialization: default fields are omitted via skip_serializing_if
    let env_json = serde_json::to_string(&w.environment).unwrap();
    assert!(!env_json.contains("day_night"));
    assert!(!env_json.contains("signals_enabled"));
    assert!(!env_json.contains("signals"));

    for _ in 0..30 {
        w.tick();
    }

    // Capture state and check perception serialization
    let captured = State::capture(&w);
    let mut resumed = captured.restore().unwrap();

    let view = PerceptionSystem::build(&w, &w.entities[0]);
    assert!(view.signals.is_empty());
    let view_json = serde_json::to_string(&view).unwrap();
    assert!(!view_json.contains("signals"));

    // Deterministic RNG and State continuation
    for _ in 0..30 {
        w.tick();
        resumed.tick();
        assert_eq!(w.rng.state(), resumed.rng.state());
        assert_eq!(
            serde_json::to_value(State::capture(&w)).unwrap(),
            serde_json::to_value(State::capture(&resumed)).unwrap()
        );
    }
}

#[test]
fn day_night_cycle_alternates_deterministically_reduces_vision_and_validates_bounds() {
    let cycle = DayNightCycle {
        period: 20,
        day_ticks: 12,
        night_vision_penalty: 5,
    };
    assert!(cycle.validate().is_ok());

    // Day: ticks 0..12
    for t in 0..12 {
        assert!(cycle.is_day(t), "tick {} should be day", t);
        assert_eq!(cycle.effective_vision_range(t, 10), 10);
    }

    // Night: ticks 12..20
    for t in 12..20 {
        assert!(!cycle.is_day(t), "tick {} should be night", t);
        assert_eq!(cycle.effective_vision_range(t, 10), 5);
        // Floor at 1
        assert_eq!(cycle.effective_vision_range(t, 3), 1);
        assert_eq!(cycle.effective_vision_range(t, 0), 1);
    }

    // Next period: tick 20 is day, tick 33 is night (33 % 20 = 13 >= 12)
    assert!(cycle.is_day(20));
    assert!(!cycle.is_day(33));

    // Integration into World perception
    let mut w = serde_json::from_str::<Scenario>(include_str!("../scenarios/meadow.json"))
        .unwrap()
        .build(99)
        .unwrap();

    w.environment.day_night = Some(cycle);

    // At tick 0 (day): full vision
    let base_vision = w.entities[0].base.life().vision_range;
    let view_day = PerceptionSystem::build(&w, &w.entities[0]);
    let day_max_dist = view_day.cells.iter().map(|(p, _)| (p.x - w.entities[0].pos.x).abs().max((p.y - w.entities[0].pos.y).abs())).max().unwrap_or(0);
    assert!(day_max_dist <= base_vision as isize);

    // Advance to tick 15 (night)
    w.tick_counter = 15;
    let view_night = PerceptionSystem::build(&w, &w.entities[0]);
    let night_max_dist = view_night.cells.iter().map(|(p, _)| (p.x - w.entities[0].pos.x).abs().max((p.y - w.entities[0].pos.y).abs())).max().unwrap_or(0);
    let expected_night_range = base_vision.saturating_sub(5).max(1);
    assert!(night_max_dist <= expected_night_range as isize);

    // Invalid parameters rejected
    assert!(DayNightCycle { period: 0, day_ticks: 0, night_vision_penalty: 5 }.validate().is_err());
    assert!(DayNightCycle { period: 1_000_001, day_ticks: 10, night_vision_penalty: 5 }.validate().is_err());
    assert!(DayNightCycle { period: 20, day_ticks: 21, night_vision_penalty: 5 }.validate().is_err());
    assert!(DayNightCycle { period: 20, day_ticks: 10, night_vision_penalty: 33 }.validate().is_err());
}

#[test]
fn signal_trace_emission_attenuation_decay_and_epistemic_anonymity() {
    let mut env = Environment::default();
    env.signals_enabled = true;

    // Emit sound signal at (5, 5) with intensity 100, radius 10, decay 10, emitted at tick 5
    let trace = SignalTrace {
        origin: Position::new(5, 5),
        kind: SignalKind::Sound,
        intensity: 100,
        radius: 10,
        decay_per_tick: 10,
        emitted_tick: 5,
    };
    assert!(trace.validate().is_ok());
    env.emit_signal(trace);
    assert_eq!(env.signals.len(), 1);

    // Listener at (5, 5) (distance 0) perceives full intensity
    let signals_at_origin = env.perceive_signals_at(Position::new(5, 5), 5);
    assert_eq!(signals_at_origin.len(), 1);
    assert_eq!(signals_at_origin[0].distance, 0);
    assert_eq!(signals_at_origin[0].local_intensity, 100);
    assert_eq!(signals_at_origin[0].kind, SignalKind::Sound);

    // Listener at (5, 8) (dy=3, dx=0, distance 3, direction Down = sector 4)
    // Attenuation = 3 * 10 = 30 => local_intensity = 70
    let signals_down = env.perceive_signals_at(Position::new(5, 8), 5);
    assert_eq!(signals_down.len(), 1);
    assert_eq!(signals_down[0].distance, 3);
    assert_eq!(signals_down[0].direction_sector, 0); // signal is at (5,5), listener at (5,8) => signal is UP from listener = sector 0
    assert_eq!(signals_down[0].local_intensity, 70);

    // Listener at (2, 5) (dx=-3, dy=0 => signal at (5,5) is RIGHT from listener = sector 2)
    let signals_left = env.perceive_signals_at(Position::new(2, 5), 5);
    assert_eq!(signals_left.len(), 1);
    assert_eq!(signals_left[0].distance, 3);
    assert_eq!(signals_left[0].direction_sector, 2);
    assert_eq!(signals_left[0].local_intensity, 70);

    // Listener outside radius (dx=11 > radius 10)
    let signals_far = env.perceive_signals_at(Position::new(16, 5), 5);
    assert!(signals_far.is_empty());

    // Listener where attenuation completely exhausts intensity (distance 10 * 10 = 100)
    let signals_boundary = env.perceive_signals_at(Position::new(15, 5), 5);
    assert!(signals_boundary.is_empty());

    // Future signal: emitted at tick 10, listener queries at tick 5 => not perceived!
    let future_trace = SignalTrace {
        origin: Position::new(6, 6),
        kind: SignalKind::Scent,
        intensity: 50,
        radius: 5,
        decay_per_tick: 5,
        emitted_tick: 10,
    };
    env.emit_signal(future_trace);
    assert_eq!(env.signals.len(), 2);
    let signals_tick5 = env.perceive_signals_at(Position::new(6, 6), 5);
    assert_eq!(signals_tick5.len(), 1); // only the sound signal from tick 5
    assert_eq!(signals_tick5[0].kind, SignalKind::Sound);

    // Querying at tick 10 perceives the scent signal
    let signals_tick10 = env.perceive_signals_at(Position::new(6, 6), 10);
    assert_eq!(signals_tick10.len(), 2);

    // Test per-tick decay during Environment::update
    let mut map = curious::map::Map::new(0, 10, 0, 10);
    let mut events = Vec::new();

    // 1 tick of update reduces sound intensity by 10 (100 -> 90) and scent by 5 (50 -> 45)
    env.update(&mut map, 11, &mut events);
    assert_eq!(env.signals[0].intensity, 90);
    assert_eq!(env.signals[1].intensity, 45);

    // 9 more ticks reduces scent intensity to 0 (45 - 9*5 = 0), so it is removed!
    for t in 12..=20 {
        env.update(&mut map, t, &mut events);
    }
    assert_eq!(env.signals.len(), 0); // both decayed to 0 and removed
}

#[test]
fn signal_capacity_is_bounded_at_64_evicting_oldest() {
    let mut env = Environment::default();
    env.signals_enabled = true;

    // Fill to 64 signals across ticks 1..=64
    for i in 1..=MAX_ENVIRONMENT_SIGNALS {
        env.emit_signal(SignalTrace {
            origin: Position::new(i as isize, 0),
            kind: SignalKind::Footstep,
            intensity: 100,
            radius: 5,
            decay_per_tick: 1,
            emitted_tick: i,
        });
    }
    assert_eq!(env.signals.len(), MAX_ENVIRONMENT_SIGNALS);

    // Emit 65th signal at tick 65
    env.emit_signal(SignalTrace {
        origin: Position::new(65, 0),
        kind: SignalKind::Footstep,
        intensity: 100,
        radius: 5,
        decay_per_tick: 1,
        emitted_tick: 65,
    });
    assert_eq!(env.signals.len(), MAX_ENVIRONMENT_SIGNALS);

    // Signal from tick 1 was evicted; signal from tick 65 is present
    assert!(!env.signals.iter().any(|s| s.emitted_tick == 1));
    assert!(env.signals.iter().any(|s| s.emitted_tick == 65));
}

#[test]
fn perception_system_integrates_signals_when_enabled() {
    let mut w = serde_json::from_str::<Scenario>(include_str!("../scenarios/meadow.json"))
        .unwrap()
        .build(123)
        .unwrap();

    let entity_pos = w.entities[0].pos;

    // With signals disabled, perception has no signals
    let view = PerceptionSystem::build(&w, &w.entities[0]);
    assert!(view.signals.is_empty());

    // Enable signals and emit a footstep near entity
    w.environment.signals_enabled = true;
    w.environment.emit_signal(SignalTrace {
        origin: Position::new(entity_pos.x + 2, entity_pos.y),
        kind: SignalKind::Footstep,
        intensity: 50,
        radius: 5,
        decay_per_tick: 5,
        emitted_tick: w.tick_counter,
    });

    let view_with_signals = PerceptionSystem::build(&w, &w.entities[0]);
    assert_eq!(view_with_signals.signals.len(), 1);
    assert_eq!(view_with_signals.signals[0].kind, SignalKind::Footstep);
    assert_eq!(view_with_signals.signals[0].distance, 2);
    assert_eq!(view_with_signals.signals[0].direction_sector, 2); // (entity_pos.x + 2, entity_pos.y) is RIGHT from entity => sector 2
    assert_eq!(view_with_signals.signals[0].local_intensity, 40); // 50 - 2*5 = 40
}
