use curious::{
    creatures::{
        carnivore::CarnivoreEntity, herbivore::HerbivoreEntity, omnivore::OmnivoreEntity,
    },
    entity::{
        controller::EntityController,
        intent::Intent,
        phase::{CorpseStage, EntityPhase},
        species::Species,
        Entity,
    },
    environment::{Environment, ResourceKind, ResourceSite, Season, SeasonCycle},
    map::{cell::Cell, movement::Position, Map},
    persistence::State,
    systems::{life_cycle::LifeCycleSystem, metabolism::MetabolismSystem},
    world::{EntitySlot, World},
};

#[test]
fn test_disabled_path_exact_state_and_rng_parity() {
    let make_world = |seed: u64| {
        let entities = vec![
            EntitySlot::new(
                1,
                Position::new(0, 0),
                EntityPhase::Active,
                EntityController::AI,
                Box::new(HerbivoreEntity::default()),
            ),
            EntitySlot::new(
                2,
                Position::new(2, 2),
                EntityPhase::Active,
                EntityController::AI,
                Box::new(HerbivoreEntity::default()),
            ),
        ];
        let mut world = World::with_seed(-10, 10, -10, 10, entities, seed);
        world.logger = curious::logger::Logger::disabled();
        world
    };

    let mut world_a = make_world(42);
    let mut world_b = make_world(42);

    // Verify default environment has no season cycle and no burrows
    assert!(world_a.environment.season_cycle.is_none());
    assert!(world_a.environment.burrows.is_empty());
    assert!(world_b.environment.season_cycle.is_none());
    assert!(world_b.environment.burrows.is_empty());

    // Run for 60 ticks
    for _ in 0..60 {
        world_a.tick();
        world_b.tick();
    }

    // Exact state and RNG parity across both runs
    assert_eq!(world_a.rng.state(), world_b.rng.state());
    let state_a = State::capture(&world_a);
    let state_b = State::capture(&world_b);
    assert_eq!(
        serde_json::to_value(&state_a).unwrap(),
        serde_json::to_value(&state_b).unwrap()
    );
}

#[test]
fn test_deterministic_checkpoint_serialization_and_resume() {
    let make_world = |seed: u64| {
        let entities = vec![EntitySlot::new(
            1,
            Position::new(0, 0),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        )];
        let mut world = World::with_seed(-10, 10, -10, 10, entities, seed);
        world.logger = curious::logger::Logger::disabled();

        // Enable seasonal climate cycle and burrows
        world.environment.season_cycle = Some(SeasonCycle {
            period: 40,
            winter_bmr_surcharge: 3,
            winter_plant_regen_divisor: 2,
        });
        assert!(world.environment.dig_burrow(Position::new(0, 0), 0, 100));
        assert!(world.environment.dig_burrow(Position::new(4, 4), 0, 30));
        world
    };

    // Uninterrupted 50-tick run
    let mut world_uninterrupted = make_world(777);
    for _ in 0..50 {
        world_uninterrupted.tick();
    }

    // Interrupted run: run 25 ticks, checkpoint/serialize/restore, run remaining 25 ticks
    let mut world_interrupted = make_world(777);
    for _ in 0..25 {
        world_interrupted.tick();
    }

    let state_mid = State::capture(&world_interrupted);
    let json_bytes = serde_json::to_string(&state_mid).unwrap();

    let restored_state: State = serde_json::from_str(&json_bytes).unwrap();
    let mut world_restored = restored_state.restore().unwrap();
    world_restored.logger = curious::logger::Logger::disabled();

    for _ in 0..25 {
        world_restored.tick();
    }

    // Verify exact match of State and RNG
    assert_eq!(world_uninterrupted.rng.state(), world_restored.rng.state());
    let state_final_a = State::capture(&world_uninterrupted);
    let state_final_b = State::capture(&world_restored);
    assert_eq!(
        serde_json::to_value(&state_final_a).unwrap(),
        serde_json::to_value(&state_final_b).unwrap()
    );
}

