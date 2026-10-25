use curious::{
    creatures::{carnivore::CarnivoreEntity, herbivore::HerbivoreEntity},
    entity::{
        Entity,
        controller::EntityController,
        intent::Intent,
        phase::EntityPhase,
    },
    map::movement::{Direction, Position, Steps},
    rules::biology::{self, BMR_ENERGY_COST, BMR_WATER_COST, PASSIVE_HEAL_AMOUNT, PASSIVE_HEAL_ENERGY_COST},
    simulation::checkpoint::State,
    world::{EntitySlot, World},
};

fn make_slot(id: usize, x: isize, y: isize, body: Box<dyn curious::entity::Entity>) -> EntitySlot {
    EntitySlot::new(
        id,
        Position::new(x, y),
        EntityPhase::Active,
        EntityController::AI,
        body,
    )
}

fn make_player_slot(id: usize, x: isize, y: isize, body: Box<dyn curious::entity::Entity>) -> EntitySlot {
    EntitySlot::new(
        id,
        Position::new(x, y),
        EntityPhase::Active,
        EntityController::Player(0),
        body,
    )
}

#[test]
fn test_1_disabled_path_exact_compatibility() {
    // Proves that when world.policy.injury_capacity_penalty is false (default),
    // State and RNG evolution across ticks are 100% byte-for-byte identical.
    let seed = 424242;

    let build_world = |injury_penalty: bool| -> World {
        let mut h1 = Box::new(HerbivoreEntity::default());
        h1.life_mut().health = 40; // Severely injured (< 120 / 2 = 60)

        let mut c1 = Box::new(CarnivoreEntity::default());
        c1.life_mut().health = 50; // Severely injured (< 140 / 2 = 70)

        let mut h2 = Box::new(HerbivoreEntity::default());
        h2.life_mut().health = 120; // Intact

        let slots = vec![
            make_slot(1, 0, 0, h1),
            make_slot(2, 5, 5, c1),
            make_slot(3, -5, -5, h2),
        ];

        let mut world = World::with_seed(-10, 10, -10, 10, slots, seed);
        world.logger = curious::logger::Logger::disabled();
        world.policy.injury_capacity_penalty = injury_penalty;
        world
    };

    let mut w_default = build_world(false);
    assert!(!w_default.policy.injury_capacity_penalty);

    let mut w_explicit_false = build_world(false);

    // Also verify that disabled path lets severely injured entity move at normal speed
    w_default.push_command(
        1,
        Intent::Move {
            steps: Steps(vec![Direction::Up, Direction::Up, Direction::Up]),
        },
    );
    w_explicit_false.push_command(
        1,
        Intent::Move {
            steps: Steps(vec![Direction::Up, Direction::Up, Direction::Up]),
        },
    );

    // Tick 0 executes the move command
    let ev1 = w_default.tick();
    let ev2 = w_explicit_false.tick();
    assert_eq!(ev1.len(), ev2.len());
    // In disabled path, entity 1 moved full 3 steps despite injury
    assert_eq!(w_default.entities[0].pos, Position::new(0, -3));
    assert_eq!(w_explicit_false.entities[0].pos, Position::new(0, -3));

    for _ in 1..10 {
        let ev1 = w_default.tick();
        let ev2 = w_explicit_false.tick();
        assert_eq!(ev1.len(), ev2.len());

        let s1 = State::capture(&w_default);
        let s2 = State::capture(&w_explicit_false);

        let json1 = serde_json::to_string(&s1).unwrap();
        let json2 = serde_json::to_string(&s2).unwrap();
        assert_eq!(json1, json2, "State must be byte-for-byte identical when disabled");
        assert_eq!(w_default.rng.state(), w_explicit_false.rng.state(), "RNG stream must match");
    }
}

