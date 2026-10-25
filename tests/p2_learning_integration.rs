use curious::{
    creatures::{carnivore::CarnivoreEntity, herbivore::HerbivoreEntity},
    entity::{
        controller::EntityController,
        intent::Intent,
        learning::{ContextWaterEntry, WaterEnd, WaterEpisode, WaterLearner, WaterResult},
        memory::{FirstIntervalRefugeRecord, KnownSector, Memory, RefugeKey},
        perception::{PerceivedEntity, Perception},
        phase::EntityPhase,
        refuge_learning::{
            choose_refuge_by_first_interval, expected_first_interval_damage,
            record_first_interval_outcome,
        },
        species::Species,
        Entity,
    },
    logger::Logger,
    map::{
        cell::Cell,
        movement::{Direction, Position, Steps},
    },
    persistence::State,
    world::{EntitySlot, World},
};
use std::collections::VecDeque;

/// Test 1: Disabled-Path Exact State and RNG Parity
/// Verifies that two independent worlds with default configurations (all new learning/curiosity
/// features disabled) produce bit-for-bit identical state, RNG, positions, and life reserves
/// over 60 ticks.
#[test]
fn test_disabled_path_exact_state_and_rng_parity() {
    let make_world = || {
        let herbivore = EntitySlot::new(
            1,
            Position::new(0, 0),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        );
        let carnivore = EntitySlot::new(
            2,
            Position::new(3, 3),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(CarnivoreEntity::default()),
        );

        // Verify default disabled state
        assert!(!herbivore.learner.context_ttl_enabled);
        assert!(herbivore.learner.context_entries.is_empty());
        assert!(!herbivore.memory.spatial_curiosity_enabled);
        assert!(herbivore.memory.known_sectors.is_empty());
        assert!(!herbivore.memory.first_interval_refuge_enabled);
        assert!(herbivore.memory.first_interval_outcomes.is_empty());

        let mut world = World::with_seed(-8, 8, -8, 8, vec![herbivore, carnivore], 4242);
        world.logger = Logger::disabled();
        world
    };

    let mut world_a = make_world();
    let mut world_b = make_world();

    for tick in 1..=60 {
        let events_a = world_a.tick();
        let events_b = world_b.tick();

        assert_eq!(
            serde_json::to_value(&events_a).unwrap(),
            serde_json::to_value(&events_b).unwrap(),
            "Event parity divergence at tick {tick}"
        );
        assert_eq!(
            world_a.rng.state(),
            world_b.rng.state(),
            "RNG state divergence at tick {tick}"
        );
    }

    let state_a = State::capture(&world_a);
    let state_b = State::capture(&world_b);
    let json_a = serde_json::to_value(&state_a).unwrap();
    let json_b = serde_json::to_value(&state_b).unwrap();
    assert_eq!(json_a, json_b, "Captured state parity mismatch after 60 ticks");

    assert_eq!(world_a.entities.len(), world_b.entities.len());
    for (ea, eb) in world_a.entities.iter().zip(world_b.entities.iter()) {
        assert_eq!(ea.pos, eb.pos);
        assert_eq!(ea.base.life().energy, eb.base.life().energy);
        assert_eq!(ea.base.life().water, eb.base.life().water);
        assert_eq!(ea.base.life().health, eb.base.life().health);
        assert_eq!(ea.base.life().stamina, eb.base.life().stamina);
    }
}

