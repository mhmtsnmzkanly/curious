#[path = "../examples/support/dynamic_habitat_candidate.rs"]
mod dynamic_habitat_candidate;

use curious::{entity::species::Species, map::movement::Position};
use dynamic_habitat_candidate::*;

#[test]
fn test_burrow_digging_success_and_costs() {
    let mut engine = DynamicHabitatEngine::new();
    let pos = Position::new(2, 3);

    // Initial herbivore parameters
    let mut energy = 50;
    let mut stamina = 20;

    let result = engine.dig_burrow(
        1,
        Species::Herbivore,
        true, // can_burrow
        energy,
        stamina,
        pos,
        true, // cell is empty
    );

    assert!(result.is_ok(), "Herbivore should succeed in digging burrow on empty cell");
    let burrow = result.unwrap();
    assert_eq!(burrow.pos, pos);
    assert_eq!(burrow.durability, INITIAL_BURROW_DURABILITY);
    assert_eq!(burrow.durability, 50);

    // Verify costs
    energy -= DIG_ENERGY_COST;
    stamina -= DIG_STAMINA_COST;
    assert_eq!(energy, 38);
    assert_eq!(stamina, 14);

    // Verify burrow exists in engine
    assert!(engine.has_burrow(pos));
}

#[test]
fn test_burrow_digging_rejection_conditions() {
    let mut engine = DynamicHabitatEngine::new();
    let pos = Position::new(1, 1);

    // 1. Carnivore cannot burrow
    let err_carnivore = engine.dig_burrow(2, Species::Carnivore, true, 50, 20, pos, true);
    assert!(matches!(err_carnivore, Err(DigError::NotHerbivore)));

    // 2. Herbivore without can_burrow capability
    let err_no_burrow = engine.dig_burrow(3, Species::Herbivore, false, 50, 20, pos, true);
    assert!(matches!(err_no_burrow, Err(DigError::CannotBurrow)));

    // 3. Insufficient energy
    let err_energy = engine.dig_burrow(4, Species::Herbivore, true, 10, 20, pos, true);
    assert!(matches!(
        err_energy,
        Err(DigError::InsufficientEnergy { available: 10, required: 12 })
    ));

    // 4. Insufficient stamina
    let err_stamina = engine.dig_burrow(5, Species::Herbivore, true, 50, 4, pos, true);
    assert!(matches!(
        err_stamina,
        Err(DigError::InsufficientStamina { available: 4, required: 6 })
    ));

    // 5. Non-empty cell (e.g. wall, food, or another burrow)
    let err_non_empty = engine.dig_burrow(6, Species::Herbivore, true, 50, 20, pos, false);
    assert!(matches!(err_non_empty, Err(DigError::CellNotEmpty)));

    // 6. Cell already has a burrow
    let _ = engine.dig_burrow(7, Species::Herbivore, true, 50, 20, pos, true);
    let err_duplicate = engine.dig_burrow(8, Species::Herbivore, true, 50, 20, pos, true);
    assert!(matches!(err_duplicate, Err(DigError::CellNotEmpty)));
}

#[test]
fn test_burrow_durability_decay_and_combat_stress_collapse() {
    let mut engine = DynamicHabitatEngine::new();
    let quiet_pos = Position::new(0, 0);
    let combat_pos = Position::new(5, 5);

    // Create two burrows
    let _ = engine.dig_burrow(1, Species::Herbivore, true, 50, 20, quiet_pos, true);
    let _ = engine.dig_burrow(2, Species::Herbivore, true, 50, 20, combat_pos, true);

    // Combat occurs adjacent to combat_pos at (5, 6)
    let combat_events = vec![Position::new(5, 6)];
    // Tick 1
    let collapsed = engine.decay_burrows(&combat_events, DEFAULT_COMBAT_STRESS_DECAY);
    assert!(collapsed.is_empty());

    let quiet_durability = engine.burrow_durability(quiet_pos).unwrap();
    let combat_durability = engine.burrow_durability(combat_pos).unwrap();
    assert_eq!(quiet_durability, 49, "Natural decay should be 1 per tick");
    assert_eq!(combat_durability, 45, "Combat stress decay should be 5 (1 + 4) per tick");

    // Manually lower durability near collapse to verify deterministic collapse
    engine.set_burrow_durability(combat_pos, 4);

    // Next combat tick should trigger collapse (4 - 5 -> 0)
    let collapsed = engine.decay_burrows(&combat_events, DEFAULT_COMBAT_STRESS_DECAY);
    assert_eq!(collapsed, vec![combat_pos]);
    assert!(!engine.has_burrow(combat_pos));
    assert!(engine.has_burrow(quiet_pos));

    // Natural decay until quiet burrow also collapses
    engine.set_burrow_durability(quiet_pos, 1);
    let collapsed_quiet = engine.decay_burrows(&[], DEFAULT_COMBAT_STRESS_DECAY);
    assert_eq!(collapsed_quiet, vec![quiet_pos]);
    assert!(!engine.has_burrow(quiet_pos));
}

