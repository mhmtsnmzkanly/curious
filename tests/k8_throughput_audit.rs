#![allow(dead_code, unused_imports)]

//! K8 Throughput & Memory Bounds Audit and P3 Empirical Comparison Tests.
//!
//! Validates:
//! 1. K8: Short-load multi-species world (30 founders: 10 Herbivores, 10 Carnivores, 10 Omnivores)
//!    running 250 ticks with day/night cycles, signals enabled, and injury capacity penalty.
//!    Verifies bounded signal count (<= 64), bounded dangers (<= 8), bounded sites (<= 32),
//!    and checkpoint JSON serialization size < 1MB.
//! 2. P3: Fixed 5 held-out seeds [101, 102, 103, 104, 105]:
//!    - Genuine independent World::with_seed runs (80 ticks each) verifying deterministic
//!      reproduction (world_a == world_b) and distinct pairwise RNG/state trajectories across seeds.
//!    - Synthetic candidate mechanism checks (perturbed by seed):
//!      * Water context-aligned residual candidate vs. unlearning baseline under drying schedule.
//!      * First-interval refuge damage learner vs. unlearning baseline with positive vs. censored route.
//! 3. P3: Fixed 5 held-out seeds [201, 202, 203, 204, 205]:
//!    - Genuine independent World::with_seed runs (80 ticks each) verifying deterministic
//!      reproduction (world_a == world_b) and distinct pairwise RNG/state trajectories across seeds.
//!    - Synthetic candidate mechanism checks (perturbed by seed):
//!      * Local coordination 3-arm harness (ControlSolo vs UnverifiedIgnored vs VerifiedLocalCoordination).
//!      * Ambush specialist vs Pursuit specialist predation performance.
//!      * Carnivore vs Omnivore putrid corpse scavenging spoilage resistance and stock-flow conservation.

#[path = "../examples/support/water_context_residual_candidate.rs"]
mod water_context_residual_candidate;
#[path = "../examples/support/refuge_first_interval_candidate.rs"]
mod refuge_first_interval_candidate;
#[path = "../examples/support/local_coordination_candidate.rs"]
mod local_coordination_candidate;
#[path = "../examples/support/ambush_predation_candidate.rs"]
mod ambush_predation_candidate;
#[path = "../examples/support/corpse_scavenging_candidate.rs"]
mod corpse_scavenging_candidate;

use curious::{
    creatures::{carnivore::CarnivoreEntity, herbivore::HerbivoreEntity, omnivore::OmnivoreEntity},
    entity::{
        controller::EntityController,
        genetics::Genome,
        phase::EntityPhase,
        species::Species,
    },
    environment::{DayNightCycle, SeasonCycle, SignalKind as EnvSignalKind, SignalTrace, MAX_ENVIRONMENT_SIGNALS},
    map::{cell::Cell, movement::Position, Map},
    persistence::State,
    world::{EntitySlot, World},
};

use water_context_residual_candidate::{
    choose_water_candidate, WaterAttemptOutcome, WaterCandidateInput, WaterContextKey,
    WaterContextResidualTable,
};
use refuge_first_interval_candidate::{
    FirstIntervalKey, FirstIntervalLearner, FirstIntervalOutcome,
    Species as CandidateSpecies,
};
use local_coordination_candidate::{
    choose_coordinated_step, process_local_alarm, resolve_alarm_call, AlarmPerception,
    AlarmResolution, ArmMetrics, CoordinationArm, LocalAgent, LocalCoord, VerifiedAlarmEvent,
};
use ambush_predation_candidate::{
    resolve_attack, update_ambush_state, AmbushState, CoverKind,
    PredatorSpecialization, PredatorState, SpecializationProfile,
};
use corpse_scavenging_candidate::{
    DecayingCorpse, FreshnessStage,
};

