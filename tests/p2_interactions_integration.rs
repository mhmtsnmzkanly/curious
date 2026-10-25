//! Comprehensive P2 Integration Tests for K2/K3/K4/K6 Juvenile Care, Ambush Predation, and Local Social Coordination.

use curious::{
    creatures::{carnivore::CarnivoreEntity, herbivore::HerbivoreEntity},
    entity::{controller::EntityController, phase::EntityPhase, Entity},
    environment::SignalKind,
    map::{cell::Cell, movement::Position, Map},
    policy::PolicySettings,
    simulation::checkpoint::State,
    systems::{interaction::InteractionSystem, perception::PerceptionSystem},
    world::{AttackPlan, EntitySlot, World},
};

#[test]
fn test_1_disabled_path_exact_state_and_rng_parity() {
    let seed = 4242;
    let build_world = |flags_explicit: bool| -> World {
        let mut h1 = Box::new(HerbivoreEntity::default());
        h1.life_mut().energy = 60;
        h1.life_mut().health = 100;
        let mut c1 = Box::new(CarnivoreEntity::default());
        c1.life_mut().energy = 70;
        c1.life_mut().health = 120;
        let mut h2 = Box::new(HerbivoreEntity::default());
        h2.life_mut().energy = 80;
        h2.life_mut().health = 120;

        let slots = vec![
            EntitySlot::new(1, Position::new(0, 0), EntityPhase::Active, EntityController::AI, h1),
            EntitySlot::new(2, Position::new(3, 3), EntityPhase::Active, EntityController::AI, c1),
            EntitySlot::new(3, Position::new(-4, -4), EntityPhase::Active, EntityController::AI, h2),
        ];

        let mut world = World::with_seed(-10, 10, -10, 10, slots, seed);
        world.logger = curious::logger::Logger::disabled();
        if flags_explicit {
            world.policy.juvenile_care_enabled = false;
            world.policy.ambush_predation_enabled = false;
            world.policy.local_coordination_enabled = false;
        }
        world
    };

    let mut w_default = build_world(false);
    let mut w_explicit = build_world(true);

    assert!(!w_default.policy.juvenile_care_enabled);
    assert!(!w_default.policy.ambush_predation_enabled);
    assert!(!w_default.policy.local_coordination_enabled);

    for tick in 1..=60 {
        let ev1 = w_default.tick();
        let ev2 = w_explicit.tick();
        assert_eq!(ev1.len(), ev2.len(), "Events count mismatch at tick {tick}");
        assert_eq!(
            w_default.rng.state(),
            w_explicit.rng.state(),
            "RNG stream divergence at tick {tick}"
        );
        let s1 = serde_json::to_value(State::capture(&w_default)).unwrap();
        let s2 = serde_json::to_value(State::capture(&w_explicit)).unwrap();
        assert_eq!(s1, s2, "State divergence at tick {tick}");
    }
}