#[test]
fn test_seasonal_winter_plant_regen_thermal_cost_and_burrow_shelter() {
    let mut env = Environment::default();
    let season_cycle = SeasonCycle {
        period: 40,
        winter_bmr_surcharge: 2,
        winter_plant_regen_divisor: 2,
    };
    env.season_cycle = Some(season_cycle);

    // 1. Season verification: period 40 => 0..10 Spring, 10..20 Summer, 20..30 Autumn, 30..40 Winter
    assert_eq!(season_cycle.current_season(0), Season::Spring);
    assert_eq!(season_cycle.current_season(9), Season::Spring);
    assert_eq!(season_cycle.current_season(10), Season::Summer);
    assert_eq!(season_cycle.current_season(20), Season::Autumn);
    assert_eq!(season_cycle.current_season(30), Season::Winter);
    assert_eq!(season_cycle.current_season(39), Season::Winter);

    // 2. Winter plant replenishment scaling
    let food_pos = Position::new(1, 1);
    let water_pos = Position::new(2, 2);
    env.set_site(ResourceSite {
        pos: food_pos,
        kind: ResourceKind::Food,
        capacity: 20,
        replenish: 4,
        interval: 2,
        schedule: None,
    });
    env.set_site(ResourceSite {
        pos: water_pos,
        kind: ResourceKind::Water,
        capacity: 20,
        replenish: 4,
        interval: 2,
        schedule: None,
    });

    let mut map = Map::new(-5, 5, -5, 5);
    let mut events = Vec::new();

    // In Spring (tick 2): food and water replenish normal amount (4)
    env.update(&mut map, 2, &mut events);
    assert_eq!(map.cell(food_pos), Some(&Cell::Food { amount: 4 }));
    assert_eq!(map.cell(water_pos), Some(&Cell::Water { amount: 4 }));

    // Reset cells to Empty
    map.set_cell(food_pos, Cell::Empty);
    map.set_cell(water_pos, Cell::Empty);

    // In Winter (tick 32): food replenishment is scaled by divisor 2 (4 / 2 = 2); water unchanged (4)
    env.update(&mut map, 32, &mut events);
    assert_eq!(map.cell(food_pos), Some(&Cell::Food { amount: 2 }));
    assert_eq!(map.cell(water_pos), Some(&Cell::Water { amount: 4 }));

    // Minimum plant replenishment floor test: site with replenish 1 scales down to min 1
    let small_food_pos = Position::new(3, 3);
    env.set_site(ResourceSite {
        pos: small_food_pos,
        kind: ResourceKind::Food,
        capacity: 10,
        replenish: 1,
        interval: 2,
        schedule: None,
    });
    map.set_cell(small_food_pos, Cell::Empty);
    env.update(&mut map, 34, &mut events);
    assert_eq!(map.cell(small_food_pos), Some(&Cell::Food { amount: 1 }));

    // 3. Burrow creation, refresh, and TTL expiration
    let burrow_pos = Position::new(0, 0);
    assert!(env.dig_burrow(burrow_pos, 10, 15));
    // Active at tick 20 (age 10 < 15)
    assert!(env.has_active_burrow(burrow_pos, 20));
    // Expired at tick 25 (age 15 >= 15)
    assert!(!env.has_active_burrow(burrow_pos, 25));

    // Pruned from burrows list on update
    env.update(&mut map, 25, &mut events);
    assert!(!env.has_active_burrow(burrow_pos, 25));
    assert!(env.burrows.is_empty());

    // 4. Burrow shelter protects against winter BMR surcharge
    // Dig burrow at (0, 0) valid from tick 30 to tick 50
    assert!(env.dig_burrow(burrow_pos, 30, 20));

    // At tick 35 (Winter):
    // Position (0, 0) is sheltered by active burrow -> surcharge = 0
    assert_eq!(env.winter_bmr_surcharge(burrow_pos, 35), 0);
    // Position (5, 5) has no burrow -> surcharge = 2
    assert_eq!(env.winter_bmr_surcharge(Position::new(5, 5), 35), 2);
    // In Spring (tick 5): no surcharge regardless of burrow
    assert_eq!(env.winter_bmr_surcharge(Position::new(5, 5), 5), 0);

    // 5. Metabolism apply_seasonal_thermal_cost helper
    let mut slot_unsheltered = EntitySlot::new(
        10,
        Position::new(5, 5),
        EntityPhase::Active,
        EntityController::AI,
        Box::new(HerbivoreEntity::default()),
    );
    slot_unsheltered.base.life_mut().energy = 50;
    MetabolismSystem::apply_seasonal_thermal_cost(&mut slot_unsheltered, &env, 35);
    assert_eq!(slot_unsheltered.base.life().energy, 48); // 50 - 2

    let mut slot_sheltered = EntitySlot::new(
        11,
        burrow_pos,
        EntityPhase::Active,
        EntityController::AI,
        Box::new(HerbivoreEntity::default()),
    );
    slot_sheltered.base.life_mut().energy = 50;
    MetabolismSystem::apply_seasonal_thermal_cost(&mut slot_sheltered, &env, 35);
    assert_eq!(slot_sheltered.base.life().energy, 50); // Protected by burrow!
}