#[test]
fn test_2_enabled_injury_capacity_loss_speed_halving_and_double_stamina() {
    // When injury_capacity_penalty is true:
    // Severely injured entity (health <= max_health / 2) has speed halved ((speed / 2).max(1))
    // and pays 2 stamina per step.
    let mut h_intact = Box::new(HerbivoreEntity::default());
    h_intact.life_mut().health = 120; // Intact (> 60), speed = 3

    let mut h_injured = Box::new(HerbivoreEntity::default());
    h_injured.life_mut().health = 50; // Severely injured (<= 60, stays injured after passive heal), speed = 3 -> effective speed = 1

    let mut c_injured = Box::new(CarnivoreEntity::default());
    c_injured.life_mut().health = 50; // Severely injured (<= 70, stays injured after passive heal), speed = 4 -> effective speed = 2

    let slots = vec![
        make_player_slot(1, 0, 0, h_intact),
        make_player_slot(2, 5, 0, h_injured),
        make_player_slot(3, 10, 0, c_injured),
    ];

    let mut world = World::with_seed(-15, 15, -15, 15, slots, 101);
    world.logger = curious::logger::Logger::disabled();
    world.policy.injury_capacity_penalty = true;

    // Entity 1 (intact herbivore): Request 3 steps
    world.push_command(
        1,
        Intent::Move {
            steps: Steps(vec![Direction::Down, Direction::Down, Direction::Down]),
        },
    );

    // Entity 2 (injured herbivore): Request 3 steps
    world.push_command(
        2,
        Intent::Move {
            steps: Steps(vec![Direction::Down, Direction::Down, Direction::Down]),
        },
    );

    // Entity 3 (injured carnivore): Request 4 steps
    world.push_command(
        3,
        Intent::Move {
            steps: Steps(vec![Direction::Down, Direction::Down, Direction::Down, Direction::Down]),
        },
    );

    let initial_stamina_1 = world.entities[0].base.life().stamina;
    let initial_stamina_2 = world.entities[1].base.life().stamina;
    let initial_stamina_3 = world.entities[2].base.life().stamina;

    world.tick();

    // 1. Intact herbivore moved all 3 steps
    assert_eq!(world.entities[0].pos, Position::new(0, 3));
    // Normal stamina cost: 3 steps * 1 = 3 exertion
    assert_eq!(world.entities[0].base.life().stamina, initial_stamina_1 - 3);

    // 2. Injured herbivore moved only 1 step ((3 / 2).max(1) = 1)
    assert_eq!(world.entities[1].pos, Position::new(5, 1));
    // Double stamina cost: 1 step * 2 = 2 exertion
    assert_eq!(world.entities[1].base.life().stamina, initial_stamina_2 - 2);

    // 3. Injured carnivore moved only 2 steps ((4 / 2).max(1) = 2)
    assert_eq!(world.entities[2].pos, Position::new(10, 2));
    // Double stamina cost: 2 steps * 2 = 4 exertion
    assert_eq!(world.entities[2].base.life().stamina, initial_stamina_3 - 4);

    // Test 2b: Insufficient stamina for double cost prevents movement
    // Injured herbivore needs 2 stamina for 1 step. Set max_stamina = 1 so stamina is capped at 1.
    world.entities[1].base.life_mut().max_stamina = 1;
    world.entities[1].base.life_mut().stamina = 1;
    world.push_command(
        2,
        Intent::Move {
            steps: Steps(vec![Direction::Down]),
        },
    );
    let events = world.tick();
    // Position should NOT change because stamina (1) < needed (2)
    assert_eq!(world.entities[1].pos, Position::new(5, 1));
    assert!(events.iter().any(|e| matches!(
        e,
        curious::simulation::SimulationEvent::ActionRejected {
            id: 2,
            ..
        }
    )));

    // Also test direct resolution rejection: locomotion::apply_with_penalty emits Fatigued
    let mut occupied = std::collections::HashMap::new();
    let mut direct_events = Vec::new();
    world.entities[1].base.life_mut().stamina = 1;
    let path = vec![Position::new(5, 2)];
    let res = curious::systems::locomotion::apply_with_penalty(
        &mut world.entities,
        2,
        &path,
        &world.map,
        &mut occupied,
        &mut direct_events,
        true,
    );
    assert_eq!(res, Some(Position::new(5, 1))); // didn't move
    assert!(direct_events.iter().any(|e| matches!(
        e,
        curious::simulation::SimulationEvent::ActionRejected {
            id: 2,
            reason: curious::simulation::event::RejectionReason::Fatigued,
            ..
        }
    )));
}