#[test]
fn test_2_deterministic_checkpoint_serialization_and_resume() {
    // A. Check that policy flags serialize when true and roundtrip accurately
    let mut policy = PolicySettings::default();
    policy.juvenile_care_enabled = true;
    policy.ambush_predation_enabled = true;
    policy.local_coordination_enabled = true;

    let json_str = serde_json::to_string(&policy).unwrap();
    assert!(json_str.contains("juvenile_care_enabled"));
    assert!(json_str.contains("ambush_predation_enabled"));
    assert!(json_str.contains("local_coordination_enabled"));

    let restored_policy: PolicySettings = serde_json::from_str(&json_str).unwrap();
    assert_eq!(policy, restored_policy);

    // B. Check that default (all false) flags are omitted via skip_serializing_if
    let default_policy = PolicySettings::default();
    let default_json = serde_json::to_string(&default_policy).unwrap();
    assert!(!default_json.contains("juvenile_care_enabled"));
    assert!(!default_json.contains("ambush_predation_enabled"));
    assert!(!default_json.contains("local_coordination_enabled"));

    // C. Legacy JSON without these fields deserializes with all 3 flags defaulting to false
    let legacy_json = r#"{"mate_rendezvous":false,"reserve_opportunities":false}"#;
    let legacy_policy: PolicySettings = serde_json::from_str(legacy_json).unwrap();
    assert!(!legacy_policy.juvenile_care_enabled);
    assert!(!legacy_policy.ambush_predation_enabled);
    assert!(!legacy_policy.local_coordination_enabled);

    // D. World State capture, restore and continuation deterministic equivalence
    let h1 = Box::new(HerbivoreEntity::default());
    let c1 = Box::new(CarnivoreEntity::default());
    let slots = vec![
        EntitySlot::new(1, Position::new(1, 1), EntityPhase::Active, EntityController::AI, h1),
        EntitySlot::new(2, Position::new(-2, 2), EntityPhase::Active, EntityController::AI, c1),
    ];
    let mut world = World::with_seed(-10, 10, -10, 10, slots, 9999);
    world.logger = curious::logger::Logger::disabled();
    world.policy.juvenile_care_enabled = true;
    world.policy.ambush_predation_enabled = true;
    world.policy.local_coordination_enabled = true;

    for _ in 0..10 {
        world.tick();
    }

    let checkpoint = State::capture(&world);
    let mut resumed = checkpoint.restore().unwrap();

    for tick in 1..=20 {
        let ev1 = world.tick();
        let ev2 = resumed.tick();
        assert_eq!(ev1.len(), ev2.len(), "Resume tick {tick} event mismatch");
        assert_eq!(world.rng.state(), resumed.rng.state(), "Resume tick {tick} RNG mismatch");
        assert_eq!(
            serde_json::to_value(State::capture(&world)).unwrap(),
            serde_json::to_value(State::capture(&resumed)).unwrap(),
            "Resume tick {tick} State mismatch"
        );
    }
}