#[test]
fn k8_short_load_tick_throughput_and_memory_bounds_are_verified() {
    let mut slots = Vec::with_capacity(30);

    // 10 Herbivores (id 1..=10)
    for i in 1..=10 {
        let x = -10isize + (((i - 1) % 5) as isize) * 4;
        let y = -10isize + (((i - 1) / 5) as isize) * 4;
        slots.push(EntitySlot::new(
            i,
            Position::new(x, y),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        ));
    }

    // 10 Carnivores (id 11..=20)
    for i in 11..=20 {
        let idx = (i - 11) as isize;
        let x = -8isize + (idx % 5) * 4;
        let y = 4isize + (idx / 5) * 4;
        slots.push(EntitySlot::new(
            i,
            Position::new(x, y),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(CarnivoreEntity::default()),
        ));
    }

    // 10 Omnivores (id 21..=30)
    for i in 21..=30 {
        let idx = (i - 21) as isize;
        let x = -6isize + (idx % 5) * 3;
        let y = -2isize + (idx / 5) * 3;
        slots.push(EntitySlot::new(
            i,
            Position::new(x, y),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(OmnivoreEntity::default()),
        ));
    }

    let mut world = World::with_seed(-15, 15, -15, 15, slots, 777);
    world.map = Map::new(-15, 15, -15, 15);
    world.logger = curious::logger::Logger::disabled();

    // Populate environmental resource patches
    for x in (-12..=12).step_by(6) {
        for y in (-12..=12).step_by(6) {
            world.map.set_cell(Position::new(x, y), Cell::Food { amount: 50 });
            world.map.set_cell(Position::new(x + 1, y), Cell::Water { amount: 50 });
        }
    }

    // Configure environment & policy as required by K8 audit
    world.environment.day_night = Some(DayNightCycle {
        period: 50,
        day_ticks: 30,
        night_vision_penalty: 2,
    });
    world.environment.signals_enabled = true;
    world.policy.injury_capacity_penalty = true;

    // Run short-load test for 250 ticks with periodic signal generation
    let start_time = std::time::Instant::now();
    let total_ticks = 250;

    let mut max_dangers = 0;
    let mut max_sites = 0;
    let mut total_active_entity_ticks = 0usize;

    for tick in 1..=total_ticks {
        // Emit periodic signals to test bounded eviction
        if tick % 2 == 0 {
            world.environment.emit_signal(SignalTrace {
                origin: Position::new((tick as isize % 15) - 7, (tick as isize % 11) - 5),
                kind: EnvSignalKind::Footstep,
                intensity: 60,
                radius: 4,
                decay_per_tick: 2,
                emitted_tick: tick,
            });
        }
        world.tick();

        // Sample memory bounds for all active entities across the run
        for slot in &world.entities {
            if slot.phase == EntityPhase::Active {
                total_active_entity_ticks += 1;
                let mem = &slot.memory;
                let dangers_len = mem.dangers.len();
                let sites_len = mem.sites.len();

                if dangers_len > max_dangers {
                    max_dangers = dangers_len;
                }
                if sites_len > max_sites {
                    max_sites = sites_len;
                }

                assert!(
                    dangers_len <= 8,
                    "Entity {} memory dangers length {} > 8 at tick {}",
                    slot.id,
                    dangers_len,
                    tick
                );
                assert!(
                    sites_len <= 32,
                    "Entity {} memory sites length {} > 32 at tick {}",
                    slot.id,
                    sites_len,
                    tick
                );
            }
        }
    }

    let elapsed = start_time.elapsed();
    let elapsed_ms = elapsed.as_millis();
    let ticks_per_sec = if elapsed_ms > 0 {
        (total_ticks as u128 * 1000) / elapsed_ms
    } else {
        total_ticks as u128 * 1000
    };

    println!(
        "K8 Audit: 250 ticks executed in {} ms (approx {} ticks/sec)",
        elapsed_ms, ticks_per_sec
    );
    println!(
        "K8 Audit: Total active entity ticks evaluated = {}, max_dangers={}, max_sites={}",
        total_active_entity_ticks, max_dangers, max_sites
    );

    assert!(
        total_active_entity_ticks > 0,
        "Expected active entities during the 250-tick run"
    );

    // Assert environment signal retention bounds
    assert!(
        world.environment.signals.len() <= MAX_ENVIRONMENT_SIGNALS,
        "Environment signals count {} exceeded MAX_ENVIRONMENT_SIGNALS ({})",
        world.environment.signals.len(),
        MAX_ENVIRONMENT_SIGNALS
    );

    // Assert JSON checkpoint serialization bounds (< 1 MB)
    let state = State::capture(&world);
    let serialized_state = serde_json::to_string(&state).unwrap();
    let state_bytes = serialized_state.len();

    println!("K8 Audit: Serialized checkpoint size = {} bytes", state_bytes);
    assert!(
        state_bytes < 1_000_000,
        "Checkpoint serialized size {} bytes exceeded 1,000,000 byte limit",
        state_bytes
    );
}