/// Test 2: Deterministic Checkpoint Serialization and Resume
/// Enables context_ttl_enabled, spatial_curiosity_enabled, and first_interval_refuge_enabled.
/// Runs World A for 40 ticks continuously.
/// Runs World B for 20 ticks, serializes to JSON, deserializes, and runs for 20 more ticks.
/// Verifies exact state, RNG, learner context, memory sectors, and entity attributes.
#[test]
fn test_deterministic_checkpoint_serialization_and_resume() {
    let make_world = || {
        let mut herbivore = EntitySlot::new(
            1,
            Position::new(0, 0),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        );
        herbivore.learner.context_ttl_enabled = true;
        herbivore.memory.spatial_curiosity_enabled = true;
        herbivore.memory.first_interval_refuge_enabled = true;

        let mut carnivore = EntitySlot::new(
            2,
            Position::new(4, 4),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(CarnivoreEntity::default()),
        );
        carnivore.learner.context_ttl_enabled = true;
        carnivore.memory.spatial_curiosity_enabled = true;
        carnivore.memory.first_interval_refuge_enabled = true;

        let mut world = World::with_seed(-10, 10, -10, 10, vec![herbivore, carnivore], 8871);
        world.logger = Logger::disabled();
        world
    };

    let mut world_a = make_world();
    let mut world_b = make_world();

    // World A runs 40 ticks uninterrupted
    for _ in 0..40 {
        world_a.tick();
    }

    // World B runs 20 ticks
    for _ in 0..20 {
        world_b.tick();
    }

    // Checkpoint World B via serde_json string serialization and restore
    let state_b = State::capture(&world_b);
    let serialized_b = serde_json::to_string(&state_b).expect("Failed to serialize State to JSON");
    let deserialized_state: State =
        serde_json::from_str(&serialized_b).expect("Failed to deserialize State from JSON");
    let mut restored_b = deserialized_state
        .restore()
        .expect("Failed to restore World from checkpoint");
    restored_b.logger = Logger::disabled();

    // Restored World B runs the remaining 20 ticks
    for _ in 0..20 {
        restored_b.tick();
    }

    // Verify bit-for-bit parity
    let captured_a = State::capture(&world_a);
    let captured_restored_b = State::capture(&restored_b);
    let val_a = serde_json::to_value(&captured_a).unwrap();
    let val_b = serde_json::to_value(&captured_restored_b).unwrap();
    assert_eq!(
        val_a, val_b,
        "Restored checkpoint drifted from continuous execution"
    );

    assert_eq!(world_a.rng.state(), restored_b.rng.state());
    assert_eq!(world_a.tick_counter, restored_b.tick_counter);
    assert_eq!(world_a.entities.len(), restored_b.entities.len());

    for (ea, eb) in world_a.entities.iter().zip(restored_b.entities.iter()) {
        assert_eq!(ea.pos, eb.pos);
        assert_eq!(ea.base.life().energy, eb.base.life().energy);
        assert_eq!(ea.base.life().water, eb.base.life().water);
        assert_eq!(ea.base.life().health, eb.base.life().health);
        assert_eq!(ea.base.life().stamina, eb.base.life().stamina);
        assert_eq!(ea.learner.context_entries, eb.learner.context_entries);
        assert_eq!(ea.memory.known_sectors, eb.memory.known_sectors);
        assert_eq!(
            ea.memory.first_interval_outcomes,
            eb.memory.first_interval_outcomes
        );
    }
}

/// Test 3: Non-Lamarckian Offspring Isolation
/// Verifies that when parents with populated known_sectors, context_entries,
/// and first_interval_outcomes reproduce in World, the newborn offspring starts
/// with completely empty known_sectors, context_entries, and first_interval_outcomes.
#[test]
fn test_non_lamarckian_offspring_isolation() {
    let mut parent1 = EntitySlot::new(
        1,
        Position::new(0, 0),
        EntityPhase::Active,
        EntityController::Player(0),
        Box::new(HerbivoreEntity::default()),
    );
    let mut parent2 = EntitySlot::new(
        2,
        Position::new(1, 0),
        EntityPhase::Active,
        EntityController::Player(1),
        Box::new(HerbivoreEntity::default()),
    );

    parent1.base.life_mut().age = 30;
    parent1.base.life_mut().energy = 100;
    parent1.base.life_mut().water = 100;
    parent2.base.life_mut().age = 30;
    parent2.base.life_mut().energy = 100;
    parent2.base.life_mut().water = 100;

    // Enable features and populate memory/learning state on parent 1
    parent1.learner.context_ttl_enabled = true;
    parent1.memory.spatial_curiosity_enabled = true;
    parent1.memory.first_interval_refuge_enabled = true;

    parent1.memory.known_sectors.push(KnownSector {
        sector_x: 2,
        sector_y: 3,
        last_visited_tick: 10,
        visit_count: 5,
    });

    let mut obs = VecDeque::new();
    obs.push_back(18);
    parent1.learner.context_entries.push(ContextWaterEntry {
        pos: Position::new(8, 8),
        speed: 2,
        move_energy_cost: 3,
        last_seen_tick: 10,
        observations: obs,
    });

    let refuge_key = RefugeKey {
        threat_id: 99,
        threat_species: Species::Carnivore,
        bearing: 0,
        threat_distance: 1,
        energy_quartile: 2,
        stamina_quartile: 2,
        refuge_options: vec![Position::new(0, 0), Position::new(2, 0)],
        endpoint: Position::new(0, 0),
    };
    let mut episodes = VecDeque::new();
    episodes.push_back((10, 15));
    parent1
        .memory
        .first_interval_outcomes
        .push(FirstIntervalRefugeRecord {
            key: refuge_key,
            episodes,
            censored_count: 0,
        });

    assert!(!parent1.memory.known_sectors.is_empty());
    assert!(!parent1.learner.context_entries.is_empty());
    assert!(!parent1.memory.first_interval_outcomes.is_empty());

    let mut world = World::with_seed(-5, 5, -5, 5, vec![parent1, parent2], 52);
    world.logger = Logger::disabled();

    world.push_command(1, Intent::Mate { target_id: 2 });
    world.push_command(2, Intent::Mate { target_id: 1 });
    world.tick();

    // Verify offspring was born
    assert_eq!(world.entities.len(), 3, "Offspring should have been born");
    let child = world
        .entities
        .iter()
        .find(|e| e.id != 1 && e.id != 2)
        .expect("Child entity not found");

    // Non-Lamarckian isolation: newborn child starts with empty state
    assert!(
        child.memory.known_sectors.is_empty(),
        "Offspring must not inherit known_sectors"
    );
    assert!(
        child.learner.context_entries.is_empty(),
        "Offspring must not inherit context_entries"
    );
    assert!(
        child.memory.first_interval_outcomes.is_empty(),
        "Offspring must not inherit first_interval_outcomes"
    );

    // Parents retain their populated state
    assert!(!world.entities[0].memory.known_sectors.is_empty());
    assert!(!world.entities[0].learner.context_entries.is_empty());
    assert!(!world.entities[0].memory.first_interval_outcomes.is_empty());
}

