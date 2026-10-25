#[path = "../examples/support/corpse_scavenging_candidate.rs"]
mod corpse_scavenging_candidate;

use curious::{entity::species::Species, map::movement::Position};
use corpse_scavenging_candidate::*;

#[test]
fn test_corpse_decay_stages_progression() {
    let mut corpse = DecayingCorpse::new(1, Position::new(2, 2), 100, 5);

    // Initial state: 100 energy (100%) => Fresh
    assert_eq!(corpse.freshness_stage(), FreshnessStage::Fresh);
    assert_eq!(corpse.remaining_energy, 100);

    // Decay for 6 ticks: 100 - (6 * 5) = 70 energy (70%) => still Fresh
    for _ in 0..6 {
        corpse.decay_tick();
    }
    assert_eq!(corpse.remaining_energy, 70);
    assert_eq!(corpse.freshness_stage(), FreshnessStage::Fresh);

    // Next tick: 70 - 5 = 65 energy (65% < 70%) => transitions to Aging!
    corpse.decay_tick();
    assert_eq!(corpse.remaining_energy, 65);
    assert_eq!(corpse.freshness_stage(), FreshnessStage::Aging);

    // Decay until 25 energy (25% < 30%) => transitions to Putrid!
    for _ in 0..8 {
        corpse.decay_tick();
    }
    assert_eq!(corpse.remaining_energy, 25);
    assert_eq!(corpse.freshness_stage(), FreshnessStage::Putrid);
}

#[test]
fn test_scent_signal_scaling_by_decay_stage() {
    let mut corpse = DecayingCorpse::new(10, Position::new(4, -3), 100, 10);

    // Fresh stage scent emission
    let fresh_signal = corpse.emit_scent(1).expect("Fresh corpse emits scent");
    assert_eq!(fresh_signal.origin, Position::new(4, -3));
    assert_eq!(fresh_signal.kind, SignalKind::Scent);
    assert_eq!(fresh_signal.radius, 3);
    assert_eq!(fresh_signal.intensity, 10);
    assert_eq!(fresh_signal.stage, FreshnessStage::Fresh);

    // Advance to Aging stage: decay to 50 energy (50%)
    for _ in 0..5 {
        corpse.decay_tick();
    }
    assert_eq!(corpse.freshness_stage(), FreshnessStage::Aging);
    let aging_signal = corpse.emit_scent(6).expect("Aging corpse emits scent");
    assert_eq!(aging_signal.radius, 6);
    assert_eq!(aging_signal.intensity, 25);
    assert_eq!(aging_signal.stage, FreshnessStage::Aging);

    // Advance to Putrid stage: decay to 20 energy (20%)
    for _ in 0..3 {
        corpse.decay_tick();
    }
    assert_eq!(corpse.freshness_stage(), FreshnessStage::Putrid);
    let putrid_signal = corpse.emit_scent(9).expect("Putrid corpse emits scent");
    assert_eq!(putrid_signal.radius, 10);
    assert_eq!(putrid_signal.intensity, 50);
    assert_eq!(putrid_signal.stage, FreshnessStage::Putrid);

    // Putrid signal has larger radius and higher intensity than Fresh and Aging
    assert!(putrid_signal.radius > aging_signal.radius && aging_signal.radius > fresh_signal.radius);
    assert!(putrid_signal.intensity > aging_signal.intensity && aging_signal.intensity > fresh_signal.intensity);
}

#[test]
fn test_exact_corpse_energy_stock_flow_accounting() {
    let mut corpse = DecayingCorpse::new(42, Position::new(0, 0), 120, 3);

    // Step 1: Carnivore eats 20 from fresh corpse
    let m1 = corpse.feed(Species::Carnivore, 20).unwrap();
    assert_eq!(m1.energy_gained, 20);

    // Step 2: 4 ticks of natural decay (4 * 3 = 12)
    for _ in 0..4 {
        corpse.decay_tick();
    }

    // Step 3: Omnivore eats 25 from aging corpse
    let m2 = corpse.feed(Species::Omnivore, 25).unwrap();
    assert_eq!(m2.energy_gained, 25);

    // Step 4: 5 ticks of natural decay (5 * 3 = 15)
    for _ in 0..5 {
        corpse.decay_tick();
    }

    // Step 5: Carnivore eats 15 from putrid corpse
    let m3 = corpse.feed(Species::Carnivore, 15).unwrap();
    assert_eq!(m3.energy_gained, 15);

    // Step 6: Omnivore eats 10 from putrid corpse
    let m4 = corpse.feed(Species::Omnivore, 10).unwrap();
    assert_eq!(m4.energy_gained, 10);

    // Step 7: Natural decay for 10 ticks until exhaustion
    for _ in 0..10 {
        corpse.decay_tick();
    }

    let audit = corpse.audit();

    // Verify each component
    assert_eq!(audit.initial_corpse_energy, 120);
    assert_eq!(audit.eaten_by_carnivores, 35); // 20 + 15
    assert_eq!(audit.eaten_by_omnivores, 35);  // 25 + 10

    // Exact closure: initial == eaten_carnivore + eaten_omnivore + decayed + remaining
    assert_eq!(
        audit.initial_corpse_energy,
        audit.eaten_by_carnivores + audit.eaten_by_omnivores + audit.decayed_energy + audit.remaining_corpse_energy
    );
    assert!(audit.is_balanced());
}

