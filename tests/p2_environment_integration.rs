use curious::{
    creatures::herbivore::HerbivoreEntity,
    entity::{
        controller::EntityController,
        phase::{CorpseStage, EntityPhase},
        species::Species,
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
        EntityPhase::Corpse { remaining: 25 },
        EntityController::AI,
        Box::new(HerbivoreEntity::default()),
    );

    // 1. Fresh stage: remaining > 20
    assert_eq!(corpse_slot.phase.corpse_stage(), Some(CorpseStage::Fresh));
    assert_eq!(LifeCycleSystem::corpse_stage(&corpse_slot), Some(CorpseStage::Fresh));

    // Advance 5 ticks -> remaining = 20: Aging stage (8 < remaining <= 20)
    for _ in 0..5 {
        corpse_slot.phase.tick();
    }
    assert_eq!(corpse_slot.phase, EntityPhase::Corpse { remaining: 20 });
    assert_eq!(corpse_slot.phase.corpse_stage(), Some(CorpseStage::Aging));

    // Advance 11 more ticks -> remaining = 9: still Aging
    for _ in 0..11 {
        corpse_slot.phase.tick();
    }
    assert_eq!(corpse_slot.phase, EntityPhase::Corpse { remaining: 9 });
    assert_eq!(corpse_slot.phase.corpse_stage(), Some(CorpseStage::Aging));

    // Advance 1 tick -> remaining = 8: Putrid stage (remaining <= 8)
    corpse_slot.phase.tick();
    assert_eq!(corpse_slot.phase, EntityPhase::Corpse { remaining: 8 });
    assert_eq!(corpse_slot.phase.corpse_stage(), Some(CorpseStage::Putrid));

    // Advance remaining ticks to 0 (8 ticks: remaining becomes 0, still Putrid)
    for _ in 0..8 {
        corpse_slot.phase.tick();
    }
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
    // Omnivore has 0 toxicity at all stages (scavenging resistance)
    assert_eq!(CorpseStage::Fresh.scavenging_toxicity(Species::Omnivore), 0);
    assert_eq!(CorpseStage::Aging.scavenging_toxicity(Species::Omnivore), 0);
    assert_eq!(CorpseStage::Putrid.scavenging_toxicity(Species::Omnivore), 0);

    // Carnivore has 0 toxicity at Fresh and Aging, but 4 health penalty at Putrid
    assert_eq!(CorpseStage::Fresh.scavenging_toxicity(Species::Carnivore), 0);
    assert_eq!(CorpseStage::Aging.scavenging_toxicity(Species::Carnivore), 0);
    assert_eq!(CorpseStage::Putrid.scavenging_toxicity(Species::Carnivore), 4);

    // Herbivore has toxicity penalty
    assert_eq!(CorpseStage::Putrid.scavenging_toxicity(Species::Herbivore), 4);
}