/// Test 4: Active Mechanism Verification End-to-End
/// Verifies Memory::curiosity_direction, WaterLearner::predict_context_residual,
/// and FirstIntervalRefugeRecord censoring separation on active instances.
#[test]
fn test_active_mechanisms_end_to_end() {
    // ---------------------------------------------------------
    // Part A: Spatial Curiosity Direction
    // ---------------------------------------------------------
    let mut memory = Memory::default();
    memory.spatial_curiosity_enabled = true;

    let mut life = HerbivoreEntity::default().life().clone();
    life.energy = 100;
    life.max_energy = 100;
    life.water = 100;
    life.max_water = 100;
    life.stamina = 100;
    life.max_stamina = 100;

    let origin = Position::new(0, 0);
    let mut view = Perception::empty();
    // Allow moving Down (0, 1) and Right (1, 0)
    view.directions.insert(Direction::Down, 2);
    view.directions.insert(Direction::Right, 2);

    // Down targets sector (0, 1); Right targets sector (1, 0)
    // Mark sector (0, 1) as visited at tick 50
    memory.known_sectors.push(KnownSector {
        sector_x: 0,
        sector_y: 1,
        last_visited_tick: 50,
        visit_count: 1,
    });

    // Right is unvisited, so curiosity_direction should choose Right
    let chosen_dir = memory.curiosity_direction(origin, &view, &life, 60);
    assert_eq!(
        chosen_dir,
        Some(Direction::Right),
        "Curiosity should prefer unvisited sector (Right)"
    );

    // Now visit Right at tick 40 (older than Down's tick 50)
    memory.known_sectors.push(KnownSector {
        sector_x: 1,
        sector_y: 0,
        last_visited_tick: 40,
        visit_count: 1,
    });

    // Right is least recently visited (tick 40 vs tick 50), should still choose Right
    let chosen_least_recent = memory.curiosity_direction(origin, &view, &life, 60);
    assert_eq!(
        chosen_least_recent,
        Some(Direction::Right),
        "Curiosity should prefer least-recently visited sector (Right)"
    );

    // Test survival constraints
    let mut low_energy_life = life.clone();
    low_energy_life.energy = low_energy_life.max_energy / 2;
    assert_eq!(
        memory.curiosity_direction(origin, &view, &low_energy_life, 60),
        None,
        "Curiosity must abort when energy is at or below 50%"
    );

    let mut low_water_life = life.clone();
    low_water_life.water = low_water_life.max_water / 2;
    assert_eq!(
        memory.curiosity_direction(origin, &view, &low_water_life, 60),
        None,
        "Curiosity must abort when water is at or below 50%"
    );

    let mut low_stamina_life = life.clone();
    low_stamina_life.stamina = low_stamina_life.max_stamina / 3;
    assert_eq!(
        memory.curiosity_direction(origin, &view, &low_stamina_life, 60),
        None,
        "Curiosity must abort when stamina is at or below 33%"
    );

    // Test threat presence
    let mut threat_view = view.clone();
    threat_view.entities.push(PerceivedEntity {
        id: 99,
        species: Species::Carnivore,
        power: 10,
        steps: Steps(vec![Direction::Down]),
        mature: true,
        health: Some(100),
        reproductive: false,
    });
    assert_eq!(
        memory.curiosity_direction(origin, &threat_view, &life, 60),
        None,
        "Curiosity must abort when carnivore is visible"
    );

    let mut threat_mem = memory.clone();
    threat_mem.dangers.push((Position::new(1, 1), 55));
    assert_eq!(
        threat_mem.curiosity_direction(origin, &view, &life, 60),
        None,
        "Curiosity must abort when remembered dangers exist"
    );

    // ---------------------------------------------------------
    // Part B: WaterLearner Context-Aligned Residual & TTL
    // ---------------------------------------------------------
    let mut learner = WaterLearner::default();
    learner.context_ttl_enabled = true;

    let water_pos = Position::new(10, 10);
    let speed = 2u8;
    let move_cost = 3u16;

    // Record first observation of net water delta +40 at tick 10
    learner.record_context_observation(water_pos, speed, move_cost, 10, 40);
    assert_eq!(
        learner.predict_context_residual(water_pos, speed, move_cost, 15),
        Some(40),
        "Initial context observation should predict exact value"
    );

    // Record second observation +20 at tick 20: average becomes (40 + 20) / 2 = 30
    learner.record_context_observation(water_pos, speed, move_cost, 20, 20);
    assert_eq!(
        learner.predict_context_residual(water_pos, speed, move_cost, 25),
        Some(30),
        "Sliding window average should update prediction to 30"
    );

    // Context mismatch: different speed or move cost or position returns None
    assert_eq!(
        learner.predict_context_residual(water_pos, 1, move_cost, 25),
        None,
        "Speed mismatch must return None"
    );
    assert_eq!(
        learner.predict_context_residual(water_pos, speed, 5, 25),
        None,
        "Move cost mismatch must return None"
    );
    assert_eq!(
        learner.predict_context_residual(Position::new(0, 0), speed, move_cost, 25),
        None,
        "Position mismatch must return None"
    );

    // TTL expiry: entry last_seen_tick is 20, TTL is 48 ticks.
    // At tick 20 + 48 = 68, it is valid (diff <= 48).
    // At tick 20 + 49 = 69, it is expired.
    assert!(learner
        .predict_context_residual(water_pos, speed, move_cost, 68)
        .is_some());
    assert_eq!(
        learner.predict_context_residual(water_pos, speed, move_cost, 69),
        None,
        "Residual prediction must return None once TTL exceeds 48 ticks"
    );

    // Prune expired context
    assert_eq!(learner.context_entries.len(), 1);
    learner.prune_expired_context(70);
    assert_eq!(
        learner.context_entries.len(),
        0,
        "prune_expired_context must remove expired entries"
    );

    // ---------------------------------------------------------
    // Part C: First-Interval Refuge Damage Censoring Separation
    // ---------------------------------------------------------
    let mut records: Vec<FirstIntervalRefugeRecord> = Vec::new();
    let r_key = RefugeKey {
        threat_id: 5,
        threat_species: Species::Carnivore,
        bearing: 0,
        threat_distance: 1,
        energy_quartile: 2,
        stamina_quartile: 2,
        refuge_options: vec![Position::new(0, 0), Position::new(4, 4)],
        endpoint: Position::new(0, 0),
    };

    // First interval damage observed as 30 at tick 10
    record_first_interval_outcome(&mut records, r_key.clone(), Some(30), 10);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].censored_count, 0);
    assert_eq!(
        expected_first_interval_damage(&records, &r_key),
        Some(30.0)
    );

    // Route was censored subsequently (outcome: None)
    record_first_interval_outcome(&mut records, r_key.clone(), None, 15);
    assert_eq!(records.len(), 1);
    assert_eq!(
        records[0].censored_count, 1,
        "Censored outcome must increment censored_count"
    );
    assert_eq!(
        expected_first_interval_damage(&records, &r_key),
        Some(30.0),
        "Censored outcome must NOT overwrite or inject fabricated 0 damage"
    );

    // Another completed episode dealt 10 damage: average is (30 + 10) / 2 = 20
    record_first_interval_outcome(&mut records, r_key.clone(), Some(10), 20);
    assert_eq!(
        expected_first_interval_damage(&records, &r_key),
        Some(20.0),
        "Expected damage should average known episodes"
    );
    assert_eq!(records[0].censored_count, 1);

    // Candidate ranking prefers lower damage candidate
    let r_key2 = RefugeKey {
        threat_id: 5,
        threat_species: Species::Carnivore,
        bearing: 0,
        threat_distance: 1,
        energy_quartile: 2,
        stamina_quartile: 2,
        refuge_options: vec![Position::new(4, 4)],
        endpoint: Position::new(4, 4),
    };
    record_first_interval_outcome(&mut records, r_key2.clone(), Some(40), 20);

    // Candidates [r_key2 (damage 40), r_key (damage 20)] -> r_key at index 1 is preferred
    let candidates = vec![r_key2.clone(), r_key.clone()];
    assert_eq!(
        choose_refuge_by_first_interval(&records, &candidates),
        Some(1),
        "Candidate with lower expected first-interval damage should be selected"
    );
}