#[test]
fn test_overgrazing_degradation_and_replenishment_halving() {
    let mut engine = DynamicHabitatEngine::new();
    let plant_pos = Position::new(3, 3);
    // Capacity 20, replenish rate 4 per interval of 1 tick
    engine.add_food_site(plant_pos, 10, 20, 4, 1);

    // 1st depletion to 0
    let eaten = engine.record_grazing(plant_pos, 10).unwrap();
    assert_eq!(eaten, 10);
    assert_eq!(engine.food_amount(plant_pos).unwrap(), 0);
    let site = engine.food_site(plant_pos).unwrap();
    assert_eq!(site.consecutive_depletions, 1);
    assert_eq!(site.status, OvergrazingStatus::Normal);

    // Replenish tick: restores 4 food
    let _ = engine.tick(1);
    assert_eq!(engine.food_amount(plant_pos).unwrap(), 4);

    // 2nd depletion to 0
    let eaten = engine.record_grazing(plant_pos, 4).unwrap();
    assert_eq!(eaten, 4);
    assert_eq!(engine.food_site(plant_pos).unwrap().consecutive_depletions, 2);
    assert_eq!(engine.food_site(plant_pos).unwrap().status, OvergrazingStatus::Normal);

    // Replenish tick: restores 4 food
    let _ = engine.tick(2);
    assert_eq!(engine.food_amount(plant_pos).unwrap(), 4);

    // 3rd depletion to 0 -> triggers Overgrazed state!
    let eaten = engine.record_grazing(plant_pos, 4).unwrap();
    assert_eq!(eaten, 4);
    let site = engine.food_site(plant_pos).unwrap();
    assert_eq!(site.consecutive_depletions, 3);
    assert_eq!(
        site.status,
        OvergrazingStatus::Overgrazed {
            recovery_cooldown: DEFAULT_OVERGRAZED_COOLDOWN
        }
    );

    // Replenishment rate is halved while overgrazed (4 / 2 = 2)
    assert_eq!(site.effective_replenish(), 2);

    let _ = engine.tick(3);
    assert_eq!(
        engine.food_amount(plant_pos).unwrap(),
        2,
        "Replenishment must be halved during overgrazing"
    );

    // Advance ticks through the recovery cooldown
    for t in 4..=(3 + DEFAULT_OVERGRAZED_COOLDOWN) {
        let _ = engine.tick(t);
    }

    // After cooldown completes, status returns to Normal and replenishment returns to 4
    let recovered_site = engine.food_site(plant_pos).unwrap();
    assert_eq!(recovered_site.status, OvergrazingStatus::Normal);
    assert_eq!(recovered_site.consecutive_depletions, 0);
    assert_eq!(recovered_site.effective_replenish(), 4);
}