#[test]
fn test_3_juvenile_parental_care_energy_conservation_and_zero_lamarckian_copy() {
    // Setup adult parent (mature, age 30 >= maturity_age 20, energy 85 > 75% of 100)
    let mut parent_herb = Box::new(HerbivoreEntity::default());
    parent_herb.life_mut().age = 30;
    parent_herb.life_mut().maturity_age = 20;
    parent_herb.life_mut().energy = 85;
    parent_herb.life_mut().max_energy = 100;
    parent_herb.life_mut().stamina = 16;
    parent_herb.life_mut().max_stamina = 16;

    let mut parent_slot = EntitySlot::new(
        1,
        Position::new(0, 0),
        EntityPhase::Active,
        EntityController::AI,
        parent_herb,
    );
    // Populate parent memory with unique learned data to ensure it's not copied to offspring
    parent_slot.memory.dangers.push((Position::new(99, 99), 10));

    // Setup juvenile offspring (immature, age 5 < maturity_age 20, energy 30 < 50% of 100)
    let mut child_herb = Box::new(HerbivoreEntity::default());
    child_herb.life_mut().age = 5;
    child_herb.life_mut().maturity_age = 20;
    child_herb.life_mut().energy = 30;
    child_herb.life_mut().max_energy = 100;
    child_herb.life_mut().health = 40;

    let mut child_slot = EntitySlot::new(
        2,
        Position::new(1, 0), // adjacent (dx=1, dy=0)
        EntityPhase::Active,
        EntityController::AI,
        child_herb,
    );
    child_slot.parents = vec![1]; // Offspring of parent 1

    // Setup another juvenile that is NOT child of parent 1
    let mut stranger_herb = Box::new(HerbivoreEntity::default());
    stranger_herb.life_mut().age = 5;
    stranger_herb.life_mut().maturity_age = 20;
    stranger_herb.life_mut().energy = 30;
    stranger_herb.life_mut().max_energy = 100;

    let mut stranger_slot = EntitySlot::new(
        3,
        Position::new(0, 1), // adjacent
        EntityPhase::Active,
        EntityController::AI,
        stranger_herb,
    );
    stranger_slot.parents = vec![99]; // Not parent 1

    let mut entities = vec![parent_slot, child_slot, stranger_slot];
    let mut events = Vec::new();

    let initial_parent_energy = entities[0].base.life().energy;
    let initial_child_energy = entities[1].base.life().energy;
    let initial_stranger_energy = entities[2].base.life().energy;

    InteractionSystem::resolve_parental_care(&mut entities, &mut events);

    let final_parent_energy = entities[0].base.life().energy;
    let final_child_energy = entities[1].base.life().energy;
    let final_stranger_energy = entities[2].base.life().energy;

    let parent_delta = final_parent_energy as i64 - initial_parent_energy as i64;
    let child_delta = final_child_energy as i64 - initial_child_energy as i64;

    // 1. Exact energy conservation: parent transferred 10, child received 10
    assert_eq!(parent_delta, -10);
    assert_eq!(child_delta, 10);
    assert_eq!(parent_delta + child_delta, 0, "Stock-flow energy conservation must be exact");

    // 2. Stranger juvenile received 0 energy
    assert_eq!(final_stranger_energy, initial_stranger_energy);

    // 3. Zero Lamarckian memory / learner copy
    assert!(entities[1].memory.dangers.is_empty(), "Learned memory must NOT be copied to offspring");
    assert_eq!(entities[1].parents, vec![1]);

    // 4. Test parental guarding defense against attacks
    let is_guarded = InteractionSystem::parent_guards_juvenile(&entities, 2);
    assert!(is_guarded, "Mature adjacent parent with sufficient stamina guards juvenile");

    let not_guarded_stranger = InteractionSystem::parent_guards_juvenile(&entities, 3);
    assert!(!not_guarded_stranger, "Non-offspring juvenile is not guarded");

    // Test resolve_attacks_with_policy damage halving (damage: 20 -> 10)
    let mut carnivore = Box::new(CarnivoreEntity::default());
    carnivore.life_mut().energy = 50;
    carnivore.life_mut().stamina = 16;
    let attacker_slot = EntitySlot::new(
        10,
        Position::new(2, 0), // adjacent to child at (1, 0)
        EntityPhase::Active,
        EntityController::AI,
        carnivore,
    );
    entities.push(attacker_slot);

    let attack_plan = vec![AttackPlan {
        attacker_id: 10,
        target_id: 2,
    }];

    let map = Map::new(-5, 5, -5, 5);
    let mut attack_events = Vec::new();
    let mut policy = PolicySettings::default();
    policy.juvenile_care_enabled = true;

    let child_health_before = entities[1].base.life().health;
    let parent_stamina_before = entities[0].base.life().stamina;

    InteractionSystem::resolve_attacks_with_policy(
        &mut entities,
        &attack_plan,
        &map,
        &mut attack_events,
        &policy,
    );

    let child_health_after = entities[1].base.life().health;
    let parent_stamina_after = entities[0].base.life().stamina;

    // Base carnivore damage is 20; halved by parental guard to 10
    assert_eq!(child_health_before - child_health_after, 10, "Incoming attack damage must be halved by 50%");
    assert_eq!(parent_stamina_before - parent_stamina_after, 4, "Guarding parent must exert 4 stamina");
}