/// Test 5: Context TTL in normal World::tick populates context_entries and alters choice
#[test]
fn test_context_ttl_world_tick_populates_and_alters_choice() {
    // Part 1: Verify world.tick() populates context_entries after drinking
    let mut herbivore = EntitySlot::new(
        1,
        Position::new(0, 0),
        EntityPhase::Active,
        EntityController::AI,
        Box::new(HerbivoreEntity::default()),
    );
    herbivore.base.life_mut().water = 10; // low water triggers drinking
    herbivore.base.life_mut().energy = 70;
    herbivore.learner.context_ttl_enabled = true;
    herbivore.learner.update_enabled = true;

    let mut world = World::with_seed(-5, 5, -5, 5, vec![herbivore], 100);
    world.logger = Logger::disabled();
    world.map = curious::map::Map::new(-5, 5, -5, 5);
    world.map.set_cell(Position::new(1, 0), Cell::Water { amount: 30 });
    world.map.set_cell(Position::new(0, 1), Cell::Water { amount: 30 });

    // Tick 1: AI herbivore chooses a water source and drinks
    let _events = world.tick();

    // Verify context_entries is populated by world.tick()
    assert!(
        !world.entities[0].learner.context_entries.is_empty(),
        "world.tick() must populate context_entries when context_ttl_enabled = true"
    );

    // Part 2: Verify context residual alters water source selection
    let make_choice_world = |context_enabled: bool, with_penalty: bool| {
        let mut slot = EntitySlot::new(
            1,
            Position::new(0, 0),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        );
        slot.base.life_mut().water = 10;
        slot.base.life_mut().energy = 70;
        slot.learner.context_ttl_enabled = context_enabled;
        slot.learner.update_enabled = true;

        if with_penalty {
            let speed = slot.base.life().speed.max(1) as u8;
            let move_cost = 1u16 + slot.genome.mobility as u16;
            // Record a negative residual for (1, 0)
            let mut obs = VecDeque::new();
            obs.push_back(-500);
            slot.learner.context_entries.push(ContextWaterEntry {
                pos: Position::new(1, 0),
                speed,
                move_energy_cost: move_cost,
                last_seen_tick: 0,
                observations: obs,
            });
        }

        let mut w = World::with_seed(-5, 5, -5, 5, vec![slot], 42);
        w.logger = Logger::disabled();
        w.map = curious::map::Map::new(-5, 5, -5, 5);
        w.map.set_cell(Position::new(1, 0), Cell::Water { amount: 20 });
        w.map.set_cell(Position::new(0, 1), Cell::Water { amount: 20 });
        w
    };

    let mut w_disabled = make_choice_world(false, false);
    let mut w_enabled_penalized = make_choice_world(true, true);

    w_disabled.tick();
    w_enabled_penalized.tick();

    // Verify disabled world chose without context residual, and context_entries is empty
    assert!(
        w_disabled.entities[0].learner.context_entries.is_empty(),
        "Disabled world must not populate context_entries"
    );

    // The two worlds made different decisions due to the context residual
    let dec_disabled = w_disabled.last_decisions[0].decision.intent.clone();
    let dec_enabled = w_enabled_penalized.last_decisions[0].decision.intent.clone();
    assert_ne!(
        dec_disabled, dec_enabled,
        "context_ttl_enabled must alter water source selection when residual is present"
    );
}