#[test]
fn test_seed_dispersal_and_maturation_into_renewable_food() {
    let mut engine = DynamicHabitatEngine::new();
    let seed_pos = Position::new(7, 8);

    let max_energy = 100;
    let _expected_threshold = (max_energy * 3) / 4; // 75

    // 1. Underfed herbivore cannot deposit seed
    let underfed = engine.deposit_seed(1, Species::Herbivore, 70, max_energy, seed_pos, true);
    assert!(matches!(
        underfed,
        Err(SeedError::InsufficientEnergyReserve {
            available: 70,
            threshold: 75
        })
    ));

    // 2. Carnivore cannot deposit seed
    let carnivore = engine.deposit_seed(2, Species::Carnivore, 80, max_energy, seed_pos, true);
    assert!(matches!(carnivore, Err(SeedError::NotHerbivore)));

    // 3. Well-fed herbivore (energy = 80 >= 75) deposits seed on Empty cell
    let seed_result = engine.deposit_seed(1, Species::Herbivore, 80, max_energy, seed_pos, true);
    assert!(seed_result.is_ok());
    let patch = seed_result.unwrap();
    assert_eq!(patch.pos, seed_pos);
    assert_eq!(patch.maturation_ticks, DEFAULT_SEED_MATURATION_TICKS);
    assert_eq!(patch.seed_energy_cost, SEED_ENERGY_COST);

    // Ticking 4 times: patch matures, not yet sprouted
    for t in 1..=4 {
        let sprouted = engine.tick(t);
        assert!(sprouted.is_empty());
        assert_eq!(engine.food_amount(seed_pos), None);
    }

    // 5th tick: patch reaches 0 maturation ticks and sprouts!
    let sprouted = engine.tick(5);
    assert_eq!(sprouted, vec![seed_pos]);
    assert_eq!(
        engine.food_amount(seed_pos).unwrap(),
        DEFAULT_SPROUT_AMOUNT,
        "Sprouted cell should have full initial sprout amount"
    );

    // Sprouted patch becomes renewable Food site
    assert!(engine.has_food_site(seed_pos));
    let new_site = engine.food_site(seed_pos).unwrap();
    assert_eq!(new_site.capacity, 20);
}

#[test]
fn test_dynamic_habitat_strict_stock_flow_accounting() {
    let mut engine = DynamicHabitatEngine::new();

    let pos1 = Position::new(0, 0);
    let pos2 = Position::new(1, 0);
    let seed_pos = Position::new(2, 2);

    // Food site 1: initial 20, capacity 30, replenish 3, interval 1
    engine.add_food_site(pos1, 20, 30, 3, 1);
    // Food site 2: initial 15, capacity 25, replenish 2, interval 2
    engine.add_food_site(pos2, 15, 25, 2, 2);

    // Step 1: Herbivore grazes from pos1 (eats 8)
    let eaten1 = engine.record_grazing(pos1, 8).unwrap();
    assert_eq!(eaten1, 8);

    // Step 2: Herbivore grazes from pos2 (eats 10)
    let eaten2 = engine.record_grazing(pos2, 10).unwrap();
    assert_eq!(eaten2, 10);

    // Step 3: Well-fed herbivore deposits seed at seed_pos
    let patch = engine
        .deposit_seed(1, Species::Herbivore, 85, 100, seed_pos, true)
        .unwrap();
    assert_eq!(patch.seed_energy_cost, 8);

    // Step 4: Run simulation for 6 ticks (allowing replenishment and seed sprouting)
    for tick in 1..=6 {
        let _ = engine.tick(tick);
    }

    // Step 5: Herbivore grazes 5 from sprouted patch
    let eaten_sprout = engine.record_grazing(seed_pos, 5).unwrap();
    assert_eq!(eaten_sprout, 5);

    // Verify stock-flow ledger
    let ledger = engine.audit_stock_flow();

    // Mathematical identity check:
    // initial + replenished + sprouted - eaten - seed_cost == net_balance
    assert_eq!(
        ledger.initial + ledger.replenished + ledger.sprouted - ledger.eaten - ledger.seed_cost,
        ledger.net_balance
    );

    // Physical stock check:
    // initial + replenished + sprouted - eaten == current_food
    assert_eq!(
        ledger.initial + ledger.replenished + ledger.sprouted - ledger.eaten,
        ledger.current_food
    );

    // Combined audit balance closure
    assert!(
        ledger.is_balanced(),
        "Strict stock-flow accounting must close exactly: {:?}",
        ledger
    );
}