#[test]
fn test_fresh_corpse_nutrition_parity() {
    let mut corpse = DecayingCorpse::new(5, Position::new(1, 1), 80, 2);

    // 1. Herbivore cannot feed on meat
    let herbivore_err = corpse.feed(Species::Herbivore, 10);
    assert!(herbivore_err.is_err());

    // 2. Carnivore feeds on Fresh corpse
    let carnivore_meal = corpse.feed(Species::Carnivore, 10).unwrap();
    assert_eq!(carnivore_meal.stage, FreshnessStage::Fresh);
    assert_eq!(carnivore_meal.energy_gained, 10);
    assert_eq!(carnivore_meal.health_penalty, 0);
    assert_eq!(carnivore_meal.stamina_penalty, 0);

    // 3. Omnivore feeds on Fresh corpse
    let omnivore_meal = corpse.feed(Species::Omnivore, 10).unwrap();
    assert_eq!(omnivore_meal.stage, FreshnessStage::Fresh);
    assert_eq!(omnivore_meal.energy_gained, 10);
    assert_eq!(omnivore_meal.health_penalty, 0);
    assert_eq!(omnivore_meal.stamina_penalty, 0);

    // Nutrition parity: both species get full meal and 0 penalty
    assert_eq!(carnivore_meal.energy_gained, omnivore_meal.energy_gained);
    assert_eq!(carnivore_meal.health_penalty, omnivore_meal.health_penalty);
    assert_eq!(carnivore_meal.stamina_penalty, omnivore_meal.stamina_penalty);
}

#[test]
fn test_putrid_corpse_scavenger_niche_separation() {
    let mut corpse = DecayingCorpse::new(8, Position::new(3, 3), 100, 10);

    // Decay corpse into Putrid stage (<= 20 energy remaining)
    for _ in 0..8 {
        corpse.decay_tick();
    }
    assert_eq!(corpse.remaining_energy, 20);
    assert_eq!(corpse.freshness_stage(), FreshnessStage::Putrid);

    // Case A: Carnivore (predator) eats from Putrid corpse
    let carnivore_meal = corpse.feed(Species::Carnivore, 5).unwrap();
    assert_eq!(carnivore_meal.stage, FreshnessStage::Putrid);
    assert_eq!(carnivore_meal.energy_gained, 5);
    assert_eq!(
        carnivore_meal.health_penalty, 6,
        "Strict predators suffer severe health spoilage penalty on putrid carrion"
    );
    assert_eq!(
        carnivore_meal.stamina_penalty, 8,
        "Strict predators suffer severe stamina spoilage penalty on putrid carrion"
    );

    // Case B: Omnivore (scavenger) eats from Putrid corpse
    let omnivore_meal = corpse.feed(Species::Omnivore, 5).unwrap();
    assert_eq!(omnivore_meal.stage, FreshnessStage::Putrid);
    assert_eq!(omnivore_meal.energy_gained, 5);
    assert_eq!(
        omnivore_meal.health_penalty, 0,
        "Scavengers with resistance suffer 0 health penalty on putrid carrion"
    );
    assert_eq!(
        omnivore_meal.stamina_penalty, 0,
        "Scavengers with resistance suffer 0 stamina penalty on putrid carrion"
    );

    // Strict ecological niche separation verified
    assert!(carnivore_meal.health_penalty > omnivore_meal.health_penalty);
    assert!(carnivore_meal.stamina_penalty > omnivore_meal.stamina_penalty);
}

#[test]
fn test_complete_decay_to_exhaustion_or_zero_energy() {
    let mut corpse = DecayingCorpse::new(99, Position::new(0, 0), 20, 7);

    assert!(!corpse.is_exhausted());
    assert!(corpse.emit_scent(1).is_some());

    // Tick 1: 20 - 7 = 13
    assert_eq!(corpse.decay_tick(), 7);
    assert_eq!(corpse.remaining_energy, 13);
    assert!(!corpse.is_exhausted());

    // Tick 2: 13 - 7 = 6
    assert_eq!(corpse.decay_tick(), 7);
    assert_eq!(corpse.remaining_energy, 6);
    assert!(!corpse.is_exhausted());

    // Tick 3: 6 - 6 = 0 (decay amount clamped to remaining energy)
    assert_eq!(corpse.decay_tick(), 6);
    assert_eq!(corpse.remaining_energy, 0);
    assert!(corpse.is_exhausted());

    // Exhausted corpse emits NO scent
    assert!(corpse.emit_scent(4).is_none());

    // Further decay yields 0
    assert_eq!(corpse.decay_tick(), 0);

    // Further feeding is rejected
    let feed_res = corpse.feed(Species::Carnivore, 5);
    assert!(feed_res.is_err());

    // Stock-flow audit closes at exactly initial 20
    let audit = corpse.audit();
    assert_eq!(audit.initial_corpse_energy, 20);
    assert_eq!(audit.decayed_energy, 20);
    assert_eq!(audit.remaining_corpse_energy, 0);
    assert!(audit.is_balanced());
}