#[test]
fn test_corpse_stage_progression_and_scavenging_toxicity() {
    let mut corpse_slot = EntitySlot::new(
        99,
        Position::new(1, 1),
        EntityPhase::Corpse { remaining: 5 },
        EntityController::AI,
        Box::new(HerbivoreEntity::default()),
    );
    corpse_slot.corpse_energy = 10;

    // 1. Fresh stage: remaining >= 4 (5 and 4 are Fresh)
    assert_eq!(corpse_slot.phase.corpse_stage(), Some(CorpseStage::Fresh));
    assert_eq!(LifeCycleSystem::corpse_stage(&corpse_slot), Some(CorpseStage::Fresh));

    // Tick once -> remaining = 4: still Fresh
    corpse_slot.phase.tick();
    assert_eq!(corpse_slot.phase, EntityPhase::Corpse { remaining: 4 });
    assert_eq!(corpse_slot.phase.corpse_stage(), Some(CorpseStage::Fresh));

    // Tick once -> remaining = 3: Aging stage (remaining == 3 or 2)
    corpse_slot.phase.tick();
    assert_eq!(corpse_slot.phase, EntityPhase::Corpse { remaining: 3 });
    assert_eq!(corpse_slot.phase.corpse_stage(), Some(CorpseStage::Aging));

    // Tick once -> remaining = 2: still Aging
    corpse_slot.phase.tick();
    assert_eq!(corpse_slot.phase, EntityPhase::Corpse { remaining: 2 });
    assert_eq!(corpse_slot.phase.corpse_stage(), Some(CorpseStage::Aging));

    // Tick once -> remaining = 1: Putrid stage
    corpse_slot.phase.tick();
    assert_eq!(corpse_slot.phase, EntityPhase::Corpse { remaining: 1 });
    assert_eq!(corpse_slot.phase.corpse_stage(), Some(CorpseStage::Putrid));

    // Tick once -> remaining = 0: still Putrid
    corpse_slot.phase.tick();
    assert_eq!(corpse_slot.phase, EntityPhase::Corpse { remaining: 0 });
    assert_eq!(corpse_slot.phase.corpse_stage(), Some(CorpseStage::Putrid));

    // Next tick transitions from 0 to Removed
    corpse_slot.phase.tick();
    assert_eq!(corpse_slot.phase, EntityPhase::Removed);
    assert_eq!(corpse_slot.phase.corpse_stage(), None);

    // 2. Active and Sleeping phases have None corpse stage
    let active_slot = EntitySlot::new(
        1,
        Position::new(0, 0),
        EntityPhase::Active,
        EntityController::AI,
        Box::new(HerbivoreEntity::default()),
    );
    assert_eq!(active_slot.phase.corpse_stage(), None);

    let sleeping_slot = EntitySlot::new(
        2,
        Position::new(0, 0),
        EntityPhase::Sleeping { remaining: 5 },
        EntityController::AI,
        Box::new(HerbivoreEntity::default()),
    );
    assert_eq!(sleeping_slot.phase.corpse_stage(), None);

    // 3. Scavenging toxicity differentiation
    assert_eq!(CorpseStage::Fresh.scavenging_toxicity(Species::Omnivore), 0);
    assert_eq!(CorpseStage::Aging.scavenging_toxicity(Species::Omnivore), 0);
    assert_eq!(CorpseStage::Putrid.scavenging_toxicity(Species::Omnivore), 0);

    assert_eq!(CorpseStage::Fresh.scavenging_toxicity(Species::Carnivore), 0);
    assert_eq!(CorpseStage::Aging.scavenging_toxicity(Species::Carnivore), 0);
    assert_eq!(CorpseStage::Putrid.scavenging_toxicity(Species::Carnivore), 4);

    assert_eq!(CorpseStage::Putrid.scavenging_toxicity(Species::Herbivore), 4);
}

