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
fn v21_checkpoint_preserves_all_new_fields_and_rejects_future_versions() {
    let mut world = sample_world();
    world.tick_counter = 100;
    world.policy.juvenile_care_enabled = true;
    world.policy.ambush_predation_enabled = true;
    world.policy.local_coordination_enabled = true;
    world.policy.water_meal_priority_enabled = true;

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
    assert_eq!(captured.version, 21);

    // Verify serde_json roundtrip
    let json_str = serde_json::to_string(&captured).expect("serialization succeeds");
    let deserialized: State = serde_json::from_str(&json_str).expect("deserialization succeeds");
    assert_eq!(deserialized.version, 21);

    // Verify restore preserves fields losslessly
    let restored = deserialized.restore().expect("restore succeeds");
    assert_eq!(restored.tick_counter, 100);
    assert!(restored.policy.juvenile_care_enabled);
    assert!(restored.policy.ambush_predation_enabled);
    assert!(restored.policy.local_coordination_enabled);
    assert!(restored.policy.water_meal_priority_enabled);

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
    assert_eq!(
        restored_ind.memory.first_interval_outcomes[0].episodes[0],
        (91, 5)
    );

    let recaptured = State::capture(&restored);
    assert_eq!(recaptured.version, 21);

    // Rejects version == 0
    let mut bad_version_0 = captured.clone();
    bad_version_0.version = 0;
    assert!(bad_version_0.restore().is_err());

    // Rejects future version == 22
    let mut bad_version_22 = captured.clone();
    bad_version_22.version = 22;
    assert!(bad_version_22.restore().is_err());

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
    future_first_interval.entities[0]
        .memory
        .first_interval_outcomes[0]
        .episodes[0]
        .0 = 101;
    assert_err_contains(
        future_first_interval,
        "future V19 learning or memory timestamp",
    );
}

#[test]
fn v19_checkpoint_migrates_experimental_flags_off_and_v21_resumes_them() {
    let mut world = sample_world();
    let mut state = State::capture(&world);
    state.version = 19;
    state.policy.food_search_reserve_enabled = true;
    let migrated = state.restore().expect("V19 checkpoint migrates");
    assert!(!migrated.policy.food_search_reserve_enabled);
    assert!(!migrated.policy.water_meal_priority_enabled);

    world.policy.food_search_reserve_enabled = true;
    world.policy.water_meal_priority_enabled = true;
    let state = State::capture(&world);
    assert_eq!(state.version, 21);
    let expected_rng = state.rng;
    let resumed = state.restore().expect("V21 checkpoint resumes");
    assert!(resumed.policy.food_search_reserve_enabled);
    assert!(resumed.policy.water_meal_priority_enabled);
    assert_eq!(resumed.rng.state(), expected_rng);
}

#[test]
fn v20_checkpoint_migrates_water_meal_priority_off() {
    let mut world = sample_world();
    let mut state = State::capture(&world);
    state.version = 20;
    state.policy.water_meal_priority_enabled = true;
    let restored = state.restore().expect("V20 checkpoint migrates");
    assert!(!restored.policy.water_meal_priority_enabled);
    assert!(!restored.policy.food_search_reserve_enabled);

    world.policy.water_meal_priority_enabled = true;
    let state = State::capture(&world);
    assert!(state.policy.water_meal_priority_enabled);
    assert!(state.restore().unwrap().policy.water_meal_priority_enabled);
}

#[test]
fn v18_checkpoint_migrates_cleanly_to_v21_with_later_features_default_off() {
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

    // Verify subsequent capture produces current version 21.
    let captured_after_restore = State::capture(&restored);
    assert_eq!(captured_after_restore.version, 21);
}

