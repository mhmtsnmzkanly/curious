#[path = "../examples/support/juvenile_parental_care_candidate.rs"]
mod juvenile_parental_care_candidate;

use juvenile_parental_care_candidate::*;

#[test]
fn test_juvenile_physiology_scales_traits_and_respects_floors() {
    // Normal adult traits: speed 3, vision 6, damage 4, bmr 2
    let juvenile = juvenile_physiology(5, 20, 3, 6, 4, 2);
    assert!(juvenile.is_juvenile);
    assert_eq!(juvenile.speed, 2);  // 3 * 2 / 3 = 2
    assert_eq!(juvenile.vision, 4); // 6 * 2 / 3 = 4
    assert_eq!(juvenile.damage, 2); // 4 / 2 = 2
    assert_eq!(juvenile.bmr, 1);    // 2 / 2 = 1

    // Small adult traits: verify minimum floor bounds
    let small_juvenile = juvenile_physiology(0, 15, 1, 1, 1, 1);
    assert!(small_juvenile.is_juvenile);
    assert_eq!(small_juvenile.speed, 1);  // (1 * 2 / 3).max(1) = 1
    assert_eq!(small_juvenile.vision, 2); // (1 * 2 / 3).max(2) = 2
    assert_eq!(small_juvenile.damage, 1); // (1 / 2).max(1) = 1
    assert_eq!(small_juvenile.bmr, 1);    // (1 / 2).max(1) = 1
}

#[test]
fn test_juvenile_physiology_reaches_full_adult_at_maturity() {
    // At exact maturity age
    let mature = juvenile_physiology(20, 20, 3, 6, 4, 2);
    assert!(!mature.is_juvenile);
    assert_eq!(mature.speed, 3);
    assert_eq!(mature.vision, 6);
    assert_eq!(mature.damage, 4);
    assert_eq!(mature.bmr, 2);

    // Well past maturity age
    let old = juvenile_physiology(50, 20, 3, 6, 4, 2);
    assert!(!old.is_juvenile);
    assert_eq!(old.speed, 3);
    assert_eq!(old.vision, 6);
    assert_eq!(old.damage, 4);
    assert_eq!(old.bmr, 2);
}

#[test]
fn test_resolve_feed_exact_energy_conservation_and_stamina_cost() {
    let mut parent = ParentState {
        id: 101,
        pos: (5, 5),
        energy: 60,
        stamina: 30,
    };
    let mut child = OffspringState {
        id: 202,
        parents: vec![101],
        pos: (5, 6), // adjacent
        age: 8,
        maturity_age: 20,
        energy: 15,
        max_energy: 100,
    };

    let parent_energy_before = parent.energy;
    let child_energy_before = child.energy;
    let transfer_amount = 25;
    let exertion_cost = 4;

    let result = resolve_feed_offspring(&mut parent, &mut child, transfer_amount, exertion_cost);
    assert!(result.is_ok());
    let outcome = result.unwrap();

    assert_eq!(outcome.transferred_energy, 25);
    // Exact conservation: parent loss equals child gain
    assert_eq!(parent_energy_before - outcome.transferred_energy, parent.energy);
    assert_eq!(child_energy_before + outcome.transferred_energy, child.energy);
    assert_eq!(parent.energy, 35);
    assert_eq!(child.energy, 40);
    // Stamina cost paid
    assert_eq!(parent.stamina, 26);
}

#[test]
fn test_resolve_feed_caps_at_child_max_energy_without_parent_overdeduction() {
    let mut parent = ParentState {
        id: 101,
        pos: (5, 5),
        energy: 50,
        stamina: 20,
    };
    let mut child = OffspringState {
        id: 202,
        parents: vec![101],
        pos: (6, 5), // adjacent
        age: 10,
        maturity_age: 20,
        energy: 95,
        max_energy: 100, // Child only needs 5 energy!
    };

    let parent_energy_before = parent.energy;
    let child_energy_before = child.energy;
    let transfer_amount = 30; // Offers 30, but child only needs 5

    let result = resolve_feed_offspring(&mut parent, &mut child, transfer_amount, 2);
    assert!(result.is_ok());
    let outcome = result.unwrap();

    // Transferred is capped to child's missing capacity (5)
    assert_eq!(outcome.transferred_energy, 5);
    // Parent is deducted only 5, not 30!
    assert_eq!(parent.energy, parent_energy_before - 5);
    assert_eq!(parent.energy, 45);
    assert_eq!(child.energy, child_energy_before + 5);
    assert_eq!(child.energy, 100);
}