#[test]
fn test_f4_repro_winter_bmr_surcharge_in_world_tick() {
    // 4-period season: tick 1 (Summer), tick 2 (Autumn), tick 3 (Winter)
    let cycle = SeasonCycle {
        period: 4,
        winter_bmr_surcharge: 17,
        winter_plant_regen_divisor: 2,
    };

    let mut unsheltered_herb = Box::new(HerbivoreEntity::default());
    unsheltered_herb.life_mut().energy = 80;
    unsheltered_herb.life_mut().max_energy = 100;

    let mut sheltered_herb = Box::new(HerbivoreEntity::default());
    sheltered_herb.life_mut().energy = 80;
    sheltered_herb.life_mut().max_energy = 100;

    let slots = vec![
        EntitySlot::new(
            1,
            Position::new(0, 0),
            EntityPhase::Active,
            EntityController::AI,
            sheltered_herb,
        ),
        EntitySlot::new(
            2,
            Position::new(5, 5),
            EntityPhase::Active,
            EntityController::AI,
            unsheltered_herb,
        ),
    ];

    let mut world = World::with_seed(-10, 10, -10, 10, slots, 123);
    world.logger = curious::logger::Logger::disabled();
    world.environment.season_cycle = Some(cycle);

    // Place burrow at (0, 0) sheltered position
    assert!(world.dig_burrow(Position::new(0, 0), 100));

    let mut unsheltered_energies = Vec::new();
    let mut sheltered_energies = Vec::new();

    // Run 3 ticks
    for _ in 1..=3 {
        // Force Idle intent so entities only pay basal metabolic costs
        world.push_command(1, Intent::Idle { duration: 1 });
        world.push_command(2, Intent::Idle { duration: 1 });
        world.tick();

        let s1 = world.entities.iter().find(|s| s.id == 1).unwrap();
        let s2 = world.entities.iter().find(|s| s.id == 2).unwrap();
        sheltered_energies.push(s1.base.life().energy);
        unsheltered_energies.push(s2.base.life().energy);
    }

    // Exact audit repro:
    // Tick 1: 80 - 1 = 79
    // Tick 2: 79 - 1 = 78
    // Tick 3: 78 - 1 - 17 (winter surcharge) = 60 for unsheltered, vs 78 - 1 = 77 for sheltered
    assert_eq!(unsheltered_energies, vec![79, 78, 60]);
    assert_eq!(sheltered_energies, vec![79, 78, 77]);
}

#[test]
fn test_f5_repro_dig_burrow_sync_and_ttl_expiration_in_world_tick() {
    let mut world = World::with_seed(-10, 10, -10, 10, Vec::new(), 42);
    world.logger = curious::logger::Logger::disabled();

    // Strict validation rejection conditions:
    // 1. ttl == 0 must return false and leave burrows empty
    assert!(!world.dig_burrow(Position::new(1, 1), 0));
    assert!(world.environment.burrows.is_empty());
    assert!(!world.environment.has_active_burrow(Position::new(1, 1), 0));

    // 2. Out-of-bounds position must return false and leave burrows empty
    assert!(!world.dig_burrow(Position::new(999, 999), 10));
    assert!(world.environment.burrows.is_empty());

    // 3. Non-Empty cell types must return false and leave burrows empty
    world.map.set_cell(Position::new(2, 2), Cell::Wall);
    assert!(!world.dig_burrow(Position::new(2, 2), 10));
    world.map.set_cell(Position::new(2, 3), Cell::Food { amount: 5 });
    assert!(!world.dig_burrow(Position::new(2, 3), 10));
    world.map.set_cell(Position::new(2, 4), Cell::Water { amount: 5 });
    assert!(!world.dig_burrow(Position::new(2, 4), 10));
    assert!(world.environment.burrows.is_empty());

    let burrow_pos = Position::new(3, 3);
    assert_eq!(world.map.cell(burrow_pos), Some(&Cell::Empty));

    // Dig burrow at tick 0 with TTL = 2
    assert!(world.dig_burrow(burrow_pos, 2));
    assert_eq!(world.map.cell(burrow_pos), Some(&Cell::Burrow));

    // Tick 1: age = 1 - 0 = 1 < 2 -> still active Burrow
    world.tick();
    assert_eq!(world.map.cell(burrow_pos), Some(&Cell::Burrow));
    assert!(world.environment.has_active_burrow(burrow_pos, world.tick_counter));

    // Tick 2: age = 2 - 0 = 2 >= 2 -> expired, reverted to Cell::Empty
    world.tick();
    assert_eq!(world.map.cell(burrow_pos), Some(&Cell::Empty));
    assert!(!world.environment.has_active_burrow(burrow_pos, world.tick_counter));
}