fn build_held_out_seed_entities(seed: u64) -> Vec<EntitySlot> {
    let ox = (seed % 5) as isize - 2;
    let oy = (seed % 3) as isize - 1;
    vec![
        EntitySlot::new(
            1,
            Position::new(-6 + ox, -6 + oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        ),
        EntitySlot::new(
            2,
            Position::new(-3 + ox, -3 + oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        ),
        EntitySlot::new(
            3,
            Position::new(4 + ox, 4 + oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(CarnivoreEntity::default()),
        ),
        EntitySlot::new(
            4,
            Position::new(6 + ox, 6 + oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(CarnivoreEntity::default()),
        ),
        EntitySlot::new(
            5,
            Position::new(ox, oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(OmnivoreEntity::default()),
        ),
        EntitySlot::new(
            6,
            Position::new(2 + ox, -2 + oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(OmnivoreEntity::default()),
        ),
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldSeedRunResult {
    pub seed: u64,
    pub rng_state: u64,
    pub active_entity_count: usize,
    pub serialized_state: String,
}

fn run_verified_world_seed_trajectory(seed: u64, ticks: usize) -> WorldSeedRunResult {
    let mut world_a = World::with_seed(-12, 12, -12, 12, build_held_out_seed_entities(seed), seed);
    world_a.logger = curious::logger::Logger::disabled();

    let mut world_b = World::with_seed(-12, 12, -12, 12, build_held_out_seed_entities(seed), seed);
    world_b.logger = curious::logger::Logger::disabled();

    for _ in 0..ticks {
        world_a.tick();
        world_b.tick();
    }

    // Exact state and RNG reproduction for the fixed seed
    assert_eq!(
        world_a.rng.state(),
        world_b.rng.state(),
        "RNG state mismatch on identical seed {}",
        seed
    );
    let state_a = State::capture(&world_a);
    let state_b = State::capture(&world_b);
    assert_eq!(
        serde_json::to_value(&state_a).unwrap(),
        serde_json::to_value(&state_b).unwrap(),
        "State checkpoint mismatch on identical seed {}",
        seed
    );

    let active_count = world_a
        .entities
        .iter()
        .filter(|s| s.phase == EntityPhase::Active)
        .count();

    WorldSeedRunResult {
        seed,
        rng_state: world_a.rng.state(),
        active_entity_count: active_count,
        serialized_state: serde_json::to_string(&state_a).unwrap(),
    }
}

fn verify_distinct_world_trajectories(runs: &[WorldSeedRunResult]) {
    for i in 0..runs.len() {
        for j in (i + 1)..runs.len() {
            assert_ne!(
                runs[i].rng_state,
                runs[j].rng_state,
                "Distinct seeds {} and {} must produce distinct RNG states!",
                runs[i].seed,
                runs[j].seed
            );
            assert_ne!(
                runs[i].serialized_state,
                runs[j].serialized_state,
                "Distinct seeds {} and {} must produce distinct World states!",
                runs[i].seed,
                runs[j].seed
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicEntitySnapshot {
    pub id: usize,
    pub pos: Position,
    pub phase: EntityPhase,
    pub energy: usize,
    pub water: usize,
    pub health: usize,
    pub stamina: usize,
    pub generation: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicWorldSnapshot {
    pub entities: Vec<DynamicEntitySnapshot>,
    pub map_resources: Vec<(Position, Cell)>,
}

impl DynamicWorldSnapshot {
    pub fn capture(world: &World) -> Self {
        let mut entities: Vec<DynamicEntitySnapshot> = world
            .entities
            .iter()
            .map(|slot| {
                let life = slot.base.life();
                DynamicEntitySnapshot {
                    id: slot.id,
                    pos: slot.pos,
                    phase: slot.phase,
                    energy: life.energy,
                    water: life.water,
                    health: life.health,
                    stamina: life.stamina,
                    generation: slot.generation,
                }
            })
            .collect();
        entities.sort_by_key(|e| e.id);

        let mut map_resources = Vec::new();
        for y in world.map.min_y..=world.map.max_y {
            for x in world.map.min_x..=world.map.max_x {
                let pos = Position::new(x, y);
                if let Some(cell) = world.map.cell(pos) {
                    if !matches!(cell, Cell::Empty) {
                        map_resources.push((pos, cell.clone()));
                    }
                }
            }
        }
        map_resources.sort_by_key(|(pos, _)| (pos.y, pos.x));

        Self {
            entities,
            map_resources,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FlagOnComparisonResult {
    pub seed: u64,
    pub control_rng_state: u64,
    pub treatment_rng_state: u64,
    pub control_dynamic_snapshot: DynamicWorldSnapshot,
    pub treatment_dynamic_snapshot: DynamicWorldSnapshot,
    pub treatment_known_sectors_total: usize,
    pub treatment_context_entries_total: usize,
    pub treatment_first_interval_outcomes_total: usize,
    pub treatment_signals_emitted_total: usize,
    pub treatment_unique_signals_emitted_total: usize,
    pub treatment_burrows_total: usize,
    pub treatment_newly_excavated_burrows_total: usize,
}

fn build_flag_comparison_entities(seed: u64) -> Vec<EntitySlot> {
    let ox = (seed % 5) as isize - 2;
    let oy = (seed % 3) as isize - 1;
    let mut slots = vec![
        EntitySlot::new(
            1,
            Position::new(-6 + ox, -6 + oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        ),
        EntitySlot::new(
            2,
            Position::new(6 + ox, -6 + oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        ),
        EntitySlot::new(
            3,
            Position::new(-4 + ox, -6 + oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(CarnivoreEntity::default()),
        ),
        EntitySlot::new(
            4,
            Position::new(6 + ox, 6 + oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(CarnivoreEntity::default()),
        ),
        EntitySlot::new(
            5,
            Position::new(ox, oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(OmnivoreEntity::default()),
        ),
        EntitySlot::new(
            6,
            Position::new(2 + ox, -2 + oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(OmnivoreEntity::default()),
        ),
    ];

    // Configure initial physiological state identically on both Control and Treatment
    // Entity 2: Herbivore roaming in empty space with high energy and stamina for winter burrow digging
    slots[1].base.life_mut().energy = 95;
    slots[1].base.life_mut().stamina = 30;

    // Entity 5: Omnivore with low water to immediately trigger water seeking & residual learning
    slots[4].base.life_mut().water = 15;
    slots[4].base.life_mut().energy = 85;

    // Entity 6: Omnivore with moderate water
    slots[5].base.life_mut().water = 50;
    slots[5].base.life_mut().energy = 85;

    slots
}

fn run_flag_on_vs_flag_off_comparison(seed: u64, ticks: usize) -> FlagOnComparisonResult {
    let ox = (seed % 5) as isize - 2;
    let oy = (seed % 3) as isize - 1;

    // 1. Initialize Control (Flags Off) and Treatment (Flags On) with identical seed, map & entities
    let mut control_world =
        World::with_seed(-12, 12, -12, 12, build_flag_comparison_entities(seed), seed);
    let mut treatment_world =
        World::with_seed(-12, 12, -12, 12, build_flag_comparison_entities(seed), seed);

    for w in [&mut control_world, &mut treatment_world] {
        w.logger = curious::logger::Logger::disabled();

        // 1. Water cells adjacent to Omnivore 5 at (ox, oy)
        w.map.set_cell(Position::new(ox + 1, oy), Cell::Water { amount: 50 });
        w.map.set_cell(Position::new(ox, oy + 1), Cell::Water { amount: 50 });
        w.map.set_cell(Position::new(ox, oy - 1), Cell::Food { amount: 50 });

        // Water cells adjacent to Omnivore 6 at (2 + ox, -2 + oy)
        w.map.set_cell(Position::new(2 + ox + 1, -2 + oy), Cell::Water { amount: 50 });
        w.map.set_cell(Position::new(2 + ox, -2 + oy + 1), Cell::Water { amount: 50 });
        w.map.set_cell(Position::new(2 + ox, -2 + oy - 1), Cell::Food { amount: 50 });

        // 2. Burrows adjacent to Herbivore 1 at (-6 + ox, -6 + oy) for refuge escape & first interval learning
        w.map.set_cell(Position::new(-7 + ox, -6 + oy), Cell::Burrow);
        w.map.set_cell(Position::new(-6 + ox, -7 + oy), Cell::Burrow);

        // General background food for ecosystem sustenance
        for x in (-10..=10).step_by(4) {
            for y in (-10..=10).step_by(4) {
                if w.map.cell(Position::new(x, y)) == Some(&Cell::Empty) {
                    w.map.set_cell(Position::new(x, y), Cell::Food { amount: 30 });
                }
            }
        }

        // Ensure clear empty roaming territory for Herbivore 2 around (6 + ox, -6 + oy)
        for dx in -2..=2 {
            for dy in -2..=2 {
                w.map.set_cell(Position::new(6 + ox + dx, -6 + oy + dy), Cell::Empty);
            }
        }
    }

    // 2. Enable V19 features exclusively on treatment_world BEFORE running ticks
    treatment_world.policy.water_residual_enabled = true;
    treatment_world.policy.injury_capacity_penalty = true;
    treatment_world.policy.juvenile_care_enabled = true;
    treatment_world.policy.ambush_predation_enabled = true;
    treatment_world.policy.local_coordination_enabled = true;

    treatment_world.environment.day_night = Some(DayNightCycle {
        period: 20,
        day_ticks: 12,
        night_vision_penalty: 1,
    });
    treatment_world.environment.signals_enabled = true;
    treatment_world.environment.season_cycle = Some(SeasonCycle {
        period: 20,
        winter_bmr_surcharge: 2,
        winter_plant_regen_divisor: 2,
    });

    for slot in &mut treatment_world.entities {
        slot.learner.context_ttl_enabled = true;
        slot.memory.spatial_curiosity_enabled = true;
        slot.memory.first_interval_refuge_enabled = true;
        slot.memory.refuge_outcomes_enabled = true;
    }

    let initial_static_burrows = [
        Position::new(-7 + ox, -6 + oy),
        Position::new(-6 + ox, -7 + oy),
    ];

    // 3. Run both worlds for 80 ticks and sample mechanism metrics
    let mut treatment_signals_emitted_total = 0usize;
    let mut treatment_unique_signals_emitted_total = 0usize;
    let mut treatment_burrows_total = 0usize;
    let mut newly_excavated_burrows: std::collections::HashSet<Position> = std::collections::HashSet::new();
    let mut treatment_known_sectors_total = 0usize;
    let mut treatment_context_entries_total = 0usize;
    let mut treatment_first_interval_outcomes_total = 0usize;

    for _ in 0..ticks {
        control_world.tick();
        treatment_world.tick();

        treatment_signals_emitted_total += treatment_world.environment.signals.len();
        treatment_unique_signals_emitted_total += treatment_world
            .environment
            .signals
            .iter()
            .filter(|s| s.emitted_tick == treatment_world.tick_counter)
            .count();

        treatment_burrows_total = treatment_burrows_total.max(treatment_world.environment.burrows.len());
        for b in &treatment_world.environment.burrows {
            if !initial_static_burrows.contains(&b.pos) {
                newly_excavated_burrows.insert(b.pos);
            }
        }

        let known_now: usize = treatment_world.entities.iter().map(|s| s.memory.known_sectors.len()).sum();
        treatment_known_sectors_total = treatment_known_sectors_total.max(known_now);

        let context_now: usize = treatment_world.entities.iter().map(|s| s.learner.context_entries.len()).sum();
        treatment_context_entries_total = treatment_context_entries_total.max(context_now);

        let first_interval_now: usize = treatment_world.entities.iter().map(|s| s.memory.first_interval_outcomes.len()).sum();
        treatment_first_interval_outcomes_total = treatment_first_interval_outcomes_total.max(first_interval_now);
    }

    let treatment_newly_excavated_burrows_total = newly_excavated_burrows.len();

    assert!(
        treatment_newly_excavated_burrows_total > 0,
        "Seed {}: expected at least one newly excavated burrow outside initial static burrows",
        seed
    );
    assert!(
        treatment_unique_signals_emitted_total > 0,
        "Seed {}: expected at least one unique signal emitted during World::tick",
        seed
    );

    let control_dynamic_snapshot = DynamicWorldSnapshot::capture(&control_world);
    let treatment_dynamic_snapshot = DynamicWorldSnapshot::capture(&treatment_world);
    let control_rng_state = control_world.rng.state();
    let treatment_rng_state = treatment_world.rng.state();

    FlagOnComparisonResult {
        seed,
        control_rng_state,
        treatment_rng_state,
        control_dynamic_snapshot,
        treatment_dynamic_snapshot,
        treatment_known_sectors_total,
        treatment_context_entries_total,
        treatment_first_interval_outcomes_total,
        treatment_signals_emitted_total,
        treatment_unique_signals_emitted_total,
        treatment_burrows_total,
        treatment_newly_excavated_burrows_total,
    }
}

fn run_coordination_harness_with_seed(
    arm: CoordinationArm,
    max_ticks: u32,
    seed: u64,
) -> ArmMetrics {
    let threat_offset = (seed % 3) as i32;
    let mut threat_pos = LocalCoord::new(8 + threat_offset, 0);

    let initial_energy = 100 + (seed % 20) as u16;
    let initial_stamina = 100 + (seed % 10) as u16;

    // Focal agent starts at (0, 0), vision range 3
    let mut focal = LocalAgent::new(1, LocalCoord::new(0, 0), initial_energy, initial_stamina, 3);

    // Sender conspecific at (3, 0), vision range 5 (can see threat at dist <= 5)
    let has_sender = arm != CoordinationArm::ControlSolo;
    let mut sender = LocalAgent::new(2, LocalCoord::new(3, 0), initial_energy, initial_stamina, 5);

    let mut min_threat_dist = focal.pos.chebyshev_distance(&threat_pos);
    let mut focal_survived = true;

    let mut alarms_emitted = 0;
    let mut alarms_verified_paid = 0;
    let mut alarms_perceived_by_focal = 0;
    let mut alarms_rejected_or_unperceived = 0;

    let initial_energy_focal = focal.energy;
    let initial_stamina_focal = focal.stamina;
    let initial_energy_sender = sender.energy;
    let initial_stamina_sender = sender.stamina;

    let energy_cost = 10 + (seed % 3) as u16;
    let stamina_cost = 5 + (seed % 2) as u16;

    let mut last_alarm: Option<VerifiedAlarmEvent> = None;

    for tick in 0..max_ticks as u64 {
        // 1. Sender evaluates threat if sender exists
        if has_sender && sender.is_alive {
            let dist_to_threat = sender.pos.chebyshev_distance(&threat_pos);
            if dist_to_threat <= sender.vision_range {
                alarms_emitted += 1;
                let threat_sec = sender.pos.sector_to(&threat_pos);
                match arm {
                    CoordinationArm::VerifiedLocalCoordination => {
                        if let AlarmResolution::Paid(evt) =
                            resolve_alarm_call(&mut sender, threat_sec, energy_cost, stamina_cost, tick)
                        {
                            alarms_verified_paid += 1;
                            last_alarm = Some(evt);
                        }
                    }
                    CoordinationArm::UnverifiedIgnored => {
                        last_alarm = Some(VerifiedAlarmEvent {
                            sender_id: sender.id,
                            sender_pos: sender.pos,
                            threat_sector: threat_sec,
                            tick,
                            paid_energy: 0,
                            paid_stamina: 0,
                            is_paid: false,
                        });
                    }
                    CoordinationArm::ControlSolo => {}
                }
            }
        }

        // 2. Focal agent evaluates perception and decides step
        let perceived = if let Some(ref alarm) = last_alarm {
            let p = process_local_alarm(&focal, alarm, tick);
            match p {
                AlarmPerception::Perceived { .. } => {
                    alarms_perceived_by_focal += 1;
                    Some(p)
                }
                _ => {
                    alarms_rejected_or_unperceived += 1;
                    None
                }
            }
        } else {
            None
        };

        // Default foraging/wandering intent is East (sector 2) towards (8, 0)
        let default_sector = 2;
        let chosen_sector = choose_coordinated_step(&mut focal, perceived.as_ref(), default_sector);
        focal.pos = focal.pos.step_in_sector(chosen_sector);
        focal.energy = focal.energy.saturating_sub(1);

        // Threat moves 1 step West towards focal
        threat_pos.x -= 1;

        let current_dist = focal.pos.chebyshev_distance(&threat_pos);
        if current_dist < min_threat_dist {
            min_threat_dist = current_dist;
        }

        // Threat attack radius is 1
        if current_dist <= 1 {
            focal_survived = false;
            focal.is_alive = false;
            break;
        }
    }

    ArmMetrics {
        arm,
        focal_survived,
        min_threat_distance_to_focal: min_threat_dist,
        total_energy_spent_focal: initial_energy_focal.saturating_sub(focal.energy),
        total_stamina_spent_focal: initial_stamina_focal.saturating_sub(focal.stamina),
        total_energy_spent_sender: initial_energy_sender.saturating_sub(sender.energy),
        total_stamina_spent_sender: initial_stamina_sender.saturating_sub(sender.stamina),
        alarms_emitted,
        alarms_verified_paid,
        alarms_perceived_by_focal,
        alarms_rejected_or_unperceived,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaterCandidateSeedComparison {
    pub seed: u64,
    pub baseline_water_gained: u32,
    pub candidate_water_gained: u32,
    pub baseline_failed_attempts: usize,
    pub candidate_failed_attempts: usize,
    pub candidate_switches_to_alt: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RefugeFirstIntervalSeedComparison {
    pub seed: u64,
    pub baseline_chosen_candidate: usize,
    pub learner_chosen_candidate: usize,
    pub route1_expected_damage: f64,
    pub route2_expected_damage: f64,
}

#[test]
fn p3_held_out_seeds_water_context_and_first_interval_refuge_comparison() {
    let seeds = [101u64, 102, 103, 104, 105];

    // --- Part 1: Real World Multi-Seed Flag-On vs Flag-Off Causal Comparison ---
    let mut flag_comparison_results = Vec::new();
    for &seed in &seeds {
        let cmp = run_flag_on_vs_flag_off_comparison(seed, 80);
        assert_ne!(
            cmp.control_dynamic_snapshot, cmp.treatment_dynamic_snapshot,
            "Seed {}: Treatment dynamic snapshot must causally diverge from Control dynamic snapshot",
            seed
        );
        assert_ne!(
            cmp.control_rng_state, cmp.treatment_rng_state,
            "Seed {}: Treatment RNG state must diverge from Control RNG state",
            seed
        );
        assert!(
            cmp.treatment_known_sectors_total > 0,
            "Seed {}: Expected spatial curiosity sectors > 0 in treatment world",
            seed
        );
        assert!(
            cmp.treatment_context_entries_total > 0,
            "Seed {}: Expected water context entries > 0 in treatment world",
            seed
        );
        assert!(
            cmp.treatment_first_interval_outcomes_total > 0,
            "Seed {}: Expected refuge first interval outcomes > 0 in treatment world",
            seed
        );
        assert!(
            cmp.treatment_signals_emitted_total > 0,
            "Seed {}: Expected signals emitted > 0 in treatment world",
            seed
        );
        assert!(
            cmp.treatment_unique_signals_emitted_total > 0,
            "Seed {}: expected at least one unique signal emitted during World::tick",
            seed
        );
        assert!(
            cmp.treatment_burrows_total > 0,
            "Seed {}: Expected winter burrows dug > 0 in treatment world",
            seed
        );
        assert!(
            cmp.treatment_newly_excavated_burrows_total > 0,
            "Seed {}: expected at least one newly excavated burrow outside initial static burrows",
            seed
        );
        flag_comparison_results.push(cmp);
    }

    assert!(
        flag_comparison_results
            .iter()
            .all(|r| r.control_dynamic_snapshot != r.treatment_dynamic_snapshot),
        "All seeds must dynamically diverge between control and treatment"
    );
    assert!(
        flag_comparison_results
            .iter()
            .all(|r| r.control_rng_state != r.treatment_rng_state),
        "All seeds must produce divergent RNG trajectories between control and treatment"
    );
    assert!(
        flag_comparison_results.iter().all(|r| r.treatment_known_sectors_total > 0),
        "All seeds must record spatial curiosity known sectors when enabled"
    );
    assert!(
        flag_comparison_results.iter().all(|r| r.treatment_context_entries_total > 0),
        "All seeds must record water context entries when enabled"
    );
    assert!(
        flag_comparison_results.iter().all(|r| r.treatment_first_interval_outcomes_total > 0),
        "All seeds must record first interval refuge outcomes when enabled"
    );
    assert!(
        flag_comparison_results.iter().all(|r| r.treatment_signals_emitted_total > 0),
        "All seeds must emit and observe signals when enabled"
    );
    assert!(
        flag_comparison_results.iter().all(|r| r.treatment_unique_signals_emitted_total > 0),
        "All seeds must emit at least one unique signal during World::tick"
    );
    assert!(
        flag_comparison_results.iter().all(|r| r.treatment_burrows_total > 0),
        "All seeds must dig winter burrows when enabled"
    );
    assert!(
        flag_comparison_results.iter().all(|r| r.treatment_newly_excavated_burrows_total > 0),
        "All seeds must excavate at least one new burrow outside initial static burrows"
    );

    // Also verify deterministic reproduction & distinct trajectories across seeds
    let mut world_runs = Vec::new();
    for &seed in &seeds {
        let run = run_verified_world_seed_trajectory(seed, 80);
        world_runs.push(run);
    }
    // Verify that every distinct seed produced a genuinely distinct World & RNG trajectory
    verify_distinct_world_trajectories(&world_runs);

    // --- Part 2: Synthetic Candidate Mechanism Checks (Perturbed by Seed) ---
    let mut water_results = Vec::new();
    let mut refuge_results = Vec::new();

    let speed = 2u8;
    let cost = 1u8;

    for &seed in &seeds {
        // 1. Water Candidate Synthetic Mechanism Check (positions and baselines perturbed by seed)
        let mut table = WaterContextResidualTable::new(true, true);
        let offset_x = (seed % 7) as i32 - 3;
        let offset_y = (seed % 5) as i32 - 2;
        let pos_dry = (10 + offset_x, offset_y);
        let pos_wet = (offset_x, 15 + offset_y);
        let base_dry = 450 + (seed % 20) as i64;
        let base_wet = 340 + (seed % 15) as i64;
        let key_dry = WaterContextKey::new(pos_dry, speed, cost).unwrap();
        let key_wet = WaterContextKey::new(pos_wet, speed, cost).unwrap();

        let candidates = vec![
            WaterCandidateInput {
                source_pos: pos_dry,
                choice_baseline: base_dry,
                steps_len: 4,
            },
            WaterCandidateInput {
                source_pos: pos_wet,
                choice_baseline: base_wet,
                steps_len: 5,
            },
        ];

        let mut baseline_water = 0u32;
        let mut candidate_water = 0u32;
        let mut baseline_fails = 0usize;
        let mut candidate_fails = 0usize;
        let mut candidate_switches = 0usize;

        // Schedule of 10 episodes:
        // Episodes 1..=3: Source A is wet (yields 20 water), Source B is wet (yields 20 water)
        // Episodes 4..=10: Source A dries up (yields 0 water, target verified missing), Source B remains wet
        for episode in 1..=10 {
            let tick = episode * 10;
            let a_dry = episode >= 4;

            // Baseline choice: pure greedy baseline comparison without residual learning
            let baseline_choice = if candidates[0].choice_baseline >= candidates[1].choice_baseline {
                0
            } else {
                1
            };

            if baseline_choice == 0 {
                if a_dry {
                    baseline_fails += 1;
                } else {
                    baseline_water += 20;
                }
            } else {
                baseline_water += 20;
            }

            // Context-aligned candidate choice
            let chosen = choose_water_candidate(&table, &candidates, speed, cost, tick).unwrap();
            if chosen == 1 {
                candidate_switches += 1;
            }

            if chosen == 0 {
                if a_dry {
                    candidate_fails += 1;
                    table.record_outcome(
                        key_dry,
                        WaterAttemptOutcome::FailedMissingResource {
                            movement_energy: 8,
                            ticks: 4,
                            choice_baseline: base_dry,
                            target_verified: true,
                        },
                        tick,
                    );
                } else {
                    candidate_water += 20;
                    table.record_outcome(
                        key_dry,
                        WaterAttemptOutcome::Reached {
                            water_gained: 20,
                            movement_energy: 8,
                            ticks: 4,
                            choice_baseline: base_dry,
                        },
                        tick,
                    );
                }
            } else {
                candidate_water += 20;
                table.record_outcome(
                    key_wet,
                    WaterAttemptOutcome::Reached {
                        water_gained: 20,
                        movement_energy: 10,
                        ticks: 5,
                        choice_baseline: base_wet,
                    },
                    tick,
                );
            }
        }

        water_results.push(WaterCandidateSeedComparison {
            seed,
            baseline_water_gained: baseline_water,
            candidate_water_gained: candidate_water,
            baseline_failed_attempts: baseline_fails,
            candidate_failed_attempts: candidate_fails,
            candidate_switches_to_alt: candidate_switches,
        });

        // 2. Refuge First-Interval Candidate Synthetic Mechanism Check (endpoints and damage perturbed by seed)
        let mut learner = FirstIntervalLearner::with_enabled(seed as usize, true, true);

        let endpoint_route1 = (8 + (seed % 5) as i32, (seed % 3) as i32);
        let endpoint_route2 = ((seed % 3) as i32, 8 + (seed % 5) as i32);

        // Candidate 0: Route straight into danger bearing (threat damage 20 + seed % 4)
        let key_route1 = FirstIntervalKey {
            subject_species: CandidateSpecies::Herbivore,
            threat_species: CandidateSpecies::Carnivore,
            threat_bearing_sector: 2,
            threat_distance_steps: 1,
            first_step_sector: 2,
            endpoint: endpoint_route1,
            subject_speed: 2,
        };

        // Candidate 1: Perpendicular evasion (threat damage 4 + seed % 2, later censored at tick 3+)
        let key_route2 = FirstIntervalKey {
            subject_species: CandidateSpecies::Herbivore,
            threat_species: CandidateSpecies::Carnivore,
            threat_bearing_sector: 2,
            threat_distance_steps: 1,
            first_step_sector: 0,
            endpoint: endpoint_route2,
            subject_speed: 2,
        };

        let route1_dmg = 20 + (seed % 4);
        let route2_dmg = 4 + (seed % 2);

        learner.record_episode(
            seed as usize,
            key_route1,
            FirstIntervalOutcome::CompletedFirstInterval {
                threat_damage: route1_dmg,
                paid_movement_energy: 2,
                later_full_route_censored: false,
            },
            1,
        );

        learner.record_episode(
            seed as usize,
            key_route2,
            FirstIntervalOutcome::CompletedFirstInterval {
                threat_damage: route2_dmg,
                paid_movement_energy: 2,
                later_full_route_censored: true,
            },
            2,
        );

        let ranking = learner.rank_first_interval_candidates(&[key_route1, key_route2], 3);
        let ranked_order = ranking.ranked_indices.clone().unwrap();

        let exp_dmg_1 = ranking.estimates[0].expected_threat_damage().unwrap();
        let exp_dmg_2 = ranking.estimates[1].expected_threat_damage().unwrap();

        refuge_results.push(RefugeFirstIntervalSeedComparison {
            seed,
            baseline_chosen_candidate: 0,
            learner_chosen_candidate: ranked_order[0],
            route1_expected_damage: exp_dmg_1,
            route2_expected_damage: exp_dmg_2,
        });
    }

    // Verify all seeds in Water Candidate Comparison:
    for res in &water_results {
        assert!(
            res.candidate_water_gained > res.baseline_water_gained,
            "Seed {}: candidate water {} not > baseline {}",
            res.seed,
            res.candidate_water_gained,
            res.baseline_water_gained
        );
        assert!(
            res.candidate_failed_attempts < res.baseline_failed_attempts,
            "Seed {}: candidate fails {} not < baseline {}",
            res.seed,
            res.candidate_failed_attempts,
            res.baseline_failed_attempts
        );
        assert!(
            res.candidate_switches_to_alt >= 5,
            "Seed {}: candidate switches {} < 5",
            res.seed,
            res.candidate_switches_to_alt
        );
    }

    // Verify all seeds in Refuge First-Interval Comparison:
    for res in &refuge_results {
        assert_eq!(
            res.learner_chosen_candidate, 1,
            "Seed {}: learner failed to prioritize low-damage route",
            res.seed
        );
        assert!(
            res.route2_expected_damage < res.route1_expected_damage,
            "Seed {}: route2 damage {} not < route1 damage {}",
            res.seed,
            res.route2_expected_damage,
            res.route1_expected_damage
        );
    }

    println!("World Runs (Seeds 101-105): {:?}", world_runs);
    println!("Water Results: {:?}", water_results);
    println!("Refuge Results: {:?}", refuge_results);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoordinationArmSeedComparison {
    pub seed: u64,
    pub solo_focal_survived: bool,
    pub solo_alarms_emitted: u32,
    pub unverified_focal_survived: bool,
    pub unverified_alarms_rejected: u32,
    pub verified_focal_survived: bool,
    pub verified_alarms_paid: u32,
    pub verified_sender_energy_spent: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NicheSpecializationSeedComparison {
    pub seed: u64,
    pub ambush_first_strike_damage: usize,
    pub pursuit_first_strike_damage: usize,
    pub ambush_stealth_reduction: usize,
    pub pursuit_stealth_reduction: usize,
    pub omnivore_putrid_health_penalty: usize,
    pub carnivore_putrid_health_penalty: usize,
    pub corpse_stock_flow_balanced: bool,
}

#[test]
fn p3_held_out_seeds_local_coordination_and_niche_comparison() {
    let seeds = [201u64, 202, 203, 204, 205];

    // --- Part 1: Real World Multi-Seed Flag-On vs Flag-Off Causal Comparison ---
    let mut flag_comparison_results = Vec::new();
    for &seed in &seeds {
        let cmp = run_flag_on_vs_flag_off_comparison(seed, 80);
        assert_ne!(
            cmp.control_dynamic_snapshot, cmp.treatment_dynamic_snapshot,
            "Seed {}: Treatment dynamic snapshot must causally diverge from Control dynamic snapshot",
            seed
        );
        assert_ne!(
            cmp.control_rng_state, cmp.treatment_rng_state,
            "Seed {}: Treatment RNG state must diverge from Control RNG state",
            seed
        );
        assert!(
            cmp.treatment_known_sectors_total > 0,
            "Seed {}: Expected spatial curiosity sectors > 0 in treatment world",
            seed
        );
        assert!(
            cmp.treatment_context_entries_total > 0,
            "Seed {}: Expected water context entries > 0 in treatment world",
            seed
        );
        assert!(
            cmp.treatment_first_interval_outcomes_total > 0,
            "Seed {}: Expected refuge first interval outcomes > 0 in treatment world",
            seed
        );
        assert!(
            cmp.treatment_signals_emitted_total > 0,
            "Seed {}: Expected signals emitted > 0 in treatment world",
            seed
        );
        assert!(
            cmp.treatment_unique_signals_emitted_total > 0,
            "Seed {}: expected at least one unique signal emitted during World::tick",
            seed
        );
        assert!(
            cmp.treatment_burrows_total > 0,
            "Seed {}: Expected winter burrows dug > 0 in treatment world",
            seed
        );
        assert!(
            cmp.treatment_newly_excavated_burrows_total > 0,
            "Seed {}: expected at least one newly excavated burrow outside initial static burrows",
            seed
        );
        flag_comparison_results.push(cmp);
    }

    assert!(
        flag_comparison_results
            .iter()
            .all(|r| r.control_dynamic_snapshot != r.treatment_dynamic_snapshot),
        "All seeds must dynamically diverge between control and treatment"
    );
    assert!(
        flag_comparison_results
            .iter()
            .all(|r| r.control_rng_state != r.treatment_rng_state),
        "All seeds must produce divergent RNG trajectories between control and treatment"
    );
    assert!(
        flag_comparison_results.iter().all(|r| r.treatment_known_sectors_total > 0),
        "All seeds must record spatial curiosity known sectors when enabled"
    );
    assert!(
        flag_comparison_results.iter().all(|r| r.treatment_context_entries_total > 0),
        "All seeds must record water context entries when enabled"
    );
    assert!(
        flag_comparison_results.iter().all(|r| r.treatment_first_interval_outcomes_total > 0),
        "All seeds must record first interval refuge outcomes when enabled"
    );
    assert!(
        flag_comparison_results.iter().all(|r| r.treatment_signals_emitted_total > 0),
        "All seeds must emit and observe signals when enabled"
    );
    assert!(
        flag_comparison_results.iter().all(|r| r.treatment_unique_signals_emitted_total > 0),
        "All seeds must emit at least one unique signal during World::tick"
    );
    assert!(
        flag_comparison_results.iter().all(|r| r.treatment_burrows_total > 0),
        "All seeds must dig winter burrows when enabled"
    );
    assert!(
        flag_comparison_results.iter().all(|r| r.treatment_newly_excavated_burrows_total > 0),
        "All seeds must excavate at least one new burrow outside initial static burrows"
    );

    // Also verify deterministic reproduction & distinct trajectories across seeds
    let mut world_runs = Vec::new();
    for &seed in &seeds {
        let run = run_verified_world_seed_trajectory(seed, 80);
        world_runs.push(run);
    }
    // Verify that every distinct seed produced a genuinely distinct World & RNG trajectory
    verify_distinct_world_trajectories(&world_runs);

    // --- Part 2: Synthetic Candidate Mechanism Checks (Perturbed by Seed) ---
    let mut coord_results = Vec::new();
    let mut niche_results = Vec::new();

    for &seed in &seeds {
        // 1. Local Coordination 3-Arm Synthetic Mechanism Check (energy/stamina/threat perturbed by seed)
        let solo = run_coordination_harness_with_seed(CoordinationArm::ControlSolo, 8, seed);
        let unverified = run_coordination_harness_with_seed(CoordinationArm::UnverifiedIgnored, 8, seed);
        let verified = run_coordination_harness_with_seed(CoordinationArm::VerifiedLocalCoordination, 8, seed);

        // Assert strictly distinct arm behaviors
        assert!(!solo.focal_survived);
        assert_eq!(solo.alarms_emitted, 0);

        assert!(!unverified.focal_survived);
        assert!(unverified.alarms_emitted > 0);
        assert_eq!(unverified.alarms_verified_paid, 0);
        assert!(unverified.alarms_rejected_or_unperceived > 0);

        assert!(verified.focal_survived);
        assert!(verified.alarms_verified_paid > 0);
        assert!(verified.total_energy_spent_sender >= 10);

        coord_results.push(CoordinationArmSeedComparison {
            seed,
            solo_focal_survived: solo.focal_survived,
            solo_alarms_emitted: solo.alarms_emitted,
            unverified_focal_survived: unverified.focal_survived,
            unverified_alarms_rejected: unverified.alarms_rejected_or_unperceived,
            verified_focal_survived: verified.focal_survived,
            verified_alarms_paid: verified.alarms_verified_paid,
            verified_sender_energy_spent: verified.total_energy_spent_sender,
        });

        // 2. Ambush vs Pursuit Specialization Synthetic Mechanism Check (stamina/energy/base damage perturbed by seed)
        let genome_ambush = Genome {
            mobility: 0,
            vision: 1,
            stamina: 2,
        };
        let genome_pursuit = Genome {
            mobility: 2,
            vision: 1,
            stamina: 1,
        };

        let initial_energy = 30 + (seed % 7) as usize;
        let initial_stamina = 20 + (seed % 5) as usize;

        let mut predator_ambush = PredatorState {
            id: 101,
            pos: (5, 5),
            energy: initial_energy,
            stamina: initial_stamina,
            ambush_state: AmbushState::None,
            specialization_profile: SpecializationProfile::from_genome(&genome_ambush),
        };
        let mut predator_pursuit = PredatorState {
            id: 102,
            pos: (5, 5),
            energy: initial_energy,
            stamina: initial_stamina,
            ambush_state: AmbushState::None,
            specialization_profile: SpecializationProfile::from_genome(&genome_pursuit),
        };

        assert_eq!(
            predator_ambush.specialization_profile.specialization,
            PredatorSpecialization::AmbushSpecialist
        );
        assert_eq!(
            predator_pursuit.specialization_profile.specialization,
            PredatorSpecialization::PursuitSpecialist
        );

        // Activate ambush state adjacent to Burrow cover
        update_ambush_state(&mut predator_ambush, true, Some(CoverKind::Burrow));
        update_ambush_state(&mut predator_pursuit, true, Some(CoverKind::Burrow));

        // Base attack: damage 10 + (seed % 4), stamina cost 3 + (seed % 2)
        let base_damage = 10 + (seed % 4) as usize;
        let base_stamina_cost = 3 + (seed % 2) as usize;
        let atk_ambush = resolve_attack(&mut predator_ambush, base_damage, base_stamina_cost).unwrap();
        let atk_pursuit = resolve_attack(&mut predator_pursuit, base_damage, base_stamina_cost).unwrap();

        // 3. Corpse Scavenging Niche Differentiation Synthetic Mechanism Check (initial energy & feeding perturbed by seed)
        let initial_corpse_energy = 100 + (seed as usize % 30);
        let mut corpse = DecayingCorpse::new(seed as usize, Position::new(0, 0), initial_corpse_energy, 10);

        // Decay corpse until Putrid stage (< 30% remaining)
        while corpse.freshness_stage() != FreshnessStage::Putrid {
            corpse.decay_tick();
        }

        let feed_amount = 10 + (seed as usize % 5);
        // Test Omnivore feeding (scavenger)
        let omnivore_meal = corpse.feed(Species::Omnivore, feed_amount).unwrap();

        // Test Carnivore feeding (predator)
        let carnivore_meal = corpse.feed(Species::Carnivore, feed_amount).unwrap();

        let audit = corpse.audit();
        assert!(audit.is_balanced());

        niche_results.push(NicheSpecializationSeedComparison {
            seed,
            ambush_first_strike_damage: atk_ambush.total_damage,
            pursuit_first_strike_damage: atk_pursuit.total_damage,
            ambush_stealth_reduction: predator_ambush.specialization_profile.stealth_reduction,
            pursuit_stealth_reduction: predator_pursuit.specialization_profile.stealth_reduction,
            omnivore_putrid_health_penalty: omnivore_meal.health_penalty,
            carnivore_putrid_health_penalty: carnivore_meal.health_penalty,
            corpse_stock_flow_balanced: audit.is_balanced(),
        });
    }

    // Verify all seeds in Coordination & Niche Specializations
    for res in &coord_results {
        assert!(!res.solo_focal_survived);
        assert!(!res.unverified_focal_survived);
        assert!(res.verified_focal_survived);
        assert!(res.verified_sender_energy_spent >= 10);
    }

    // Verify that coordination metrics across distinct seeds actually differ (perturbed by seed)
    let any_distinct_energy = coord_results
        .windows(2)
        .any(|w| w[0].verified_sender_energy_spent != w[1].verified_sender_energy_spent);
    assert!(
        any_distinct_energy,
        "Seed perturbation must produce distinct sender energy values across seeds"
    );

    for res in &niche_results {
        // Ambush specialist deals higher burst damage from ambush (+4 vs +1 bonus)
        assert!(res.ambush_first_strike_damage > res.pursuit_first_strike_damage);
        // Ambush specialist provides higher stealth reduction (3 vs 1)
        assert!(res.ambush_stealth_reduction > res.pursuit_stealth_reduction);
        // Omnivore suffers zero health penalty on Putrid carrion, whereas Carnivore suffers 6
        assert_eq!(res.omnivore_putrid_health_penalty, 0);
        assert_eq!(res.carnivore_putrid_health_penalty, 6);
        // Stock-flow conservation strictly holds
        assert!(res.corpse_stock_flow_balanced);
    }

    println!("World Runs (Seeds 201-205): {:?}", world_runs);
    println!("Coordination Results: {:?}", coord_results);
    println!("Niche Specialization Results: {:?}", niche_results);
}