/// Test 6: Spatial curiosity in normal World::tick populates sectors and alters movement trajectory
#[test]
fn test_spatial_curiosity_world_tick_populates_sectors_and_alters_trajectory() {
    let make_curiosity_world = |curiosity_enabled: bool| {
        let mut slot = EntitySlot::new(
            1,
            Position::new(0, 0),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        );
        slot.base.life_mut().energy = 80;
        slot.base.life_mut().max_energy = 80;
        slot.base.life_mut().water = 60;
        slot.base.life_mut().max_water = 60;
        slot.base.life_mut().stamina = 16;
        slot.base.life_mut().max_stamina = 16;
        slot.memory.spatial_curiosity_enabled = curiosity_enabled;

        // In curiosity-enabled case, pre-populate sectors Down, Up, Left so Right is the only unvisited adjacent sector
        if curiosity_enabled {
            slot.memory.known_sectors.push(KnownSector {
                sector_x: 0,
                sector_y: 1, // Down
                last_visited_tick: 50,
                visit_count: 2,
            });
            slot.memory.known_sectors.push(KnownSector {
                sector_x: 0,
                sector_y: -1, // Up
                last_visited_tick: 50,
                visit_count: 2,
            });
            slot.memory.known_sectors.push(KnownSector {
                sector_x: -1,
                sector_y: 0, // Left
                last_visited_tick: 50,
                visit_count: 2,
            });
        }

        let mut w = World::with_seed(-10, 10, -10, 10, vec![slot], 77);
        w.logger = Logger::disabled();
        w
    };

    let mut w_disabled = make_curiosity_world(false);
    let mut w_enabled = make_curiosity_world(true);

    // Run 5 ticks of world simulation
    for _ in 0..5 {
        w_disabled.tick();
        w_enabled.tick();
    }

    // 1. Verify known_sectors is populated via world.tick() when enabled, and empty when disabled
    assert!(
        !w_enabled.entities[0].memory.known_sectors.is_empty(),
        "known_sectors must be populated when spatial_curiosity_enabled is true"
    );
    assert!(
        w_disabled.entities[0].memory.known_sectors.is_empty(),
        "known_sectors must stay empty when spatial_curiosity_enabled is false"
    );

    // 2. Trajectories diverged because spatial curiosity directed movement towards unvisited sectors
    assert_ne!(
        w_disabled.entities[0].pos,
        w_enabled.entities[0].pos,
        "spatial_curiosity_enabled must produce a different movement trajectory"
    );
}