#[test]
fn raw_v18_checkpoint_json_without_later_keys_deserializes_and_migrates_to_v21() {
    let raw_v18_json = r#"{
        "version": 18,
        "tick": 42,
        "rng": 12345,
        "next_entity_id": 2,
        "bounds": [-5, 5, -5, 5],
        "policy": {
            "mate_rendezvous": false,
            "reserve_opportunities": false,
            "water_residual_enabled": false,
            "injury_capacity_penalty": false
        },
        "genetics": {
            "enabled": true,
            "mutation_per_mille": 20,
            "extra_traits_enabled": false
        },
        "observation": {
            "full_all": false,
            "full_for": []
        },
        "environment": {
            "sites": [
                {
                    "pos": { "x": 1, "y": 0 },
                    "kind": "Food",
                    "capacity": 20,
                    "replenish": 1,
                    "interval": 2,
                    "schedule": null
                }
            ]
        },
        "resources": [
            [{ "x": 1, "y": 0 }, { "Food": { "amount": 10 } }]
        ],
        "commands": [],
        "entities": [
            {
                "id": 1,
                "pos": { "x": 0, "y": 0 },
                "phase": "Active",
                "controller": "AI",
                "species": "Herbivore",
                "generation": 0,
                "parents": [],
                "corpse_energy": 0,
                "genome": {
                    "mobility": 1,
                    "vision": 1,
                    "stamina": 1
                },
                "learner": {
                    "enabled": true,
                    "update_enabled": true,
                    "values": [],
                    "residuals": [],
                    "episode": null,
                    "last_episode": null
                },
                "hunt_learner": {
                    "enabled": false,
                    "update_enabled": true,
                    "values": [],
                    "risk_values": [],
                    "risk_enabled": false,
                    "episode": null,
                    "last_episode": null
                },
                "memory": {
                    "social": { "target": null, "retry_after": 0 },
                    "enabled": true,
                    "companions_enabled": true,
                    "sites": [],
                    "dangers": [],
                    "companions": [],
                    "refuge_outcomes_enabled": false,
                    "refuge_outcomes": [],
                    "refuge_episode": null,
                    "pending_refuge_closures": [],
                    "refuge_generalization_enabled": false,
                    "refuge_generalization_update_enabled": true,
                    "refuge_generalized_outcomes": []
                },
                "life": {
                    "age": 10,
                    "health": 100,
                    "energy": 50,
                    "water": 40,
                    "stamina": 16,
                    "max_age": 100,
                    "max_health": 100,
                    "max_energy": 80,
                    "max_water": 60,
                    "max_stamina": 16,
                    "maturity_age": 20,
                    "reproduction_interval": 20,
                    "reproduction_cooldown": 0,
                    "reproduction_reserve_per_mille": 500,
                    "vision_range": 6,
                    "speed": 3,
                    "moves_used": 0,
                    "exertion": 0,
                    "death_cause": null,
                    "max_meal": 20,
                    "move_energy_cost": 1,
                    "vision_metabolic_cost": 0,
                    "stamina_metabolic_cost": 0
                }
            }
        ]
    }"#;

    // Verify raw JSON string strictly does NOT contain any V19 keys
    assert!(!raw_v18_json.contains("juvenile_care_enabled"));
    assert!(!raw_v18_json.contains("ambush_predation_enabled"));
    assert!(!raw_v18_json.contains("local_coordination_enabled"));
    assert!(!raw_v18_json.contains("day_night"));
    assert!(!raw_v18_json.contains("signals_enabled"));
    assert!(!raw_v18_json.contains("signals"));
    assert!(!raw_v18_json.contains("season_cycle"));
    assert!(!raw_v18_json.contains("burrows"));
    assert!(!raw_v18_json.contains("context_ttl_enabled"));
    assert!(!raw_v18_json.contains("context_entries"));
    assert!(!raw_v18_json.contains("spatial_curiosity_enabled"));
    assert!(!raw_v18_json.contains("known_sectors"));
    assert!(!raw_v18_json.contains("first_interval_refuge_enabled"));
    assert!(!raw_v18_json.contains("first_interval_outcomes"));

    let state = serde_json::from_str::<State>(&raw_v18_json)
        .expect("Raw V18 JSON without V19 keys must deserialize cleanly into State");
    assert_eq!(state.version, 18);

    let mut restored = state
        .restore()
        .expect("Raw V18 state must restore cleanly into World");

    // Advance world with a tick to confirm runtime continuity
    restored.tick();

    // Verify that subsequent capture automatically migrates to current version 21.
    let captured = State::capture(&restored);
    assert_eq!(captured.version, 21);

    // Verify all V19 features are default-off cleanly:
    assert!(!restored.policy.juvenile_care_enabled);
    assert!(!restored.policy.ambush_predation_enabled);
    assert!(!restored.policy.local_coordination_enabled);

    assert!(restored.environment.day_night.is_none());
    assert!(!restored.environment.signals_enabled);
    assert!(restored.environment.signals.is_empty());
    assert!(restored.environment.season_cycle.is_none());
    assert!(restored.environment.burrows.is_empty());

    let ind = &restored.entities[0];
    assert!(!ind.learner.context_ttl_enabled);
    assert!(ind.learner.context_entries.is_empty());
    assert!(!ind.memory.spatial_curiosity_enabled);
    assert!(ind.memory.known_sectors.is_empty());
    assert!(!ind.memory.first_interval_refuge_enabled);
    assert!(ind.memory.first_interval_outcomes.is_empty());
}
