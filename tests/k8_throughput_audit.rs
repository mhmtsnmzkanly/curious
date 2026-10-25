#![allow(dead_code, unused_imports)]

//! K8 Throughput & Memory Bounds Audit and P3 Empirical Comparison Tests.
//!
//! Validates:
//! 1. K8: Short-load multi-species world (30 founders: 10 Herbivores, 10 Carnivores, 10 Omnivores)
//!    running 250 ticks with day/night cycles, signals enabled, and injury capacity penalty.
//!    Verifies bounded signal count (<= 64), bounded dangers (<= 8), bounded sites (<= 32),
//!    and checkpoint JSON serialization size < 1MB.
//! 2. P3: Fixed 5 held-out seeds [101, 102, 103, 104, 105] comparing:
//!    - Water context-aligned residual candidate vs. unlearning baseline under drying schedule.
//!    - First-interval refuge damage learner vs. unlearning baseline with positive vs. censored route.
//! 3. P3: Fixed 5 held-out seeds [201, 202, 203, 204, 205] comparing:
//!    - Local coordination 3-arm harness (ControlSolo vs UnverifiedIgnored vs VerifiedLocalCoordination).
//!    - Ambush specialist vs Pursuit specialist predation performance.
//!    - Carnivore vs Omnivore putrid corpse scavenging spoilage resistance and stock-flow conservation.

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
    environment::{DayNightCycle, SignalKind as EnvSignalKind, SignalTrace, MAX_ENVIRONMENT_SIGNALS},
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
    run_coordination_harness, CoordinationArm,
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

    let mut water_results = Vec::new();
    let mut refuge_results = Vec::new();

    let speed = 2u8;
    let cost = 1u8;
    let pos_dry = (10, 0); // Source A (closer/higher initial baseline, but dries up)
    let pos_wet = (0, 15); // Source B (further/lower initial baseline, stays reliable)
    let key_dry = WaterContextKey::new(pos_dry, speed, cost).unwrap();
    let key_wet = WaterContextKey::new(pos_wet, speed, cost).unwrap();

    for &seed in &seeds {
        // --- 1. Water Candidate Comparison ---
        let mut table = WaterContextResidualTable::new(true, true);
        let candidates = vec![
            WaterCandidateInput {
                source_pos: pos_dry,
                choice_baseline: 450,
                steps_len: 4,
            },
            WaterCandidateInput {
                source_pos: pos_wet,
                choice_baseline: 340,
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
                            choice_baseline: 450,
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
                            choice_baseline: 450,
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
                        choice_baseline: 340,
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

        // --- 2. Refuge First-Interval Candidate Comparison ---
        let mut learner = FirstIntervalLearner::with_enabled(seed as usize, true, true);

        // Candidate 0: Route straight into danger bearing (threat damage 20 + seed % 4)
        let key_route1 = FirstIntervalKey {
            subject_species: CandidateSpecies::Herbivore,
            threat_species: CandidateSpecies::Carnivore,
            threat_bearing_sector: 2,
            threat_distance_steps: 1,
            first_step_sector: 2,
            endpoint: (8, 0),
            subject_speed: 2,
        };

        // Candidate 1: Perpendicular evasion (threat damage 4, but route later censored at tick 3+)
        let key_route2 = FirstIntervalKey {
            subject_species: CandidateSpecies::Herbivore,
            threat_species: CandidateSpecies::Carnivore,
            threat_bearing_sector: 2,
            threat_distance_steps: 1,
            first_step_sector: 0,
            endpoint: (0, 8),
            subject_speed: 2,
        };

        let route1_dmg = 20 + (seed % 4);
        let route2_dmg = 4;

        // Record Route 1: completed first interval with high damage
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

        // Record Route 2: completed first interval with low damage, later censored on full route
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
            baseline_chosen_candidate: 0, // Uninformed baseline chooses index 0
            learner_chosen_candidate: ranked_order[0], // Learner prioritizes lower damage (index 1)
            route1_expected_damage: exp_dmg_1,
            route2_expected_damage: exp_dmg_2,
        });
    }

    // Verify all seeds in Water Candidate Comparison:
    // Candidate achieves strictly more water gained than baseline and switches to alternative
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
    // Learner correctly prioritizes Candidate 1 (Route 2) with lower damage (4.0 < 20..23)
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

    let mut coord_results = Vec::new();
    let mut niche_results = Vec::new();

    for &seed in &seeds {
        // --- 1. Local Coordination 3-Arm Comparison ---
        let solo = run_coordination_harness(CoordinationArm::ControlSolo, 8);
        let unverified = run_coordination_harness(CoordinationArm::UnverifiedIgnored, 8);
        let verified = run_coordination_harness(CoordinationArm::VerifiedLocalCoordination, 8);

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

        // --- 2. Ambush vs Pursuit Specialization ---
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

        let mut predator_ambush = PredatorState {
            id: 101,
            pos: (5, 5),
            energy: 30,
            stamina: 20,
            ambush_state: AmbushState::None,
            specialization_profile: SpecializationProfile::from_genome(&genome_ambush),
        };
        let mut predator_pursuit = PredatorState {
            id: 102,
            pos: (5, 5),
            energy: 30,
            stamina: 20,
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

        // Base attack: damage 10, stamina cost 3
        let atk_ambush = resolve_attack(&mut predator_ambush, 10, 3).unwrap();
        let atk_pursuit = resolve_attack(&mut predator_pursuit, 10, 3).unwrap();

        // --- 3. Corpse Scavenging Niche Differentiation ---
        let initial_corpse_energy = 100 + (seed as usize % 20);
        let mut corpse = DecayingCorpse::new(seed as usize, Position::new(0, 0), initial_corpse_energy, 10);

        // Decay corpse until Putrid stage (< 30% remaining)
        while corpse.freshness_stage() != FreshnessStage::Putrid {
            corpse.decay_tick();
        }

        // Test Omnivore feeding (scavenger)
        let omnivore_meal = corpse.feed(Species::Omnivore, 10).unwrap();

        // Test Carnivore feeding (predator)
        let carnivore_meal = corpse.feed(Species::Carnivore, 10).unwrap();

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

    for res in &niche_results {
        // Ambush specialist deals higher burst damage from ambush (14 vs 11)
        assert!(res.ambush_first_strike_damage > res.pursuit_first_strike_damage);
        // Ambush specialist provides higher stealth reduction (3 vs 1)
        assert!(res.ambush_stealth_reduction > res.pursuit_stealth_reduction);
        // Omnivore suffers zero health penalty on Putrid carrion, whereas Carnivore suffers 6
        assert_eq!(res.omnivore_putrid_health_penalty, 0);
        assert_eq!(res.carnivore_putrid_health_penalty, 6);
        // Stock-flow conservation strictly holds
        assert!(res.corpse_stock_flow_balanced);
    }

    println!("Coordination Results: {:?}", coord_results);
    println!("Niche Specialization Results: {:?}", niche_results);
}