#[test]
fn test_resolve_feed_rejections() {
    let mut parent = ParentState {
        id: 101,
        pos: (0, 0),
        energy: 50,
        stamina: 10,
    };

    // 1. Not own offspring
    let mut non_child = OffspringState {
        id: 999,
        parents: vec![777], // parent 101 not in list
        pos: (0, 1),
        age: 5,
        maturity_age: 20,
        energy: 10,
        max_energy: 100,
    };
    assert_eq!(
        resolve_feed_offspring(&mut parent, &mut non_child, 10, 2),
        Err(FeedError::NotOwnOffspring)
    );

    // 2. Child already mature
    let mut mature_child = OffspringState {
        id: 202,
        parents: vec![101],
        pos: (0, 1),
        age: 20,
        maturity_age: 20,
        energy: 10,
        max_energy: 100,
    };
    assert_eq!(
        resolve_feed_offspring(&mut parent, &mut mature_child, 10, 2),
        Err(FeedError::ChildMature)
    );

    // 3. Not adjacent (distance > 1)
    let mut distant_child = OffspringState {
        id: 203,
        parents: vec![101],
        pos: (5, 5),
        age: 5,
        maturity_age: 20,
        energy: 10,
        max_energy: 100,
    };
    assert_eq!(
        resolve_feed_offspring(&mut parent, &mut distant_child, 10, 2),
        Err(FeedError::NotAdjacent)
    );

    // 4. Insufficient stamina
    let mut low_stamina_parent = ParentState {
        id: 101,
        pos: (0, 0),
        energy: 50,
        stamina: 1, // requires 3
    };
    let mut child = OffspringState {
        id: 204,
        parents: vec![101],
        pos: (0, 1),
        age: 5,
        maturity_age: 20,
        energy: 10,
        max_energy: 100,
    };
    assert_eq!(
        resolve_feed_offspring(&mut low_stamina_parent, &mut child, 10, 3),
        Err(FeedError::InsufficientStamina)
    );

    // 5. Parent has no energy
    let mut zero_energy_parent = ParentState {
        id: 101,
        pos: (0, 0),
        energy: 0,
        stamina: 10,
    };
    assert_eq!(
        resolve_feed_offspring(&mut zero_energy_parent, &mut child, 10, 2),
        Err(FeedError::ParentHasNoEnergy)
    );
}

#[test]
fn test_resolve_guard_offspring_interception_attribution_and_zero_memory_leak() {
    let mut guardian = GuardianState {
        id: 42,
        pos: (10, 10),
        health: 100,
        stamina: 20,
    };
    let mut juvenile = JuvenileVictimState {
        id: 84,
        parents: vec![42],
        pos: (10, 11), // adjacent
        age: 5,
        maturity_age: 18,
        health: 30,
    };

    let attacker_id = 999;
    let attack_damage = 15;
    let guard_stamina_cost = 5;

    // 1. Valid guard: guardian intercepts damage
    let outcome = resolve_guard_offspring(
        &mut guardian,
        &mut juvenile,
        attacker_id,
        attack_damage,
        guard_stamina_cost,
    );

    assert!(outcome.intercepted);
    assert_eq!(outcome.guardian_id, 42);
    assert_eq!(outcome.juvenile_id, 84);
    assert_eq!(outcome.attacker_id, 999);
    assert_eq!(outcome.intercepted_damage, 15);
    assert_eq!(outcome.damage_taken_by_juvenile, 0);

    // Juvenile remains unharmed
    assert_eq!(juvenile.health, 30);
    // Guardian absorbed damage and stamina cost
    assert_eq!(guardian.health, 85);
    assert_eq!(guardian.stamina, 15);

    // 2. Failed guard when guardian is depleted of stamina
    guardian.stamina = 2; // requires 5
    let failed_outcome = resolve_guard_offspring(
        &mut guardian,
        &mut juvenile,
        attacker_id,
        attack_damage,
        guard_stamina_cost,
    );

    assert!(!failed_outcome.intercepted);
    assert_eq!(failed_outcome.intercepted_damage, 0);
    assert_eq!(failed_outcome.damage_taken_by_juvenile, 15);
    // Juvenile takes the damage
    assert_eq!(juvenile.health, 15);
    // Guardian takes no damage and pays no stamina
    assert_eq!(guardian.health, 85);
    assert_eq!(guardian.stamina, 2);

    // 3. Zero memory leak assertion:
    // Physical guardian and juvenile interactions are strictly bodily actions;
    // they do not exchange or inherit any learned memory representations.
    assert_eq!(juvenile.parents, vec![42]);
}
