use curious::{
    creatures::{
        carnivore::CarnivoreEntity, herbivore::HerbivoreEntity, omnivore::OmnivoreEntity,
    },
    entity::{controller::EntityController, phase::EntityPhase},
    environment::{DayNightCycle, Environment, SeasonCycle},
    map::{cell::Cell, movement::Position},
    persistence::State,
    world::{EntitySlot, World},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HabitatTopology {
    OpenPlains,
    CorridorMaze,
    SeasonalScarcity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AblationArm {
    ControlFlagsOff,
    WaterContextOnly,
    RefugeFirstIntervalOnly,
    SpatialCuriosityOnly,
    ClimateAndBurrowsOnly,
    SocialAndNicheOnly,
    AllV19Features,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArmRunMetrics {
    pub known_sectors: usize,
    pub context_entries: usize,
    pub first_interval_outcomes: usize,
    pub newly_excavated_burrows: usize,
    pub unique_signals_emitted: usize,
}

fn build_ablation_entities(seed: u64) -> Vec<EntitySlot> {
    let ox = (seed % 5) as isize - 2;
    let oy = (seed % 3) as isize - 1;

    let mut slots = vec![
        // Entity 1: Herbivore in threat range of Carnivore 3 and adjacent to 2 burrows
        EntitySlot::new(
            1,
            Position::new(-6 + ox, -6 + oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        ),
        // Entity 2: Herbivore roaming in empty space with high energy and stamina for winter burrow digging
        EntitySlot::new(
            2,
            Position::new(6 + ox, -6 + oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        ),
        // Entity 3: Carnivore threatening Herbivore 1
        EntitySlot::new(
            3,
            Position::new(-4 + ox, -6 + oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(CarnivoreEntity::default()),
        ),
        // Entity 4: Carnivore roaming in distant quadrant
        EntitySlot::new(
            4,
            Position::new(6 + ox, 6 + oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(CarnivoreEntity::default()),
        ),
        // Entity 5: Omnivore with low water to immediately trigger water seeking & residual learning
        EntitySlot::new(
            5,
            Position::new(ox, oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(OmnivoreEntity::default()),
        ),
        // Entity 6: Omnivore with moderate water
        EntitySlot::new(
            6,
            Position::new(2 + ox, -2 + oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(OmnivoreEntity::default()),
        ),
    ];

    slots[0].base.life_mut().health = 100;
    slots[0].base.life_mut().energy = 70;
    slots[0].base.life_mut().stamina = 16;

    slots[1].base.life_mut().energy = 95;
    slots[1].base.life_mut().stamina = 30;

    slots[2].base.life_mut().energy = 80;
    slots[2].base.life_mut().stamina = 16;

    slots[4].base.life_mut().water = 15;
    slots[4].base.life_mut().energy = 85;

    slots[5].base.life_mut().water = 50;
    slots[5].base.life_mut().energy = 85;

    slots
}

fn configure_arm(world: &mut World, arm: AblationArm) {
    match arm {
        AblationArm::ControlFlagsOff => {}
        AblationArm::WaterContextOnly => {
            world.policy.water_residual_enabled = true;
            for slot in &mut world.entities {
                slot.learner.context_ttl_enabled = true;
            }
        }
        AblationArm::RefugeFirstIntervalOnly => {
            for slot in &mut world.entities {
                slot.memory.first_interval_refuge_enabled = true;
                slot.memory.refuge_outcomes_enabled = true;
            }
        }
        AblationArm::SpatialCuriosityOnly => {
            for slot in &mut world.entities {
                slot.memory.spatial_curiosity_enabled = true;
            }
        }
        AblationArm::ClimateAndBurrowsOnly => {
            world.environment.season_cycle = Some(SeasonCycle {
                period: 20,
                winter_bmr_surcharge: 2,
                winter_plant_regen_divisor: 2,
            });
        }
        AblationArm::SocialAndNicheOnly => {
            world.policy.juvenile_care_enabled = true;
            world.policy.ambush_predation_enabled = true;
            world.policy.local_coordination_enabled = true;
            world.environment.signals_enabled = true;
            world.environment.day_night = Some(DayNightCycle {
                period: 20,
                day_ticks: 12,
                night_vision_penalty: 1,
            });
        }
        AblationArm::AllV19Features => {
            world.policy.water_residual_enabled = true;
            world.policy.juvenile_care_enabled = true;
            world.policy.ambush_predation_enabled = true;
            world.policy.local_coordination_enabled = true;
            world.environment.signals_enabled = true;
            world.environment.day_night = Some(DayNightCycle {
                period: 20,
                day_ticks: 12,
                night_vision_penalty: 1,
            });
            world.environment.season_cycle = Some(SeasonCycle {
                period: 20,
                winter_bmr_surcharge: 2,
                winter_plant_regen_divisor: 2,
            });
            for slot in &mut world.entities {
                slot.learner.context_ttl_enabled = true;
                slot.memory.spatial_curiosity_enabled = true;
                slot.memory.first_interval_refuge_enabled = true;
                slot.memory.refuge_outcomes_enabled = true;
            }
        }
    }
}

fn build_world(topology: HabitatTopology, seed: u64, arm: AblationArm) -> (World, usize) {
    let ox = (seed % 5) as isize - 2;
    let oy = (seed % 3) as isize - 1;

    let mut world = World::with_seed(-12, 12, -12, 12, build_ablation_entities(seed), seed);
    world.logger = curious::logger::Logger::disabled();

    // Setup base resources common to all habitats:
    // 1. Water cells adjacent to thirsty Omnivore 5 at (ox, oy)
    world.map.set_cell(Position::new(ox + 1, oy), Cell::Water { amount: 50 });
    world.map.set_cell(Position::new(ox, oy + 1), Cell::Water { amount: 50 });
    world.map.set_cell(Position::new(ox, oy - 1), Cell::Food { amount: 50 });

    // Water cells adjacent to Omnivore 6 at (2 + ox, -2 + oy)
    world.map.set_cell(Position::new(2 + ox + 1, -2 + oy), Cell::Water { amount: 50 });
    world.map.set_cell(Position::new(2 + ox, -2 + oy + 1), Cell::Water { amount: 50 });
    world.map.set_cell(Position::new(2 + ox, -2 + oy - 1), Cell::Food { amount: 50 });

    // 2. Starting burrows adjacent to Herbivore 1 at (-6 + ox, -6 + oy) for refuge escape & first interval learning
    world.map.set_cell(Position::new(-7 + ox, -6 + oy), Cell::Burrow);
    world.map.set_cell(Position::new(-6 + ox, -7 + oy), Cell::Burrow);

    // 3. Clear empty roaming territory for Herbivore 2 around (6 + ox, -6 + oy)
    for dx in -2..=2 {
        for dy in -2..=2 {
            world.map.set_cell(Position::new(6 + ox + dx, -6 + oy + dy), Cell::Empty);
        }
    }

    // Now apply topology-specific terrain features:
    match topology {
        HabitatTopology::OpenPlains => {
            // General background food for ecosystem sustenance
            for x in (-10..=10).step_by(4) {
                for y in (-10..=10).step_by(4) {
                    if world.map.cell(Position::new(x, y)) == Some(&Cell::Empty) {
                        world.map.set_cell(Position::new(x, y), Cell::Food { amount: 30 });
                    }
                }
            }
        }
        HabitatTopology::CorridorMaze => {
            // Internal walls forming narrow corridors and obstacles
            for y in -10..=10 {
                // Vertical wall at x = -2 (leaving corridors at y = -3, y = 3)
                if y != -3 && y != 3 {
                    let wpos = Position::new(-2, y);
                    if world.map.cell(wpos) == Some(&Cell::Empty) {
                        world.map.set_cell(wpos, Cell::Wall);
                    }
                }
                // Vertical wall at x = 3 (leaving corridors at y = -5, y = 5)
                if y != -5 && y != 5 {
                    let wpos = Position::new(3, y);
                    if world.map.cell(wpos) == Some(&Cell::Empty) {
                        world.map.set_cell(wpos, Cell::Wall);
                    }
                }
            }
            // Some food patches along corridors
            for y in (-8..=8).step_by(4) {
                if world.map.cell(Position::new(0, y)) == Some(&Cell::Empty) {
                    world.map.set_cell(Position::new(0, y), Cell::Food { amount: 30 });
                }
            }
        }
        HabitatTopology::SeasonalScarcity => {
            // Sparse food sources: remove all existing food except 4 peripheral food patches
            for y in -12..=12 {
                for x in -12..=12 {
                    let pos = Position::new(x, y);
                    if matches!(world.map.cell(pos), Some(Cell::Food { .. }))
                        && pos != Position::new(ox, oy - 1)
                        && pos != Position::new(2 + ox, -2 + oy - 1)
                    {
                        world.map.set_cell(pos, Cell::Empty);
                    }
                }
            }
            world.map.set_cell(Position::new(-10, -10), Cell::Food { amount: 20 });
            world.map.set_cell(Position::new(10, 10), Cell::Food { amount: 20 });
            world.map.set_cell(Position::new(-10, 10), Cell::Food { amount: 20 });
            world.map.set_cell(Position::new(10, -10), Cell::Food { amount: 20 });
        }
    }

    // Synchronize environment from the final map
    world.environment = Environment::from_map(&world.map);
    let initial_burrow_count = world.environment.burrows.len();

    // Now configure the specific ablation arm:
    configure_arm(&mut world, arm);

    (world, initial_burrow_count)
}

fn run_arm(world: &mut World, initial_burrow_count: usize, ticks: usize) -> ArmRunMetrics {
    let mut max_known_sectors = 0usize;
    let mut max_context_entries = 0usize;
    let mut max_first_interval_outcomes = 0usize;
    let mut max_burrows = world.environment.burrows.len();
    let mut unique_signals_emitted = 0usize;

    for _ in 0..ticks {
        world.tick();

        let current_known: usize = world.entities.iter().map(|s| s.memory.known_sectors.len()).sum();
        max_known_sectors = max_known_sectors.max(current_known);

        let current_context: usize = world.entities.iter().map(|s| s.learner.context_entries.len()).sum();
        max_context_entries = max_context_entries.max(current_context);

        let current_first_interval: usize = world.entities.iter().map(|s| s.memory.first_interval_outcomes.len()).sum();
        max_first_interval_outcomes = max_first_interval_outcomes.max(current_first_interval);

        max_burrows = max_burrows.max(world.environment.burrows.len());

        unique_signals_emitted += world
            .environment
            .signals
            .iter()
            .filter(|s| s.emitted_tick == world.tick_counter)
            .count();
    }

    let newly_excavated_burrows = max_burrows.saturating_sub(initial_burrow_count);

    ArmRunMetrics {
        known_sectors: max_known_sectors,
        context_entries: max_context_entries,
        first_interval_outcomes: max_first_interval_outcomes,
        newly_excavated_burrows,
        unique_signals_emitted,
    }
}

/// Test 1: P3 single-feature ablation selectivity and isolation
/// Verifies that each single-feature ablation arm activates ONLY its target mechanism
/// and produces exactly 0 records in other disabled V19 mechanisms.
#[test]
fn test_p3_single_feature_ablation_selectivity_and_isolation() {
    let habitats = [
        HabitatTopology::OpenPlains,
        HabitatTopology::CorridorMaze,
        HabitatTopology::SeasonalScarcity,
    ];
    let seeds = [301u64, 302, 303, 304, 305];

    for &topology in &habitats {
        for &seed in &seeds {
            // 1. ControlFlagsOff: all 5 mechanisms must be 0
            let (mut world, initial_burrows) = build_world(topology, seed, AblationArm::ControlFlagsOff);
            let metrics = run_arm(&mut world, initial_burrows, 80);
            assert_eq!(metrics.known_sectors, 0, "ControlFlagsOff must have 0 known_sectors");
            assert_eq!(metrics.context_entries, 0, "ControlFlagsOff must have 0 context_entries");
            assert_eq!(metrics.first_interval_outcomes, 0, "ControlFlagsOff must have 0 first_interval_outcomes");
            assert_eq!(metrics.newly_excavated_burrows, 0, "ControlFlagsOff must have 0 newly_excavated_burrows");
            assert_eq!(metrics.unique_signals_emitted, 0, "ControlFlagsOff must have 0 unique_signals_emitted");

            // 2. WaterContextOnly: context_entries > 0, all others 0
            let (mut world, initial_burrows) = build_world(topology, seed, AblationArm::WaterContextOnly);
            let metrics = run_arm(&mut world, initial_burrows, 80);
            assert!(metrics.context_entries > 0, "WaterContextOnly must have >0 context_entries");
            assert_eq!(metrics.known_sectors, 0, "WaterContextOnly must have 0 known_sectors");
            assert_eq!(metrics.first_interval_outcomes, 0, "WaterContextOnly must have 0 first_interval_outcomes");
            assert_eq!(metrics.newly_excavated_burrows, 0, "WaterContextOnly must have 0 newly_excavated_burrows");
            assert_eq!(metrics.unique_signals_emitted, 0, "WaterContextOnly must have 0 unique_signals_emitted");

            // 3. RefugeFirstIntervalOnly: first_interval_outcomes > 0, all others 0
            let (mut world, initial_burrows) = build_world(topology, seed, AblationArm::RefugeFirstIntervalOnly);
            let metrics = run_arm(&mut world, initial_burrows, 80);
            assert!(metrics.first_interval_outcomes > 0, "RefugeFirstIntervalOnly must have >0 first_interval_outcomes");
            assert_eq!(metrics.known_sectors, 0, "RefugeFirstIntervalOnly must have 0 known_sectors");
            assert_eq!(metrics.context_entries, 0, "RefugeFirstIntervalOnly must have 0 context_entries");
            assert_eq!(metrics.newly_excavated_burrows, 0, "RefugeFirstIntervalOnly must have 0 newly_excavated_burrows");
            assert_eq!(metrics.unique_signals_emitted, 0, "RefugeFirstIntervalOnly must have 0 unique_signals_emitted");

            // 4. SpatialCuriosityOnly: known_sectors > 0, all others 0
            let (mut world, initial_burrows) = build_world(topology, seed, AblationArm::SpatialCuriosityOnly);
            let metrics = run_arm(&mut world, initial_burrows, 80);
            assert!(metrics.known_sectors > 0, "SpatialCuriosityOnly must have >0 known_sectors");
            assert_eq!(metrics.context_entries, 0, "SpatialCuriosityOnly must have 0 context_entries");
            assert_eq!(metrics.first_interval_outcomes, 0, "SpatialCuriosityOnly must have 0 first_interval_outcomes");
            assert_eq!(metrics.newly_excavated_burrows, 0, "SpatialCuriosityOnly must have 0 newly_excavated_burrows");
            assert_eq!(metrics.unique_signals_emitted, 0, "SpatialCuriosityOnly must have 0 unique_signals_emitted");

            // 5. ClimateAndBurrowsOnly: newly_excavated_burrows > 0, all others 0
            let (mut world, initial_burrows) = build_world(topology, seed, AblationArm::ClimateAndBurrowsOnly);
            let metrics = run_arm(&mut world, initial_burrows, 80);
            assert!(metrics.newly_excavated_burrows > 0, "ClimateAndBurrowsOnly must have >0 newly_excavated_burrows");
            assert_eq!(metrics.known_sectors, 0, "ClimateAndBurrowsOnly must have 0 known_sectors");
            assert_eq!(metrics.context_entries, 0, "ClimateAndBurrowsOnly must have 0 context_entries");
            assert_eq!(metrics.first_interval_outcomes, 0, "ClimateAndBurrowsOnly must have 0 first_interval_outcomes");
            assert_eq!(metrics.unique_signals_emitted, 0, "ClimateAndBurrowsOnly must have 0 unique_signals_emitted");

            // 6. SocialAndNicheOnly: unique_signals_emitted > 0, all others 0
            let (mut world, initial_burrows) = build_world(topology, seed, AblationArm::SocialAndNicheOnly);
            let metrics = run_arm(&mut world, initial_burrows, 80);
            assert!(metrics.unique_signals_emitted > 0, "SocialAndNicheOnly must have >0 unique_signals_emitted");
            assert_eq!(metrics.known_sectors, 0, "SocialAndNicheOnly must have 0 known_sectors");
            assert_eq!(metrics.context_entries, 0, "SocialAndNicheOnly must have 0 context_entries");
            assert_eq!(metrics.first_interval_outcomes, 0, "SocialAndNicheOnly must have 0 first_interval_outcomes");
            assert_eq!(metrics.newly_excavated_burrows, 0, "SocialAndNicheOnly must have 0 newly_excavated_burrows");

            // 7. AllV19Features: all 5 mechanisms > 0
            let (mut world, initial_burrows) = build_world(topology, seed, AblationArm::AllV19Features);
            let metrics = run_arm(&mut world, initial_burrows, 80);
            assert!(metrics.known_sectors > 0, "AllV19Features must have >0 known_sectors");
            assert!(metrics.context_entries > 0, "AllV19Features must have >0 context_entries");
            assert!(metrics.first_interval_outcomes > 0, "AllV19Features must have >0 first_interval_outcomes");
            assert!(metrics.newly_excavated_burrows > 0, "AllV19Features must have >0 newly_excavated_burrows");
            assert!(metrics.unique_signals_emitted > 0, "AllV19Features must have >0 unique_signals_emitted");
        }
    }
}

/// Test 2: P3 multi-habitat ablation matrix and determinism
/// Runs the 3 habitats × 5 seeds × 7 arms matrix, verifying:
/// 1. 100% deterministic replay (State capture and RNG state).
/// 2. Cross-habitat dynamic trajectory divergence.
#[test]
fn test_p3_multi_habitat_ablation_matrix_and_determinism() {
    let habitats = [
        HabitatTopology::OpenPlains,
        HabitatTopology::CorridorMaze,
        HabitatTopology::SeasonalScarcity,
    ];
    let seeds = [301u64, 302, 303, 304, 305];
    let arms = [
        AblationArm::ControlFlagsOff,
        AblationArm::WaterContextOnly,
        AblationArm::RefugeFirstIntervalOnly,
        AblationArm::SpatialCuriosityOnly,
        AblationArm::ClimateAndBurrowsOnly,
        AblationArm::SocialAndNicheOnly,
        AblationArm::AllV19Features,
    ];

    for &seed in &seeds {
        for &arm in &arms {
            let mut final_states = Vec::new();
            let mut final_rngs = Vec::new();

            for &topology in &habitats {
                // Run 1
                let (mut world1, initial_burrows1) = build_world(topology, seed, arm);
                let _metrics1 = run_arm(&mut world1, initial_burrows1, 80);
                let state1 = State::capture(&world1);
                let rng1 = world1.rng.state();

                // Replay Run 2 to verify determinism
                let (mut world2, initial_burrows2) = build_world(topology, seed, arm);
                let _metrics2 = run_arm(&mut world2, initial_burrows2, 80);
                let state2 = State::capture(&world2);
                let rng2 = world2.rng.state();

                // Exact deterministic replay
                assert_eq!(rng1, rng2, "RNG state must be identical upon replay for {:?}, seed {}", topology, seed);
                assert_eq!(
                    serde_json::to_value(&state1).unwrap(),
                    serde_json::to_value(&state2).unwrap(),
                    "Captured state must be identical upon replay for {:?}, seed {}", topology, seed
                );

                final_states.push(state1);
                final_rngs.push(rng1);
            }

            // Cross-habitat divergence: OpenPlains, CorridorMaze, and SeasonalScarcity must diverge
            let plains_val = serde_json::to_value(&final_states[0]).unwrap();
            let maze_val = serde_json::to_value(&final_states[1]).unwrap();
            let scarcity_val = serde_json::to_value(&final_states[2]).unwrap();

            assert_ne!(
                plains_val, maze_val,
                "OpenPlains and CorridorMaze must diverge for seed {}, arm {:?}", seed, arm
            );
            assert_ne!(
                plains_val, scarcity_val,
                "OpenPlains and SeasonalScarcity must diverge for seed {}, arm {:?}", seed, arm
            );
            assert_ne!(
                maze_val, scarcity_val,
                "CorridorMaze and SeasonalScarcity must diverge for seed {}, arm {:?}", seed, arm
            );
        }
    }
}
