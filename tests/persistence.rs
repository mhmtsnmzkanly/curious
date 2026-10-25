use curious::{
    creatures::herbivore::HerbivoreEntity,
    entity::{
        controller::EntityController,
        learning::ContextWaterEntry,
        memory::KnownSector,
        phase::EntityPhase,
        refuge_learning::{FirstIntervalRefugeRecord, RefugeKey},
        species::Species,
    },
    environment::{BurrowSite, SeasonCycle, SignalKind, SignalTrace},
    map::movement::Position,
    persistence::State,
    world::{EntitySlot, World},
};
use std::collections::VecDeque;

fn sample_world() -> World {
    let herbivore = HerbivoreEntity::default();
    let slot = EntitySlot::new(
        1,
        Position::new(0, 0),
        EntityPhase::Active,
        EntityController::Player(0),
        Box::new(herbivore),
    );
    World::with_seed(-10, 10, -10, 10, vec![slot], 123)
}

fn assert_err_contains(state: State, expected: &str) {
    match state.restore() {
        Err(err) => assert!(
            err.to_string().contains(expected),
            "expected error containing {:?}, got {:?}",
            expected,
            err.to_string()
        ),
        Ok(_) => panic!("expected restore to return Err, but it succeeded"),
    }
}

#[test]
fn v19_checkpoint_preserves_all_new_fields_and_rejects_future_versions() {
    let mut world = sample_world();
    world.tick_counter = 100;
    world.policy.juvenile_care_enabled = true;
    world.policy.ambush_predation_enabled = true;
    world.policy.local_coordination_enabled = true;

    world.environment.signals_enabled = true;
    world.environment.signals.push(SignalTrace {
        origin: Position::new(1, 2),
        kind: SignalKind::Sound,
        intensity: 50,
        radius: 8,
        decay_per_tick: 5,
        emitted_tick: 95,
    });
    world.environment.season_cycle = Some(SeasonCycle {
        period: 400,
        winter_bmr_surcharge: 10,
        winter_plant_regen_divisor: 2,
    });
    world.environment.burrows.push(BurrowSite {
        pos: Position::new(3, 4),
        created_tick: 90,
        ttl: 200,
    });

    let individual = &mut world.entities[0];
    individual.learner.context_ttl_enabled = true;
    let mut obs = VecDeque::new();
    obs.push_back(10);
    individual.learner.context_entries.push(ContextWaterEntry {
        pos: Position::new(2, 2),
        speed: 1,
        move_energy_cost: 1,
        last_seen_tick: 98,
        observations: obs,
    });

    individual.memory.spatial_curiosity_enabled = true;
    individual.memory.known_sectors.push(KnownSector {
        sector_x: 1,
        sector_y: -1,
        last_visited_tick: 92,
        visit_count: 3,
    });

    individual.memory.first_interval_refuge_enabled = true;
    let mut eps = VecDeque::new();
    eps.push_back((91, 5));
    individual
        .memory
        .first_interval_outcomes
        .push(FirstIntervalRefugeRecord {
            key: RefugeKey {
                threat_id: 99,
                threat_species: Species::Carnivore,
                bearing: 0,
                threat_distance: 1,
                energy_quartile: 2,
                stamina_quartile: 2,
                refuge_options: vec![Position::new(3, 4)],
                endpoint: Position::new(3, 4),
            },
            episodes: eps,
            censored_count: 0,
        });

    let captured = State::capture(&world);
    assert_eq!(captured.version, 19);

    // Verify serde_json roundtrip
    let json_str = serde_json::to_string(&captured).expect("serialization succeeds");
    let deserialized: State = serde_json::from_str(&json_str).expect("deserialization succeeds");
    assert_eq!(deserialized.version, 19);

    // Verify restore preserves fields losslessly
    let restored = deserialized.restore().expect("restore succeeds");
    assert_eq!(restored.tick_counter, 100);
    assert!(restored.policy.juvenile_care_enabled);
    assert!(restored.policy.ambush_predation_enabled);
    assert!(restored.policy.local_coordination_enabled);

    assert!(restored.environment.signals_enabled);
    assert_eq!(restored.environment.signals.len(), 1);
    assert_eq!(restored.environment.signals[0].emitted_tick, 95);
    assert_eq!(
        restored.environment.season_cycle,
        Some(SeasonCycle {
            period: 400,
            winter_bmr_surcharge: 10,
            winter_plant_regen_divisor: 2,
        })
    );
    assert_eq!(restored.environment.burrows.len(), 1);
    assert_eq!(restored.environment.burrows[0].created_tick, 90);

    let restored_ind = &restored.entities[0];
    assert!(restored_ind.learner.context_ttl_enabled);
    assert_eq!(restored_ind.learner.context_entries.len(), 1);
    assert_eq!(restored_ind.learner.context_entries[0].last_seen_tick, 98);
    assert!(restored_ind.memory.spatial_curiosity_enabled);
    assert_eq!(restored_ind.memory.known_sectors.len(), 1);
    assert_eq!(restored_ind.memory.known_sectors[0].last_visited_tick, 92);
    assert!(restored_ind.memory.first_interval_refuge_enabled);
    assert_eq!(restored_ind.memory.first_interval_outcomes.len(), 1);
    assert_eq!(restored_ind.memory.first_interval_outcomes[0].episodes[0], (91, 5));

    let recaptured = State::capture(&restored);
    assert_eq!(recaptured.version, 19);

    // Rejects version == 0
    let mut bad_version_0 = captured.clone();
    bad_version_0.version = 0;
    assert!(bad_version_0.restore().is_err());

    // Rejects future version == 20
    let mut bad_version_20 = captured.clone();
    bad_version_20.version = 20;
    assert!(bad_version_20.restore().is_err());

    // Rejects future environment timestamp (signal emitted_tick > tick)
    let mut future_signal = captured.clone();
    future_signal.environment.signals[0].emitted_tick = 101;
    assert_err_contains(future_signal, "future environment timestamp");

    // Rejects future environment timestamp (burrow created_tick > tick)
    let mut future_burrow = captured.clone();
    future_burrow.environment.burrows[0].created_tick = 101;
    assert_err_contains(future_burrow, "future environment timestamp");

    // Rejects future learning timestamp (context entry last_seen_tick > tick)
    let mut future_context = captured.clone();
    future_context.entities[0].learner.context_entries[0].last_seen_tick = 101;
    assert_err_contains(future_context, "future V19 learning or memory timestamp");

    // Rejects future memory timestamp (known sector last_visited_tick > tick)
    let mut future_sector = captured.clone();
    future_sector.entities[0].memory.known_sectors[0].last_visited_tick = 101;
    assert_err_contains(future_sector, "future V19 learning or memory timestamp");

    // Rejects future memory timestamp (first interval outcome episode tick > tick)
    let mut future_first_interval = captured.clone();
    future_first_interval.entities[0].memory.first_interval_outcomes[0].episodes[0].0 = 101;
    assert_err_contains(future_first_interval, "future V19 learning or memory timestamp");
}