#[test]
fn test_4_ambush_stealth_and_local_alarm_emission() {
    // Part A: Ambush Stealth in PerceptionSystem
    // Setup predator on Cell::Burrow with full stamina
    let mut carnivore = Box::new(CarnivoreEntity::default());
    carnivore.life_mut().stamina = 16;
    carnivore.life_mut().max_stamina = 16;
    let predator_slot = EntitySlot::new(
        2,
        Position::new(0, 5), // At distance 5 from observer at (0, 0)
        EntityPhase::Active,
        EntityController::AI,
        carnivore,
    );

    // Setup observer herbivore at (0, 0) with vision_range = 5
    let mut observer_herb = Box::new(HerbivoreEntity::default());
    observer_herb.life_mut().vision_range = 5;
    let observer_slot = EntitySlot::new(
        1,
        Position::new(0, 0),
        EntityPhase::Active,
        EntityController::AI,
        observer_herb,
    );

    let mut world = World::with_seed(-10, 10, -10, 10, vec![observer_slot, predator_slot], 12345);
    world.logger = curious::logger::Logger::disabled();
    // Place Burrow (shelter) at predator's position
    world.map.set_cell(Position::new(0, 5), Cell::Burrow);

    // When policy.ambush_predation_enabled is false: predator at distance 5 IS detected
    world.policy.ambush_predation_enabled = false;
    let view_disabled = PerceptionSystem::build(&world, &world.entities[0]);
    assert!(
        view_disabled.entities.iter().any(|e| e.id == 2),
        "Predator at distance 5 must be perceived when ambush predation is disabled"
    );

    // When policy.ambush_predation_enabled is true: detectability radius reduced by 1
    // (max detect = 5 - 1 = 4), so predator at distance 5 is NOT detected
    world.policy.ambush_predation_enabled = true;
    let view_enabled = PerceptionSystem::build(&world, &world.entities[0]);
    assert!(
        !view_enabled.entities.iter().any(|e| e.id == 2),
        "Predator in shelter with full stamina must have detectability radius reduced by 1 (unspotted at dist 5)"
    );

    // If predator is closer (distance 4), it IS detected even when stealthed
    world.entities[1].pos = Position::new(0, 4);
    world.map.set_cell(Position::new(0, 4), Cell::Burrow);
    let view_close = PerceptionSystem::build(&world, &world.entities[0]);
    assert!(
        view_close.entities.iter().any(|e| e.id == 2),
        "Stealthed predator at distance <= 4 must be spotted"
    );

    // Part B: Ambush strike damage bonus (+5)
    let mut attack_events = Vec::new();
    let attack_plan = vec![AttackPlan {
        attacker_id: 2,
        target_id: 1,
    }];
    // Put predator adjacent to observer at (0, 1), on Cell::Burrow with full stamina
    world.entities[1].pos = Position::new(0, 1);
    world.entities[0].base.life_mut().health = 100;
    world.entities[1].base.life_mut().stamina = 16;
    world.entities[1].base.life_mut().max_stamina = 16;
    world.map.set_cell(Position::new(0, 1), Cell::Burrow);

    InteractionSystem::resolve_attacks_with_policy(
        &mut world.entities,
        &attack_plan,
        &world.map,
        &mut attack_events,
        &world.policy,
    );

    // Base carnivore damage is 20 + ambush strike bonus 5 = 25
    assert_eq!(world.entities[0].base.life().health, 75, "Ambush strike must deal base 20 + 5 bonus = 25 damage");

    // Part C: Local Alarm Emission in InteractionSystem
    let speaker_id = 1; // Herbivore at (0, 0)
    world.entities[0].base.life_mut().energy = 20;
    world.entities[0].base.life_mut().stamina = 15;
    world.entities[0].base.life_mut().vision_range = 6;
    world.entities[1].pos = Position::new(0, 2); // Carnivore threat within vision

    let emitted = InteractionSystem::emit_local_alarm(
        &mut world.entities,
        &mut world.environment,
        speaker_id,
        10,
    );
    assert!(emitted, "Alarm must emit when threat is visible");
    assert_eq!(world.entities[0].base.life().energy, 19, "Must deduct 1 energy");
    assert_eq!(world.entities[0].base.life().stamina, 14, "Must deduct 1 stamina");

    // Verify signal trace was received and perceived by a peer at (1, 0)
    let signals = world.environment.perceive_signals_at(Position::new(1, 0), 10);
    assert!(
        signals.iter().any(|s| s.kind == SignalKind::Sound),
        "Peer must perceive acoustic alarm signal"
    );

    // When no threat is visible, emit_local_alarm returns false and deducts nothing
    world.entities.retain(|s| s.id == 1); // Remove threat
    let no_threat_emit = InteractionSystem::emit_local_alarm(
        &mut world.entities,
        &mut world.environment,
        speaker_id,
        11,
    );
    assert!(!no_threat_emit, "Cannot emit alarm when no threat is visible");
    assert_eq!(world.entities[0].base.life().energy, 19, "Energy must not change when alarm fails");
    assert_eq!(world.entities[0].base.life().stamina, 14, "Stamina must not change when alarm fails");
}