/// Test 7: First-interval refuge learning preserves damage on censored routes and alters choice
#[test]
fn test_first_interval_refuge_world_tick_preserves_outcome_and_alters_choice() {
    // -------------------------------------------------------------------------
    // Part 1: First-interval damage is preserved even if threat is lost on tick 2
    // -------------------------------------------------------------------------
    let mut herbivore = EntitySlot::new(
        1,
        Position::new(3, 2),
        EntityPhase::Active,
        EntityController::AI,
        Box::new(HerbivoreEntity::default()),
    );
    herbivore.base.life_mut().speed = 1;
    herbivore.base.life_mut().health = 100;
    herbivore.base.life_mut().energy = 70;
    herbivore.base.life_mut().stamina = 16;
    herbivore.memory.first_interval_refuge_enabled = true;

    let mut threat = EntitySlot::new(
        2,
        Position::new(3, 1),
        EntityPhase::Active,
        EntityController::AI,
        Box::new(CarnivoreEntity::default()),
    );
    threat.base.life_mut().speed = 1;
    threat.base.life_mut().energy = 30;

    let mut world = World::with_seed(-5, 10, -5, 10, vec![herbivore, threat], 33);
    world.logger = Logger::disabled();
    world.map = curious::map::Map::new(-5, 10, -5, 10);
    // Burrow A at (0, 2)
    world.map.set_cell(Position::new(0, 2), Cell::Burrow);

    // Tick 1: Threat attacks herbivore; herbivore flees towards burrow
    world.tick();

    // Verify first-interval outcome is already recorded on tick 1
    assert_eq!(
        world.entities[0].memory.first_interval_outcomes.len(), 1,
        "First-interval outcome must be recorded after the first resolver interval"
    );
    let first_interval_damage = world.entities[0].memory.first_interval_outcomes[0].episodes[0].1;
    assert!(
        first_interval_damage > 0,
        "First-interval damage must reflect threat attack damage"
    );

    // Tick 2: Move threat far away so threat sight is lost (ThreatLost closure)
    world.entities[1].pos = Position::new(9, 9);
    world.tick();
    world.tick(); // Threat unseen ticks >= 2 triggers ThreatLost closure

    // Episode is closed (no longer active)
    assert!(
        world.entities[0].memory.refuge_episode.is_none(),
        "Episode should close after losing sight of threat"
    );
    // Crucial check: First interval record was NOT lost or wiped out by route censoring!
    assert_eq!(
        world.entities[0].memory.first_interval_outcomes.len(), 1,
        "First interval outcome must be retained even when full route is censored"
    );
    assert_eq!(
        world.entities[0].memory.first_interval_outcomes[0].episodes[0].1,
        first_interval_damage
    );

    // -------------------------------------------------------------------------
    // Part 2: First-interval learning alters refuge choice on subsequent encounter
    // -------------------------------------------------------------------------
    let make_refuge_world = |safer_burrow_is_a: bool| {
        let mut h = EntitySlot::new(
            1,
            Position::new(1, 1),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        );
        h.base.life_mut().speed = 1;
        h.base.life_mut().health = 100;
        h.base.life_mut().energy = 70;
        h.base.life_mut().stamina = 16;
        h.memory.first_interval_refuge_enabled = true;

        let mut t = EntitySlot::new(
            2,
            Position::new(1, 2),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(CarnivoreEntity::default()),
        );
        t.base.life_mut().speed = 1;
        t.base.life_mut().energy = 70;

        let mut w = World::with_seed(-5, 10, -5, 10, vec![h, t], 42);
        w.logger = Logger::disabled();
        w.map = curious::map::Map::new(-5, 10, -5, 10);
        w.map.set_cell(Position::new(0, 1), Cell::Burrow); // Burrow A (Left)
        w.map.set_cell(Position::new(1, 0), Cell::Burrow); // Burrow B (Up)

        if safer_burrow_is_a {
            // Pre-seed first_interval_outcomes: Burrow A had 5 damage, Burrow B had 35 damage
            let key_a = RefugeKey {
                threat_id: 2,
                threat_species: Species::Carnivore,
                bearing: 4,
                threat_distance: 0,
                energy_quartile: 3,
                stamina_quartile: 3,
                refuge_options: vec![Position::new(1, 0), Position::new(0, 1)],
                endpoint: Position::new(0, 1),
            };
            let key_b = RefugeKey {
                threat_id: 2,
                threat_species: Species::Carnivore,
                bearing: 4,
                threat_distance: 0,
                energy_quartile: 3,
                stamina_quartile: 3,
                refuge_options: vec![Position::new(1, 0), Position::new(0, 1)],
                endpoint: Position::new(1, 0),
            };
            record_first_interval_outcome(&mut w.entities[0].memory.first_interval_outcomes, key_a, Some(5), 0);
            record_first_interval_outcome(&mut w.entities[0].memory.first_interval_outcomes, key_b, Some(35), 0);
        }
        w
    };

    let mut w_uninformed = make_refuge_world(false);
    let mut w_informed = make_refuge_world(true);

    w_uninformed.tick();
    w_informed.tick();

    let dec_uninformed = w_uninformed.last_decisions[0].decision.intent.clone();
    let dec_informed = w_informed.last_decisions[0].decision.intent.clone();

    // The informed herbivore chose Burrow A (Left, lower damage 5) while uninformed chose Burrow B (Up, fallback tie-breaker)
    assert_ne!(
        dec_uninformed, dec_informed,
        "first_interval_refuge_enabled must alter shelter choice based on learned damage"
    );
    assert_eq!(
        dec_informed,
        Intent::Move { steps: Steps(vec![Direction::Left]) },
        "Informed herbivore must move towards the safer shelter (Burrow A at 0, 1)"
    );
}