#[test]
fn test_f5_herbivore_winter_automatic_burrow_excavation() {
    let mut herbivore = Box::new(HerbivoreEntity::default());
    herbivore.life_mut().energy = 70; // > max_energy (80) / 2 = 40
    herbivore.life_mut().stamina = 16; // >= 4
    let herb_pos = Position::new(4, 4);
    let herb_slot = EntitySlot::new(
        1,
        herb_pos,
        EntityPhase::Active,
        EntityController::AI,
        herbivore,
    );

    let mut world = World::with_seed(-10, 10, -10, 10, vec![herb_slot], 1234);
    world.logger = curious::logger::Logger::disabled();
    // 4-period cycle: tick 3 is Winter
    world.environment.season_cycle = Some(SeasonCycle {
        period: 4,
        winter_bmr_surcharge: 5,
        winter_plant_regen_divisor: 2,
    });
    world.map.set_cell(herb_pos, Cell::Empty);
    // Fast forward to tick 2, so next tick() is tick 3 (Winter)
    world.tick_counter = 2;

    world.tick();

    assert_eq!(world.tick_counter, 3);
    assert_eq!(world.environment.season_cycle.unwrap().current_season(3), Season::Winter);
    assert_eq!(world.map.cell(herb_pos), Some(&Cell::Burrow));
    assert!(world.environment.has_active_burrow(herb_pos, world.tick_counter));
    let herb_after = world.entities.iter().find(|s| s.id == 1).unwrap();
    // Herbivore exerted stamina to excavate shelter
    assert!(herb_after.base.life().stamina <= 15);
}