#[test]
fn test_3_recovery_via_passive_healing_restores_full_speed() {
    let mut h = Box::new(HerbivoreEntity::default());
    // Max health is 120. Set health to 60 (exactly on threshold 120/2 = 60).
    // High energy and water to enable passive healing.
    h.life_mut().health = 60;
    h.life_mut().max_energy = 100;
    h.life_mut().energy = 90; // > 100 * 3 / 4 = 75
    h.life_mut().max_water = 80;
    h.life_mut().water = 80;
    h.life_mut().stamina = 16;

    let slots = vec![make_player_slot(1, 0, 0, h)];
    let mut world = World::with_seed(-10, 10, -10, 10, slots, 77);
    world.logger = curious::logger::Logger::disabled();
    world.policy.injury_capacity_penalty = true;

    // Prior to tick: entity is severely injured
    assert!(world.entities[0].base.life().is_severely_injured());
    assert_eq!(world.entities[0].base.life().effective_speed(true), 1);

    // Tick 1: Metabolism updates. Passive healing runs:
    // Health increases from 60 to 61 (PASSIVE_HEAL_AMOUNT = 1).
    // Energy decreases by PASSIVE_HEAL_ENERGY_COST (2) + BMR (1) = 3.
    world.tick();

    let life = world.entities[0].base.life();
    assert_eq!(life.health, 60 + PASSIVE_HEAL_AMOUNT);
    assert_eq!(life.energy, 90 - PASSIVE_HEAL_ENERGY_COST - BMR_ENERGY_COST);

    // Now health (61) > max_health / 2 (60) => no longer severely injured!
    assert!(!life.is_severely_injured());
    assert_eq!(life.effective_speed(true), 3, "Full speed 3 must be automatically restored");

    // Push 3 steps move command
    world.push_command(
        1,
        Intent::Move {
            steps: Steps(vec![Direction::Right, Direction::Right, Direction::Right]),
        },
    );
    world.tick();

    // Entity moves full 3 steps
    assert_eq!(world.entities[0].pos, Position::new(3, 0));
}

#[test]
fn test_4_full_accounting_across_intact_injured_healed_and_dead_states() {
    let mut h = Box::new(HerbivoreEntity::default());
    h.life_mut().health = 120;
    h.life_mut().max_energy = 100;
    h.life_mut().energy = 90;
    h.life_mut().max_water = 80;
    h.life_mut().water = 80;
    h.life_mut().max_stamina = 16;
    h.life_mut().stamina = 16;

    let slots = vec![make_player_slot(1, 0, 0, h)];
    let mut world = World::with_seed(-10, 10, -10, 10, slots, 999);
    world.logger = curious::logger::Logger::disabled();
    world.policy.injury_capacity_penalty = true;

    // --- State 1: Intact ---
    assert_eq!(world.entities[0].base.life().health, 120);
    assert!(!world.entities[0].base.life().is_severely_injured());
    assert_eq!(world.entities[0].base.life().effective_speed(true), 3);

    // --- Transition to Severely Injured via combat damage ---
    // Apply 70 damage: 120 - 70 = 50 (<= 60)
    world.entities[0].base.life_mut().take_damage(70);
    assert_eq!(world.entities[0].base.life().health, 50);
    assert!(world.entities[0].base.life().is_severely_injured());
    assert_eq!(world.entities[0].base.life().effective_speed(true), 1);

    // --- Move while severely injured ---
    let energy_before_move = world.entities[0].base.life().energy;
    world.push_command(
        1,
        Intent::Move {
            steps: Steps(vec![Direction::Up, Direction::Up]), // attempts 2, capped at 1
        },
    );
    world.tick();

    // Moved 1 step to (0, -1)
    assert_eq!(world.entities[0].pos, Position::new(0, -1));

    // Energy accounting:
    // 1 movement step (cost 1) + 1 BMR + passive heal (2) = 4 total energy consumed
    // (because energy 90 was still > 75)
    let expected_energy = energy_before_move - 1 - biology::BMR_ENERGY_COST - PASSIVE_HEAL_ENERGY_COST;
    assert_eq!(world.entities[0].base.life().energy, expected_energy);

    // Health accounting:
    // Started at 50, healed +1 by passive heal = 51
    assert_eq!(world.entities[0].base.life().health, 51);

    // Water accounting:
    // 80 - BMR_WATER_COST (1) = 79
    assert_eq!(world.entities[0].base.life().water, 80 - BMR_WATER_COST);

    // --- Transition to Dead ---
    // Apply fatal damage
    world.entities[0].base.life_mut().damage_from(
        100,
        curious::simulation::event::DeathCause::Combat,
    );
    assert_eq!(world.entities[0].base.life().health, 0);
    assert!(!world.entities[0].base.life().is_alive());
    assert_eq!(
        world.entities[0].base.life().death_cause,
        Some(curious::simulation::event::DeathCause::Combat)
    );

    // Dead entity cannot move or passively heal on subsequent ticks
    let dead_pos = world.entities[0].pos;
    let dead_energy = world.entities[0].base.life().energy;
    world.push_command(
        1,
        Intent::Move {
            steps: Steps(vec![Direction::Down]),
        },
    );
    world.tick();

    // Position and energy remain frozen
    assert_eq!(world.entities[0].pos, dead_pos);
    assert_eq!(world.entities[0].base.life().energy, dead_energy);
    assert_eq!(world.entities[0].base.life().health, 0);
}