#[test]
fn test_5_f2_repro_world_tick_wiring_care_attack_and_alarm() {
    use curious::entity::intent::Intent;

    // 1. Juvenile care guarding in normal world.tick()
    // Mature parent at (1, 0) with stamina 16, full health
    let mut parent = Box::new(HerbivoreEntity::default());
    parent.life_mut().age = 30;
    parent.life_mut().maturity_age = 20;
    parent.life_mut().stamina = 16;
    parent.life_mut().max_stamina = 16;
    parent.life_mut().health = 120;
    parent.life_mut().max_health = 120;
    parent.life_mut().energy = 50;
    parent.life_mut().max_energy = 100;
    let parent_slot = EntitySlot::new(
        1,
        Position::new(1, 0),
        EntityPhase::Active,
        EntityController::AI,
        parent,
    );

    // Juvenile child at (0, 0) with health 40, energy 50
    let mut child = Box::new(HerbivoreEntity::default());
    child.life_mut().age = 5;
    child.life_mut().maturity_age = 20;
    child.life_mut().health = 40;
    child.life_mut().energy = 50;
    let mut child_slot = EntitySlot::new(
        2,
        Position::new(0, 0),
        EntityPhase::Active,
        EntityController::AI,
        child,
    );
    child_slot.parents = vec![1];

    // Carnivore attacker at (0, 1) adjacent to child
    let mut attacker = Box::new(CarnivoreEntity::default());
    attacker.life_mut().energy = 50;
    attacker.life_mut().stamina = 16;
    attacker.life_mut().health = 100;
    let attacker_slot = EntitySlot::new(
        3,
        Position::new(0, 1),
        EntityPhase::Active,
        EntityController::AI,
        attacker,
    );

    let mut world = World::with_seed(-10, 10, -10, 10, vec![parent_slot, child_slot, attacker_slot], 555);
    world.logger = curious::logger::Logger::disabled();
    world.policy.juvenile_care_enabled = true;

    // Issue Attack command on attacker targeting child (id: 2)
    world.push_command(3, Intent::Attack { target_id: 2 });
    world.push_command(1, Intent::Idle { duration: 1 });
    world.push_command(2, Intent::Idle { duration: 1 });

    world.tick();

    let child_after = world.entities.iter().find(|s| s.id == 2).unwrap();
    let parent_after = world.entities.iter().find(|s| s.id == 1).unwrap();

    // Damage is halved by guardian parent from 20 to 10: child health drops 40 -> 30
    assert_eq!(child_after.base.life().health, 30);
    // Guarding parent exerted 4 stamina: 16 -> 12
    assert_eq!(parent_after.base.life().stamina, 12);

    // 2. Parental feeding care in normal world.tick()
    // Set parent energy to 85 (> 75% of 100), child energy to 20 (< 50% of 100)
    world.entities.retain(|s| s.id == 1 || s.id == 2);
    for slot in world.entities.iter_mut() {
        if slot.id == 1 {
            slot.base.life_mut().energy = 85;
        } else if slot.id == 2 {
            slot.base.life_mut().energy = 20;
        }
    }

    world.push_command(1, Intent::Idle { duration: 1 });
    world.push_command(2, Intent::Idle { duration: 1 });
    world.tick();

    let p_after_care = world.entities.iter().find(|s| s.id == 1).unwrap();
    let c_after_care = world.entities.iter().find(|s| s.id == 2).unwrap();

    // Parent transferred 10 energy to child. Each paid 1 BMR energy.
    // Parent: 85 - 10 - 1 = 74
    // Child: 20 + 10 - 1 = 29
    assert_eq!(p_after_care.base.life().energy, 74);
    assert_eq!(c_after_care.base.life().energy, 29);

    // 3. Local alarm coordination in normal world.tick()
    let mut coord_world = World::with_seed(-10, 10, -10, 10, Vec::new(), 777);
    coord_world.logger = curious::logger::Logger::disabled();
    coord_world.policy.local_coordination_enabled = true;

    let mut speaker_herb = Box::new(HerbivoreEntity::default());
    speaker_herb.life_mut().energy = 30;
    speaker_herb.life_mut().stamina = 20;
    speaker_herb.life_mut().vision_range = 6;
    let speaker_slot = EntitySlot::new(
        100,
        Position::new(0, 0),
        EntityPhase::Active,
        EntityController::AI,
        speaker_herb,
    );

    let mut threat_carn = Box::new(CarnivoreEntity::default());
    threat_carn.life_mut().energy = 50;
    let threat_slot = EntitySlot::new(
        200,
        Position::new(0, 2), // distance 2 <= vision_range 6
        EntityPhase::Active,
        EntityController::AI,
        threat_carn,
    );

    coord_world.entities.push(speaker_slot);
    coord_world.entities.push(threat_slot);

    // Run world.tick() with local coordination enabled
    coord_world.tick();

    // Verify acoustic alarm signal was emitted into environment
    let signals = coord_world.environment.perceive_signals_at(Position::new(1, 0), coord_world.tick_counter);
    assert!(
        signals.iter().any(|s| s.kind == SignalKind::Sound),
        "Local alarm signal must be emitted in normal world.tick() when threat is visible"
    );
}