#[test]
fn v18_checkpoint_migrates_cleanly_to_v19_with_v19_features_default_off() {
    let mut world = sample_world();
    world.tick_counter = 50;

    // Set V19 features on the world before capture
    world.policy.juvenile_care_enabled = true;
    world.policy.ambush_predation_enabled = true;
    world.policy.local_coordination_enabled = true;

    world.environment.signals_enabled = true;
    world.environment.signals.push(SignalTrace {
        origin: Position::new(0, 1),
        kind: SignalKind::Sound,
        intensity: 20,
        radius: 4,
        decay_per_tick: 2,
        emitted_tick: 40,
    });
    world.environment.season_cycle = Some(SeasonCycle {
        period: 200,
        winter_bmr_surcharge: 5,
        winter_plant_regen_divisor: 2,
    });
    world.environment.burrows.push(BurrowSite {
        pos: Position::new(1, 1),
        created_tick: 30,
        ttl: 100,
    });

    let individual = &mut world.entities[0];
    individual.learner.context_ttl_enabled = true;
    let mut obs = VecDeque::new();
    obs.push_back(5);
    individual.learner.context_entries.push(ContextWaterEntry {
        pos: Position::new(1, 1),
        speed: 1,
        move_energy_cost: 1,
        last_seen_tick: 45,
        observations: obs,
    });

    individual.memory.spatial_curiosity_enabled = true;
    individual.memory.known_sectors.push(KnownSector {
        sector_x: 0,
        sector_y: 0,
        last_visited_tick: 40,
        visit_count: 1,
    });

    individual.memory.first_interval_refuge_enabled = true;
    let mut eps = VecDeque::new();
    eps.push_back((35, 2));
    individual
        .memory
        .first_interval_outcomes
        .push(FirstIntervalRefugeRecord {
            key: RefugeKey {
                threat_id: 10,
                threat_species: Species::Carnivore,
                bearing: 2,
                threat_distance: 1,
                energy_quartile: 1,
                stamina_quartile: 1,
                refuge_options: vec![Position::new(1, 1)],
                endpoint: Position::new(1, 1),
            },
            episodes: eps,
            censored_count: 0,
        });

    let mut state = State::capture(&world);
    // Demote version to 18 to simulate older checkpoint
    state.version = 18;

    let restored = state.restore().expect("v18 checkpoint restores cleanly");

    // Verify all V19 features are cleanly migrated as false/empty/None
    assert!(!restored.policy.juvenile_care_enabled);
    assert!(!restored.policy.ambush_predation_enabled);
    assert!(!restored.policy.local_coordination_enabled);

    assert!(restored.environment.day_night.is_none());
    assert!(!restored.environment.signals_enabled);
    assert!(restored.environment.signals.is_empty());
    assert!(restored.environment.season_cycle.is_none());
    assert!(restored.environment.burrows.is_empty());

    let migrated_ind = &restored.entities[0];
    assert!(!migrated_ind.learner.context_ttl_enabled);
    assert!(migrated_ind.learner.context_entries.is_empty());
    assert!(!migrated_ind.memory.spatial_curiosity_enabled);
    assert!(migrated_ind.memory.known_sectors.is_empty());
    assert!(!migrated_ind.memory.first_interval_refuge_enabled);
    assert!(migrated_ind.memory.first_interval_outcomes.is_empty());

    // Verify subsequent capture produces version 19
    let captured_after_restore = State::capture(&restored);
    assert_eq!(captured_after_restore.version, 19);
}
