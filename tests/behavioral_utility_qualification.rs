use curious::{
    creatures::{
        carnivore::CarnivoreEntity, herbivore::HerbivoreEntity, omnivore::OmnivoreEntity,
    },
    entity::{controller::EntityController, phase::EntityPhase, species::Species},
    environment::{DayNightCycle, Environment, SeasonCycle},
    map::{cell::Cell, movement::Position},
    persistence::State,
    world::{EntitySlot, World},
};
use std::path::Path;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QualificationArm {
    ControlFlagsOff,
    WaterContextOnly,
    RefugeFirstIntervalOnly,
    SpatialCuriosityOnly,
    InjuryCapacityPenaltyOnly,
    JuvenileParentalCareOnly,
    AmbushPredationOnly,
    LocalCoordinationOnly,
    DayNightCycleOnly,
    SeasonAndBurrowsOnly,
    QualifiedCandidateBundle,
    AllV19Features,
}

impl QualificationArm {
    pub fn name(self) -> &'static str {
        match self {
            Self::ControlFlagsOff => "ControlFlagsOff",
            Self::WaterContextOnly => "WaterContextOnly",
            Self::RefugeFirstIntervalOnly => "RefugeFirstIntervalOnly",
            Self::SpatialCuriosityOnly => "SpatialCuriosityOnly",
            Self::InjuryCapacityPenaltyOnly => "InjuryCapacityPenaltyOnly",
            Self::JuvenileParentalCareOnly => "JuvenileParentalCareOnly",
            Self::AmbushPredationOnly => "AmbushPredationOnly",
            Self::LocalCoordinationOnly => "LocalCoordinationOnly",
            Self::DayNightCycleOnly => "DayNightCycleOnly",
            Self::SeasonAndBurrowsOnly => "SeasonAndBurrowsOnly",
            Self::QualifiedCandidateBundle => "QualifiedCandidateBundle",
            Self::AllV19Features => "AllV19Features",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualificationVerdict {
    /// Strict Pareto / no-survival-deficit and positive or neutral physical vector: candidate for controlled live promotion.
    PassLiveCandidate,
    /// Inherently trade-off, environmental stressor, or predator-skewed pressure: keep default-off in live coexistence.
    TradeOffKeepDefaultOff,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PhysicalOutcomeVector {
    pub survived_entity_ticks: usize,
    pub prey_survived_ticks: usize,
    pub final_alive_count: usize,
    pub total_health_end: usize,
    pub total_energy_end: usize,
    pub total_water_end: usize,
    pub total_damage_taken: usize,
    pub mechanism_activations: usize,
}

fn build_qualification_entities(seed: u64) -> Vec<EntitySlot> {
    let ox = (seed % 3) as isize - 1;
    let oy = ((seed / 3) % 3) as isize - 1;

    let mut slots = vec![
        // 1: Adult Herbivore parent near Juvenile 2 and refuge burrows
        EntitySlot::new(
            1,
            Position::new(-6 + ox, -6 + oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        ),
        // 2: Juvenile Herbivore (age 10 < maturity, low energy, adjacent to parent 1 and threatened by Carnivore 3)
        EntitySlot::new(
            2,
            Position::new(-6 + ox, -5 + oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        ),
        // 3: Carnivore threatening Herbivore 1 & Juvenile 2
        EntitySlot::new(
            3,
            Position::new(-4 + ox, -5 + oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(CarnivoreEntity::default()),
        ),
        // 4: Adult Herbivore roamer/excavator
        EntitySlot::new(
            4,
            Position::new(6 + ox, -6 + oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        ),
        // 5: Wounded Herbivore (tests injury_capacity_penalty vs normal locomotion)
        EntitySlot::new(
            5,
            Position::new(-6 + ox, 6 + oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        ),
        // 6: Thirsty Omnivore (tests water_residual + context_ttl learning)
        EntitySlot::new(
            6,
            Position::new(ox, oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(OmnivoreEntity::default()),
        ),
        // 7: Second Omnivore Explorer
        EntitySlot::new(
            7,
            Position::new(3 + ox, 3 + oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(OmnivoreEntity::default()),
        ),
        // 8: Second Carnivore near cover
        EntitySlot::new(
            8,
            Position::new(-4 + ox, 5 + oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(CarnivoreEntity::default()),
        ),
        // 9: Adult Herbivore in northern sector
        EntitySlot::new(
            9,
            Position::new(5 + ox, 6 + oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        ),
        // 10: Juvenile Herbivore near Adult 9
        EntitySlot::new(
            10,
            Position::new(5 + ox, 5 + oy),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        ),
    ];

    // Configure adult parent 1 & juvenile 2
    slots[0].base.life_mut().age = 120;
    slots[0].base.life_mut().health = 100;
    slots[0].base.life_mut().energy = 90;
    slots[0].base.life_mut().water = 85;
    slots[0].base.life_mut().stamina = 24;

    slots[1].base.life_mut().age = 8; // juvenile
    slots[1].parents = vec![1];
    slots[1].base.life_mut().health = 65;
    slots[1].base.life_mut().energy = 32;
    slots[1].base.life_mut().water = 75;
    slots[1].base.life_mut().stamina = 16;

    // Carnivore 3
    slots[2].base.life_mut().age = 100;
    slots[2].base.life_mut().health = 100;
    slots[2].base.life_mut().energy = 75;
    slots[2].base.life_mut().water = 80;
    slots[2].base.life_mut().stamina = 20;

    // Herbivore 4 (high energy/stamina)
    slots[3].base.life_mut().age = 110;
    slots[3].base.life_mut().energy = 95;
    slots[3].base.life_mut().water = 85;
    slots[3].base.life_mut().stamina = 30;

    // Wounded Herbivore 5 (health < 50% triggers injury capacity penalty when enabled)
    slots[4].base.life_mut().age = 90;
    slots[4].base.life_mut().health = 38;
    slots[4].base.life_mut().energy = 60;
    slots[4].base.life_mut().water = 60;
    slots[4].base.life_mut().stamina = 14;

    // Thirsty Omnivore 6
    slots[5].base.life_mut().age = 100;
    slots[5].base.life_mut().water = 18;
    slots[5].base.life_mut().energy = 85;

    // Explorer Omnivore 7
    slots[6].base.life_mut().age = 100;
    slots[6].base.life_mut().water = 65;
    slots[6].base.life_mut().energy = 80;

    // Carnivore 8 near cover
    slots[7].base.life_mut().age = 100;
    slots[7].base.life_mut().energy = 70;
    slots[7].base.life_mut().water = 80;
    slots[7].base.life_mut().stamina = 20;

    // Adult parent 9 & juvenile 10
    slots[8].base.life_mut().age = 130;
    slots[8].base.life_mut().energy = 92;
    slots[8].base.life_mut().water = 85;
    slots[8].base.life_mut().stamina = 24;

    slots[9].base.life_mut().age = 6; // juvenile
    slots[9].parents = vec![9];
    slots[9].base.life_mut().health = 70;
    slots[9].base.life_mut().energy = 28;
    slots[9].base.life_mut().water = 80;
    slots[9].base.life_mut().stamina = 14;

    slots
}

fn configure_qualification_arm(world: &mut World, arm: QualificationArm) {
    match arm {
        QualificationArm::ControlFlagsOff => {}
        QualificationArm::WaterContextOnly => {
            world.policy.water_residual_enabled = true;
            for slot in &mut world.entities {
                slot.learner.context_ttl_enabled = true;
            }
        }
        QualificationArm::RefugeFirstIntervalOnly => {
            for slot in &mut world.entities {
                slot.memory.first_interval_refuge_enabled = true;
                slot.memory.refuge_outcomes_enabled = true;
            }
        }
        QualificationArm::SpatialCuriosityOnly => {
            for slot in &mut world.entities {
                slot.memory.spatial_curiosity_enabled = true;
            }
        }
        QualificationArm::InjuryCapacityPenaltyOnly => {
            world.policy.injury_capacity_penalty = true;
        }
        QualificationArm::JuvenileParentalCareOnly => {
            world.policy.juvenile_care_enabled = true;
        }
        QualificationArm::AmbushPredationOnly => {
            world.policy.ambush_predation_enabled = true;
        }
        QualificationArm::LocalCoordinationOnly => {
            world.policy.local_coordination_enabled = true;
            world.environment.signals_enabled = true;
        }
        QualificationArm::DayNightCycleOnly => {
            world.environment.day_night = Some(DayNightCycle {
                period: 20,
                day_ticks: 12,
                night_vision_penalty: 1,
            });
        }
        QualificationArm::SeasonAndBurrowsOnly => {
            world.environment.season_cycle = Some(SeasonCycle {
                period: 20,
                winter_bmr_surcharge: 2,
                winter_plant_regen_divisor: 2,
            });
        }
        QualificationArm::QualifiedCandidateBundle => {
            // Only the empirically Pareto-qualified features that pass the strict no-survival-deficit gate:
            // 1. JuvenileParentalCareOnly (strictly dominates ControlFlagsOff across all survival & vital stock metrics)
            // 2. RefugeFirstIntervalOnly (zero survival deficit, bounded per-owner first-interval attribution)
            world.policy.juvenile_care_enabled = true;
            for slot in &mut world.entities {
                slot.memory.first_interval_refuge_enabled = true;
                slot.memory.refuge_outcomes_enabled = true;
            }
        }
        QualificationArm::AllV19Features => {
            world.policy.water_residual_enabled = true;
            world.policy.injury_capacity_penalty = true;
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

fn build_qualification_world(seed: u64, arm: QualificationArm) -> (World, usize) {
    let ox = (seed % 3) as isize - 1;
    let oy = ((seed / 3) % 3) as isize - 1;

    let mut world = World::with_seed(
        -12,
        12,
        -12,
        12,
        build_qualification_entities(seed),
        seed,
    );
    world.logger = curious::logger::Logger::disabled();

    // Water and food around Omnivore 6 at (ox, oy)
    world
        .map
        .set_cell(Position::new(ox + 1, oy), Cell::Water { amount: 60 });
    world
        .map
        .set_cell(Position::new(ox, oy + 1), Cell::Water { amount: 60 });
    world
        .map
        .set_cell(Position::new(ox, oy - 1), Cell::Food { amount: 60 });

    // Refuge burrows near Herbivore 1 & Juvenile 2
    world
        .map
        .set_cell(Position::new(-7 + ox, -6 + oy), Cell::Burrow);
    world
        .map
        .set_cell(Position::new(-6 + ox, -7 + oy), Cell::Burrow);

    // Ambush cover near Carnivore 8 and Carnivore 3
    world
        .map
        .set_cell(Position::new(-4 + ox, 4 + oy), Cell::Wall);
    world
        .map
        .set_cell(Position::new(-3 + ox, -5 + oy), Cell::Wall);

    // Clear empty digging space around Herbivore 4 at (6 + ox, -6 + oy)
    for dx in -2..=2 {
        for dy in -2..=2 {
            world
                .map
                .set_cell(Position::new(6 + ox + dx, -6 + oy + dy), Cell::Empty);
        }
    }

    // Distributed food and water patches across sectors for sustained 150-tick run
    for x in (-10..=10).step_by(4) {
        for y in (-10..=10).step_by(4) {
            let p = Position::new(x, y);
            if world.map.cell(p) == Some(&Cell::Empty) {
                if (x + y).rem_euclid(8) == 0 {
                    world.map.set_cell(p, Cell::Water { amount: 50 });
                } else {
                    world.map.set_cell(p, Cell::Food { amount: 45 });
                }
            }
        }
    }

    world.environment = Environment::from_map(&world.map);
    let initial_burrows = world.environment.burrows.len();
    configure_qualification_arm(&mut world, arm);
    (world, initial_burrows)
}

fn run_qualification_simulation(seed: u64, arm: QualificationArm, ticks: usize) -> PhysicalOutcomeVector {
    let (mut world, initial_burrows) = build_qualification_world(seed, arm);
    let mut vec_out = PhysicalOutcomeVector::default();
    let mut max_burrows = initial_burrows;
    let mut max_context = 0usize;
    let mut max_first_interval = 0usize;
    let mut max_sectors = 0usize;

    for _ in 0..ticks {
        let prev_health: std::collections::HashMap<usize, usize> = world
            .entities
            .iter()
            .filter(|s| matches!(s.phase, EntityPhase::Active) && s.base.life().health > 0)
            .map(|s| (s.id, s.base.life().health))
            .collect();

        world.tick();

        for slot in &world.entities {
            if matches!(slot.phase, EntityPhase::Active) && slot.base.life().health > 0 {
                vec_out.survived_entity_ticks += 1;
                if slot.base.species() != Species::Carnivore {
                    vec_out.prey_survived_ticks += 1;
                }
            }
            if let Some(&old_h) = prev_health.get(&slot.id) {
                let new_h = if matches!(slot.phase, EntityPhase::Active) {
                    slot.base.life().health
                } else {
                    0
                };
                if old_h > new_h {
                    vec_out.total_damage_taken += old_h - new_h;
                }
            }
        }

        max_burrows = max_burrows.max(world.environment.burrows.len());
        max_context = max_context.max(
            world
                .entities
                .iter()
                .map(|s| s.learner.context_entries.len())
                .sum(),
        );
        max_first_interval = max_first_interval.max(
            world
                .entities
                .iter()
                .map(|s| s.memory.first_interval_outcomes.len())
                .sum(),
        );
        max_sectors = max_sectors.max(
            world
                .entities
                .iter()
                .map(|s| s.memory.known_sectors.len())
                .sum(),
        );
        vec_out.mechanism_activations += world
            .environment
            .signals
            .iter()
            .filter(|s| s.emitted_tick == world.tick_counter)
            .count();
    }

    for slot in &world.entities {
        if matches!(slot.phase, EntityPhase::Active) && slot.base.life().health > 0 {
            vec_out.final_alive_count += 1;
            vec_out.total_health_end += slot.base.life().health;
            vec_out.total_energy_end += slot.base.life().energy;
            vec_out.total_water_end += slot.base.life().water;
        }
    }

    vec_out.mechanism_activations += max_context
        + max_first_interval
        + max_sectors
        + max_burrows.saturating_sub(initial_burrows);

    vec_out
}

pub fn classify_feature_arm(
    _arm: QualificationArm,
    control: &PhysicalOutcomeVector,
    treatment: &PhysicalOutcomeVector,
) -> QualificationVerdict {
    // Strict no-survival-deficit Pareto gate on aggregate 10-seed physical outcome vector:
    // 1. Must not reduce survived_entity_ticks, prey_survived_ticks, or final_alive_count
    // 2. Must not reduce end-of-horizon vital stocks (total_health_end, total_energy_end, total_water_end)
    // 3. Damage taken per survived entity-tick must not exceed ControlFlagsOff
    let no_survival_deficit = treatment.survived_entity_ticks >= control.survived_entity_ticks
        && treatment.prey_survived_ticks >= control.prey_survived_ticks
        && treatment.final_alive_count >= control.final_alive_count;
    let no_vital_collapse = treatment.total_health_end >= control.total_health_end
        && treatment.total_energy_end >= control.total_energy_end
        && treatment.total_water_end >= control.total_water_end;
    let dmg_per_tick_ok = (treatment.total_damage_taken as u128)
        * (control.survived_entity_ticks.max(1) as u128)
        <= (control.total_damage_taken as u128) * (treatment.survived_entity_ticks.max(1) as u128);

    if no_survival_deficit && no_vital_collapse && dmg_per_tick_ok {
        QualificationVerdict::PassLiveCandidate
    } else {
        QualificationVerdict::TradeOffKeepDefaultOff
    }
}

#[test]
fn test_ten_seed_150_tick_behavioral_utility_and_pareto_qualification() {
    let seeds: Vec<u64> = (401..=410).collect();
    let ticks = 150usize;

    let arms = [
        QualificationArm::ControlFlagsOff,
        QualificationArm::WaterContextOnly,
        QualificationArm::RefugeFirstIntervalOnly,
        QualificationArm::SpatialCuriosityOnly,
        QualificationArm::InjuryCapacityPenaltyOnly,
        QualificationArm::JuvenileParentalCareOnly,
        QualificationArm::AmbushPredationOnly,
        QualificationArm::LocalCoordinationOnly,
        QualificationArm::DayNightCycleOnly,
        QualificationArm::SeasonAndBurrowsOnly,
        QualificationArm::QualifiedCandidateBundle,
        QualificationArm::AllV19Features,
    ];

    let mut totals: std::collections::HashMap<QualificationArm, PhysicalOutcomeVector> =
        std::collections::HashMap::new();

    for &arm in &arms {
        let mut sum = PhysicalOutcomeVector::default();
        for &seed in &seeds {
            let out = run_qualification_simulation(seed, arm, ticks);
            sum.survived_entity_ticks += out.survived_entity_ticks;
            sum.prey_survived_ticks += out.prey_survived_ticks;
            sum.final_alive_count += out.final_alive_count;
            sum.total_health_end += out.total_health_end;
            sum.total_energy_end += out.total_energy_end;
            sum.total_water_end += out.total_water_end;
            sum.total_damage_taken += out.total_damage_taken;
            sum.mechanism_activations += out.mechanism_activations;
        }
        totals.insert(arm, sum);
    }

    let control = totals.get(&QualificationArm::ControlFlagsOff).unwrap().clone();
    println!("\n=== WAVE 11 BEHAVIORAL UTILITY & PARETO TABLE (10 SEEDS 401..=410, 150 TICKS) ===");
    for &arm in &arms {
        let v = totals.get(&arm).unwrap();
        let verdict = classify_feature_arm(arm, &control, v);
        println!(
            "{:<26} | survived_ticks={:<5} | prey_ticks={:<5} | alive_end={:<3} | health_end={:<4} | energy_end={:<4} | water_end={:<4} | dmg_taken={:<4} | act={:<4} | verdict={:?}",
            arm.name(),
            v.survived_entity_ticks,
            v.prey_survived_ticks,
            v.final_alive_count,
            v.total_health_end,
            v.total_energy_end,
            v.total_water_end,
            v.total_damage_taken,
            v.mechanism_activations,
            verdict
        );
    }

    let qualified_bundle = totals
        .get(&QualificationArm::QualifiedCandidateBundle)
        .unwrap();
    let all_v19 = totals.get(&QualificationArm::AllV19Features).unwrap();

    // 1. QualifiedCandidateBundle must strictly preserve or improve survival and energy over AllV19Features
    // (which includes winter thermal surcharge, winter food scarcity, and injury capacity penalties).
    assert!(
        qualified_bundle.survived_entity_ticks >= control.survived_entity_ticks,
        "QualifiedCandidateBundle must not cause any survival deficit vs ControlFlagsOff"
    );
    assert!(
        qualified_bundle.total_energy_end > all_v19.total_energy_end,
        "QualifiedCandidateBundle ({}) must preserve significantly more total energy than AllV19Features ({})",
        qualified_bundle.total_energy_end,
        all_v19.total_energy_end
    );
    assert!(
        qualified_bundle.survived_entity_ticks >= all_v19.survived_entity_ticks,
        "QualifiedCandidateBundle ({}) must achieve >= survived_entity_ticks than AllV19Features ({})",
        qualified_bundle.survived_entity_ticks,
        all_v19.survived_entity_ticks
    );
}

fn load_live_v12_state_json() -> Option<String> {
    let tmp_json = Path::new("/tmp/curious-shadow-eval/live_v12_latest_state.json");
    if tmp_json.exists() {
        if let Ok(s) = std::fs::read_to_string(tmp_json) {
            return Some(s);
        }
    }
    let archive_path = Path::new("/root/curious/data/coexistence/history-000870.sqlite.zst");
    if archive_path.exists() {
        let temp_dir = tempfile::tempdir().ok()?;
        let sqlite_path = temp_dir.path().join("shadow-history-000870.sqlite");
        let input = std::fs::File::open(archive_path).ok()?;
        let mut decoder = zstd::stream::read::Decoder::new(input).ok()?;
        let mut output = std::fs::File::create(&sqlite_path).ok()?;
        std::io::copy(&mut decoder, &mut output).ok()?;
        drop(output);
        let conn = rusqlite::Connection::open_with_flags(
            &sqlite_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .ok()?;
        let state_json: String = conn
            .query_row(
                "SELECT state_json FROM records ORDER BY id DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .ok()?;
        return Some(state_json);
    }
    None
}

#[test]
fn test_live_v12_shadow_restore_and_20_tick_continuation() {
    let Some(raw_json) = load_live_v12_state_json() else {
        println!("Skipping live V12 shadow file check (archive not present on this host)");
        return;
    };

    let state_v12: State =
        serde_json::from_str(&raw_json).expect("Live V12 JSON must deserialize cleanly");
    assert_eq!(state_v12.version, 12);
    let initial_tick = state_v12.tick;
    let initial_entities = state_v12.entities.len();
    assert!(
        initial_entities >= 600,
        "Expected 600+ entities in real V12 state, got {}",
        initial_entities
    );

    // Run 1: ControlFlagsOff (20 ticks from real live V12 checkpoint)
    let mut control_world = state_v12
        .clone()
        .restore()
        .expect("V12 state must restore into V19 engine without error");
    let start_ctrl = Instant::now();
    for _ in 0..20 {
        control_world.tick();
    }
    let ctrl_elapsed_us = start_ctrl.elapsed().as_micros() as u64;
    let ctrl_avg_tick_us = ctrl_elapsed_us / 20;

    let ctrl_captured = State::capture(&control_world);
    assert_eq!(ctrl_captured.version, 19);
    assert_eq!(ctrl_captured.tick, initial_tick + 20);
    let ctrl_json = serde_json::to_string(&ctrl_captured).unwrap();
    let _ = ctrl_captured
        .clone()
        .restore()
        .expect("Captured V19 state after ControlFlagsOff shadow run must validate & restore");

    let ctrl_herbivores = control_world
        .entities
        .iter()
        .filter(|s| {
            matches!(s.phase, EntityPhase::Active)
                && s.base.life().health > 0
                && s.base.species() == Species::Herbivore
        })
        .count();
    let ctrl_carnivores = control_world
        .entities
        .iter()
        .filter(|s| {
            matches!(s.phase, EntityPhase::Active)
                && s.base.life().health > 0
                && s.base.species() == Species::Carnivore
        })
        .count();

    // Run 2: QualifiedCandidateBundle (20 ticks from real live V12 checkpoint)
    let mut cand_world = state_v12
        .restore()
        .expect("V12 state must restore into V19 engine without error");
    configure_qualification_arm(&mut cand_world, QualificationArm::QualifiedCandidateBundle);
    let start_cand = Instant::now();
    let mut cand_signals_emitted = 0usize;
    for _ in 0..20 {
        cand_world.tick();
        cand_signals_emitted += cand_world
            .environment
            .signals
            .iter()
            .filter(|s| s.emitted_tick == cand_world.tick_counter)
            .count();
    }
    let cand_elapsed_us = start_cand.elapsed().as_micros() as u64;
    let cand_avg_tick_us = cand_elapsed_us / 20;

    let cand_captured = State::capture(&cand_world);
    assert_eq!(cand_captured.version, 19);
    assert_eq!(cand_captured.tick, initial_tick + 20);
    let cand_json = serde_json::to_string(&cand_captured).unwrap();
    let _ = cand_captured
        .clone()
        .restore()
        .expect("Captured V19 state after QualifiedCandidateBundle shadow run must validate & restore");

    let cand_herbivores = cand_world
        .entities
        .iter()
        .filter(|s| {
            matches!(s.phase, EntityPhase::Active)
                && s.base.life().health > 0
                && s.base.species() == Species::Herbivore
        })
        .count();
    let cand_carnivores = cand_world
        .entities
        .iter()
        .filter(|s| {
            matches!(s.phase, EntityPhase::Active)
                && s.base.life().health > 0
                && s.base.species() == Species::Carnivore
        })
        .count();
    let cand_known_sectors: usize = cand_world
        .entities
        .iter()
        .map(|s| s.memory.known_sectors.len())
        .sum();
    let cand_context_entries: usize = cand_world
        .entities
        .iter()
        .map(|s| s.learner.context_entries.len())
        .sum();
    let cand_first_interval: usize = cand_world
        .entities
        .iter()
        .map(|s| s.memory.first_interval_outcomes.len())
        .sum();

    println!(
        "\n=== LIVE V12 SHADOW CONTINUATION (START TICK={}, ENTITIES={}) ===",
        initial_tick, initial_entities
    );
    println!(
        "ControlFlagsOff (20 ticks): herb={} carn={} avg_tick_us={} json_bytes={}",
        ctrl_herbivores,
        ctrl_carnivores,
        ctrl_avg_tick_us,
        ctrl_json.len()
    );
    println!(
        "QualifiedCandidateBundle (20 ticks): herb={} carn={} avg_tick_us={} json_bytes={} sectors={} water_ctx={} refuge_fi={} signals={}",
        cand_herbivores,
        cand_carnivores,
        cand_avg_tick_us,
        cand_json.len(),
        cand_known_sectors,
        cand_context_entries,
        cand_first_interval,
        cand_signals_emitted
    );

    assert!(ctrl_herbivores > 400 && ctrl_carnivores > 50);
    assert!(cand_herbivores >= ctrl_herbivores && cand_carnivores >= ctrl_carnivores);
    assert!(cand_first_interval > 0);
    assert!(cand_json.len() < 4 * 1024 * 1024);
}

#[test]
fn test_offspring_v19_capability_flag_propagation_with_zero_memory_inheritance() {
    use curious::entity::memory::KnownSector;
    use curious::entity::refuge_learning::{RefugeKey, record_first_interval_outcome};

    let mut p1 = EntitySlot::new(
        1,
        Position::new(0, 0),
        EntityPhase::Active,
        EntityController::AI,
        Box::new(HerbivoreEntity::default()),
    );
    let mut p2 = EntitySlot::new(
        2,
        Position::new(1, 0),
        EntityPhase::Active,
        EntityController::AI,
        Box::new(HerbivoreEntity::default()),
    );

    for p in [&mut p1, &mut p2] {
        p.base.life_mut().age = 100;
        p.base.life_mut().health = p.base.life().max_health;
        p.base.life_mut().energy = p.base.life().max_energy;
        p.base.life_mut().water = p.base.life().max_water;
        p.base.life_mut().stamina = p.base.life().max_stamina;
        p.memory.refuge_outcomes_enabled = true;
        p.memory.first_interval_refuge_enabled = true;
        p.memory.spatial_curiosity_enabled = true;
        p.learner.context_ttl_enabled = true;

        record_first_interval_outcome(
            &mut p.memory.first_interval_outcomes,
            RefugeKey {
                threat_id: 99,
                threat_species: Species::Carnivore,
                bearing: 0,
                threat_distance: 1,
                energy_quartile: 2,
                stamina_quartile: 2,
                refuge_options: vec![Position::new(2, 2)],
                endpoint: Position::new(2, 2),
            },
            Some(0),
            0,
        );
        p.memory.known_sectors.push(KnownSector {
            sector_x: 0,
            sector_y: 0,
            visit_count: 1,
            last_visited_tick: 0,
        });
        p.learner
            .record_context_observation(Position::new(3, 3), 2, 1, 0, 45);
    }

    let mut world = World::with_seed(-5, 5, -5, 5, vec![p1, p2], 777);
    for y in -5..=5 {
        for x in -5..=5 {
            world.map.set_cell(Position::new(x, y), Cell::Empty);
        }
    }
    world.environment = Environment::from_map(&world.map);
    world.tick();

    assert_eq!(world.entities.len(), 3, "Offspring must be born on tick 1");
    let child = &world.entities[2];
    assert_eq!(child.id, 3);
    assert_eq!(child.generation, 1);

    // 1. V19 capability flags propagate via && when both parents have them enabled:
    assert!(child.memory.refuge_outcomes_enabled);
    assert!(child.memory.first_interval_refuge_enabled);
    assert!(child.memory.spatial_curiosity_enabled);
    assert!(child.learner.context_ttl_enabled);

    // 2. Strict zero learned memory inheritance: child tables must be completely empty at birth!
    assert!(child.memory.first_interval_outcomes.is_empty());
    assert!(child.memory.known_sectors.is_empty());
    assert!(child.learner.context_entries.is_empty());
    assert!(child.memory.refuge_outcomes.is_empty());
}

#[test]
#[ignore]
fn run_live_v12_shadow_1000_tick_soak_four_arms() {
    let raw_json = load_live_v12_state_json().expect("Live V12 archive must be readable");
    let state_v12: State = serde_json::from_str(&raw_json).expect("Valid V12 JSON");
    let initial_next_id = state_v12.next_entity_id;

    let arms = [
        QualificationArm::ControlFlagsOff,
        QualificationArm::JuvenileParentalCareOnly,
        QualificationArm::RefugeFirstIntervalOnly,
        QualificationArm::QualifiedCandidateBundle,
    ];

    for &arm in &arms {
        let mut world = state_v12.clone().restore().expect("Restore V12");
        configure_qualification_arm(&mut world, arm);

        println!("\n================================================================================");
        println!("ARM: {}", arm.name());
        println!("Tick  | Herb | Carn | Alive | Births | MaxGen | FI_Rec | Offspring_FI | Mean_H | Mean_E | Mean_W | JSON_Bytes | ms/tick");
        println!("--------------------------------------------------------------------------------");

        let mut cumulative_us = 0u128;
        for step in 0..=1000 {
            if step % 100 == 0 {
                let mut herb = 0usize;
                let mut carn = 0usize;
                let mut max_gen = 0usize;
                let mut sum_h = 0usize;
                let mut sum_e = 0usize;
                let mut sum_w = 0usize;
                let mut fi_rec = 0usize;
                let mut offspring_fi = 0usize;

                for s in &world.entities {
                    if matches!(s.phase, EntityPhase::Active) && s.base.life().health > 0 {
                        match s.base.species() {
                            Species::Herbivore => herb += 1,
                            Species::Carnivore => carn += 1,
                            Species::Omnivore => {}
                        }
                        max_gen = max_gen.max(s.generation);
                        sum_h += s.base.life().health;
                        sum_e += s.base.life().energy;
                        sum_w += s.base.life().water;
                        let fi_len = s.memory.first_interval_outcomes.len();
                        fi_rec += fi_len;
                        if s.id >= initial_next_id {
                            offspring_fi += fi_len;
                        }
                    }
                }
                let alive = herb + carn;
                let births = world.next_entity_id.saturating_sub(initial_next_id);
                let mean_h = if alive > 0 { sum_h as f64 / alive as f64 } else { 0.0 };
                let mean_e = if alive > 0 { sum_e as f64 / alive as f64 } else { 0.0 };
                let mean_w = if alive > 0 { sum_w as f64 / alive as f64 } else { 0.0 };
                let captured = State::capture(&world);
                let json_bytes = serde_json::to_string(&captured).unwrap().len();
                if step == 1000 {
                    let _ = captured.restore().expect("1000-tick state must validate & restore");
                }
                let ms_per_tick = if step > 0 {
                    (cumulative_us as f64 / 1000.0) / (step as f64)
                } else {
                    0.0
                };
                println!(
                    "{:<5} | {:<4} | {:<4} | {:<5} | {:<6} | {:<6} | {:<6} | {:<12} | {:<6.1} | {:<6.1} | {:<6.1} | {:<10} | {:.2}",
                    world.tick_counter,
                    herb,
                    carn,
                    alive,
                    births,
                    max_gen,
                    fi_rec,
                    offspring_fi,
                    mean_h,
                    mean_e,
                    mean_w,
                    json_bytes,
                    ms_per_tick
                );
            }
            if step < 1000 {
                let t0 = Instant::now();
                world.tick();
                cumulative_us += t0.elapsed().as_micros();
            }
        }
    }
}