#[test]
fn test_6_f2_reachable_ambush_cover_carnivore_wall_in_world_tick() {
    use curious::entity::intent::Intent;

    // Ordinary Carnivore (can_burrow() == false) on Cell::Empty adjacent to Cell::Wall
    let mut carnivore = Box::new(CarnivoreEntity::default());
    carnivore.life_mut().stamina = 16;
    carnivore.life_mut().max_stamina = 16;
    carnivore.life_mut().energy = 50;
    assert!(!carnivore.can_burrow(), "Carnivores cannot burrow");

    let carn_pos = Position::new(0, 5);
    let predator_slot = EntitySlot::new(
        2,
        carn_pos,
        EntityPhase::Active,
        EntityController::AI,
        carnivore,
    );

    let mut observer_herb = Box::new(HerbivoreEntity::default());
    observer_herb.life_mut().vision_range = 5;
    let observer_slot = EntitySlot::new(
        1,
        Position::new(0, 0),
        EntityPhase::Active,
        EntityController::AI,
        observer_herb,
    );

    let mut world = World::with_seed(-10, 10, -10, 10, vec![observer_slot, predator_slot], 9999);
    world.logger = curious::logger::Logger::disabled();
    world.policy.ambush_predation_enabled = true;

    // Carnivore is at (0, 5), standing on Cell::Empty
    world.map.set_cell(carn_pos, Cell::Empty);
    // Wall is adjacent to Carnivore at (0, 6)
    world.map.set_cell(Position::new(0, 6), Cell::Wall);

    assert!(
        curious::systems::perception::is_in_ambush_cover(&world.map, carn_pos),
        "Standing on Cell::Empty next to Cell::Wall must count as ambush cover"
    );

    // 1. Perception: carnivore is in cover with full stamina -> detectability reduced by 1
    // Distance = 5 > max_detect (4), so observer does NOT spot the carnivore
    let view = PerceptionSystem::build(&world, &world.entities[0]);
    assert!(
        !view.entities.iter().any(|e| e.id == 2),
        "Carnivore in wall cover with full stamina must have stealth in PerceptionSystem"
    );

    // 2. Ambush strike bonus in world.tick():
    // Move carnivore to (0, 1) on Cell::Empty, adjacent to target at (0, 0)
    // Put Wall adjacent to carnivore at (1, 1)
    let p_slot = world.entities.iter_mut().find(|s| s.id == 2).unwrap();
    p_slot.pos = Position::new(0, 1);
    p_slot.base.life_mut().stamina = 16;
    world.map.set_cell(Position::new(0, 1), Cell::Empty);
    world.map.set_cell(Position::new(1, 1), Cell::Wall);

    world.entities.iter_mut().find(|s| s.id == 1).unwrap().base.life_mut().health = 120;
    world.entities.iter_mut().find(|s| s.id == 1).unwrap().base.life_mut().max_health = 120;

    // Carnivore attacks observer in normal world.tick()
    world.push_command(2, Intent::Attack { target_id: 1 });
    world.push_command(1, Intent::Idle { duration: 1 });
    world.tick();

    let victim_after = world.entities.iter().find(|s| s.id == 1).unwrap();
    // Base damage is 20 + 5 ambush bonus = 25 damage: health drops 120 -> 95
    assert_eq!(
        victim_after.base.life().health,
        95,
        "Ambush strike from cover next to Wall must deal 25 damage (20 + 5 bonus)"
    );
}