#[test]
fn test_f6_repro_corpse_progression_and_toxicity_in_world_tick() {
    // 1. Verify corpse progression in normal world ticks
    let mut dying_herb = Box::new(HerbivoreEntity::default());
    dying_herb.life_mut().health = 0; // will die on finalize_tick
    let victim_slot = EntitySlot::new(
        10,
        Position::new(0, 0),
        EntityPhase::Active,
        EntityController::AI,
        dying_herb,
    );

    let mut world = World::with_seed(-10, 10, -10, 10, vec![victim_slot], 99);
    world.logger = curious::logger::Logger::disabled();

    // Tick 1: entity dies on finalize_tick -> Corpse { remaining: 5 } (Fresh)
    world.tick();
    let slot = world.entities.iter().find(|s| s.id == 10).unwrap();
    assert_eq!(slot.phase, EntityPhase::Corpse { remaining: 5 });
    assert_eq!(slot.phase.corpse_stage(), Some(CorpseStage::Fresh));
    let initial_energy = slot.corpse_energy;
    assert_eq!(initial_energy, 60);

    // Tick 2: remaining decrements to 4 -> still Fresh; retains full nutrition
    world.tick();
    let slot = world.entities.iter().find(|s| s.id == 10).unwrap();
    assert_eq!(slot.phase, EntityPhase::Corpse { remaining: 4 });
    assert_eq!(slot.phase.corpse_stage(), Some(CorpseStage::Fresh));
    assert_eq!(slot.corpse_energy, 60);

    // Tick 3: remaining decrements to 3 -> Aging; corpse_energy decays by 1 on every aging/putrid tick
    world.tick();
    let slot = world.entities.iter().find(|s| s.id == 10).unwrap();
    assert_eq!(slot.phase, EntityPhase::Corpse { remaining: 3 });
    assert_eq!(slot.phase.corpse_stage(), Some(CorpseStage::Aging));
    assert_eq!(slot.corpse_energy, 59);

    // Tick 4: remaining decrements to 2 -> Aging; corpse_energy decays by 1
    world.tick();
    let slot = world.entities.iter().find(|s| s.id == 10).unwrap();
    assert_eq!(slot.phase, EntityPhase::Corpse { remaining: 2 });
    assert_eq!(slot.phase.corpse_stage(), Some(CorpseStage::Aging));
    assert_eq!(slot.corpse_energy, 58);

    // Tick 5: remaining decrements to 1 -> Putrid; corpse_energy decays by 1
    world.tick();
    let slot = world.entities.iter().find(|s| s.id == 10).unwrap();
    assert_eq!(slot.phase, EntityPhase::Corpse { remaining: 1 });
    assert_eq!(slot.phase.corpse_stage(), Some(CorpseStage::Putrid));
    assert_eq!(slot.corpse_energy, 57);

    // 2. Scavenging toxicity in World::tick:
    // Setup Carnivore and Omnivore eating Putrid corpse at (0, 0)
    let mut putrid_corpse = EntitySlot::new(
        20,
        Position::new(0, 0),
        EntityPhase::Corpse { remaining: 1 },
        EntityController::AI,
        Box::new(HerbivoreEntity::default()),
    );
    putrid_corpse.corpse_energy = 50;

    let mut carnivore = Box::new(CarnivoreEntity::default());
    carnivore.life_mut().health = 100;
    carnivore.life_mut().energy = 10;
    let carnivore_slot = EntitySlot::new(
        21,
        Position::new(1, 0), // adjacent
        EntityPhase::Active,
        EntityController::AI,
        carnivore,
    );

    let mut scavenger_world = World::with_seed(
        -10,
        10,
        -10,
        10,
        vec![putrid_corpse, carnivore_slot],
        1001,
    );
    scavenger_world.logger = curious::logger::Logger::disabled();
    scavenger_world.push_command(
        21,
        Intent::Eat {
            at: curious::map::movement::Steps(vec![curious::map::movement::Direction::Left]),
            corpse_id: Some(20),
        },
    );

    scavenger_world.tick();
    let carnivore_after = scavenger_world.entities.iter().find(|s| s.id == 21).unwrap();
    // Carnivore took 4 toxicity damage from eating Putrid corpse: 100 - 4 = 96
    assert_eq!(carnivore_after.base.life().health, 96);

    // Now test Omnivore eating Putrid corpse: 0 toxicity damage
    let mut putrid_corpse_2 = EntitySlot::new(
        30,
        Position::new(0, 0),
        EntityPhase::Corpse { remaining: 1 },
        EntityController::AI,
        Box::new(HerbivoreEntity::default()),
    );
    putrid_corpse_2.corpse_energy = 50;

    let mut omnivore = Box::new(OmnivoreEntity::default());
    omnivore.life_mut().health = 100;
    omnivore.life_mut().energy = 10;
    let omnivore_slot = EntitySlot::new(
        31,
        Position::new(1, 0),
        EntityPhase::Active,
        EntityController::AI,
        omnivore,
    );

    let mut omnivore_world = World::with_seed(
        -10,
        10,
        -10,
        10,
        vec![putrid_corpse_2, omnivore_slot],
        1002,
    );
    omnivore_world.logger = curious::logger::Logger::disabled();
    omnivore_world.push_command(
        31,
        Intent::Eat {
            at: curious::map::movement::Steps(vec![curious::map::movement::Direction::Left]),
            corpse_id: Some(30),
        },
    );

    omnivore_world.tick();
    let omnivore_after = omnivore_world.entities.iter().find(|s| s.id == 31).unwrap();
    // Omnivore took 0 toxicity damage: health remains 100
    assert_eq!(omnivore_after.base.life().health, 100);

    // 3. Carnivore at full energy (amount == 0) eating Putrid corpse takes 0 toxicity damage
    let mut putrid_corpse_3 = EntitySlot::new(
        40,
        Position::new(0, 0),
        EntityPhase::Corpse { remaining: 1 },
        EntityController::AI,
        Box::new(HerbivoreEntity::default()),
    );
    putrid_corpse_3.corpse_energy = 50;

    let mut full_carnivore = Box::new(CarnivoreEntity::default());
    full_carnivore.life_mut().health = 140;
    full_carnivore.life_mut().max_health = 140;
    full_carnivore.life_mut().max_energy = 90;
    // Energy > max_energy so after 1 BMR cost in begin_tick, energy remains >= max_energy (amount == 0)
    full_carnivore.life_mut().energy = 95;
    let full_carnivore_slot = EntitySlot::new(
        41,
        Position::new(1, 0),
        EntityPhase::Active,
        EntityController::AI,
        full_carnivore,
    );

    let mut full_world = World::with_seed(
        -10,
        10,
        -10,
        10,
        vec![putrid_corpse_3, full_carnivore_slot],
        1003,
    );
    full_world.logger = curious::logger::Logger::disabled();
    full_world.push_command(
        41,
        Intent::Eat {
            at: curious::map::movement::Steps(vec![curious::map::movement::Direction::Left]),
            corpse_id: Some(40),
        },
    );

    full_world.tick();
    let full_after = full_world.entities.iter().find(|s| s.id == 41).unwrap();
    // Because amount was 0, no toxicity damage is applied: health remains 140
    assert_eq!(full_after.base.life().health, 140);
}