/// Test 8: F7 context overflow validation and safe sum
#[test]
fn test_f7_context_overflow_validation_and_safe_sum() {
    let mut learner = WaterLearner::default();
    learner.context_ttl_enabled = true;
    let mut obs = VecDeque::new();
    obs.push_back(i32::MAX);
    obs.push_back(i32::MAX);
    learner.context_entries.push(ContextWaterEntry {
        pos: Position::new(2, 2),
        speed: 2,
        move_energy_cost: 3,
        last_seen_tick: 10,
        observations: obs,
    });

    // 1. Validate must reject out-of-bounds observation values (> 1000)
    assert!(
        learner.validate().is_err(),
        "Learner::validate must return Err for unvalidated i32::MAX context observations"
    );

    // 2. predict_context_residual must NOT panic on i32::MAX values, computing sum safely via i64
    let pred = learner.predict_context_residual(Position::new(2, 2), 2, 3, 15);
    assert_eq!(
        pred,
        Some(i32::MAX),
        "predict_context_residual must calculate average without integer overflow panic"
    );

    // 3. record_context_observation clamps input to [-1000, 1000]
    let mut safe_learner = WaterLearner::default();
    safe_learner.context_ttl_enabled = true;
    safe_learner.record_context_observation(Position::new(3, 3), 2, 3, 10, i32::MAX);
    assert!(
        safe_learner.validate().is_ok(),
        "Learner::validate must pass after record_context_observation clamps value"
    );
    assert_eq!(
        safe_learner.context_entries[0].observations[0], 1000,
        "record_context_observation must clamp i32::MAX to 1000"
    );
}

/// Test 9: F1 recheck identical consecutive episodes are not skipped and use physical residual scale
#[test]
fn test_f1_recheck_identical_consecutive_episodes_are_not_skipped_and_use_physical_residual_scale() {
    // 1. Direct unit tests of episode_physical_residual across outcomes and censoring
    let make_result = |end: WaterEnd,
                       water: usize,
                       move_energy: usize,
                       damage: usize,
                       baseline: Option<i64>,
                       overflowed: bool,
                       censored: bool| {
        WaterResult {
            attempt: WaterEpisode {
                target: Position::new(1, 0),
                ticks: 1,
                movement_energy: move_energy,
                incoming_damage: damage,
                choice_baseline: baseline,
                outcome_overflowed: overflowed,
                candidate_censored: censored,
            },
            water,
            end,
            reward: Some(123),
        }
    };

    // Successful Reached outcome: net = 25 - 5 - 2 = 18; residual = 18 - 10 = 8
    let res_reached = make_result(WaterEnd::Reached, 25, 5, 2, Some(10), false, false);
    assert_eq!(
        WaterLearner::episode_physical_residual(&res_reached),
        Some(8),
        "Physical residual must equal net outcome minus baseline"
    );

    // Missing resource / Failed outcome: net = 0 - 3 - 0 = -3; residual = -3 - 5 = -8
    let res_failed = make_result(WaterEnd::Failed, 0, 3, 0, Some(5), false, false);
    assert_eq!(
        WaterLearner::episode_physical_residual(&res_failed),
        Some(-8),
        "Failed missing resource must compute physical residual from net loss"
    );

    // Non-qualifying ends return None
    let res_interrupted = make_result(WaterEnd::Interrupted, 10, 0, 0, Some(5), false, false);
    assert_eq!(WaterLearner::episode_physical_residual(&res_interrupted), None);
    let res_timed_out = make_result(WaterEnd::TimedOut, 0, 10, 0, Some(5), false, false);
    assert_eq!(WaterLearner::episode_physical_residual(&res_timed_out), None);

    // Censored or overflowed attempts return None
    let res_censored = make_result(WaterEnd::Reached, 25, 5, 2, Some(10), false, true);
    assert_eq!(WaterLearner::episode_physical_residual(&res_censored), None);
    let res_overflow = make_result(WaterEnd::Reached, 25, 5, 2, Some(10), true, false);
    assert_eq!(WaterLearner::episode_physical_residual(&res_overflow), None);

    // 2. Integration test in World::tick:
    // Run two identical consecutive water-drinking episodes with the exact same
    // water gain, movement energy, and ticks.
    let mut herbivore = EntitySlot::new(
        1,
        Position::new(0, 0),
        EntityPhase::Active,
        EntityController::AI,
        Box::new(HerbivoreEntity::default()),
    );
    herbivore.base.life_mut().water = 10;
    herbivore.base.life_mut().energy = 70;
    herbivore.learner.context_ttl_enabled = true;
    herbivore.learner.update_enabled = true;

    let mut world = World::with_seed(-5, 5, -5, 5, vec![herbivore], 100);
    world.logger = Logger::disabled();
    world.map = curious::map::Map::new(-5, 5, -5, 5);
    world.map.set_cell(Position::new(1, 0), Cell::Water { amount: 1 });

    // Tick 1: AI herbivore chooses (1, 0) and drinks 1 water
    world.tick();

    let ent = &world.entities[0];
    let last_ep1 = ent
        .learner
        .last_episode
        .as_ref()
        .expect("Episode 1 must complete");
    assert_eq!(last_ep1.water, 1);
    assert_eq!(last_ep1.attempt.movement_energy, 0);
    assert_eq!(last_ep1.attempt.incoming_damage, 0);
    let baseline1 = last_ep1
        .attempt
        .choice_baseline
        .expect("choice_baseline must be captured");
    let net1 = last_ep1.water as i64
        - last_ep1.attempt.movement_energy as i64
        - last_ep1.attempt.incoming_damage as i64;
    let expected_residual1 = (net1 - baseline1).clamp(-1000, 1000) as i32;

    assert_eq!(
        ent.learner.context_entries.len(),
        1,
        "First episode must create a context entry"
    );
    assert_eq!(
        ent.learner.context_entries[0].observations.len(),
        1,
        "First episode must record exactly 1 observation"
    );
    assert_eq!(
        ent.learner.context_entries[0].observations[0], expected_residual1,
        "Observation 1 must equal the physical residual"
    );
    // Reinforcement reward would be: 1 * 10 - 0 * 2 - 0 = 10
    let reward1 = last_ep1.reward.unwrap_or(0);
    assert_ne!(
        ent.learner.context_entries[0].observations[0], reward1,
        "Context observation must be on the physical residual scale, NOT the reward scale"
    );

    // Setup identical conditions for Episode 2: reset water to 10 and map water cell to 1
    world.entities[0].base.life_mut().water = 10;
    world.entities[0].base.life_mut().energy = 70;
    world.map.set_cell(Position::new(1, 0), Cell::Water { amount: 1 });

    // Tick 2: AI herbivore drinks identical water at (1, 0)
    world.tick();

    let ent2 = &world.entities[0];
    let last_ep2 = ent2
        .learner
        .last_episode
        .as_ref()
        .expect("Episode 2 must complete");
    assert_eq!(last_ep2.water, 1);
    assert_eq!(last_ep2.attempt.movement_energy, 0);
    assert_eq!(last_ep2.attempt.incoming_damage, 0);
    let baseline2 = last_ep2
        .attempt
        .choice_baseline
        .expect("choice_baseline must be captured");
    assert_eq!(
        baseline1, baseline2,
        "Identical conditions produce identical baseline"
    );

    // CRITICAL ASSERTION: The second identical episode MUST NOT be skipped!
    assert_eq!(
        ent2.learner.context_entries[0].observations.len(),
        2,
        "Identical consecutive episode MUST NOT be skipped; observations.len() must be 2"
    );
    assert_eq!(
        ent2.learner.context_entries[0].observations[1], expected_residual1,
        "Observation 2 must equal the exact same physical residual as observation 1"
    );
}
