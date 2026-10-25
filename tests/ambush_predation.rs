#[path = "../examples/support/ambush_predation_candidate.rs"]
mod ambush_predation_candidate;

use curious::{
    entity::genetics::Genome,
    map::cell::Cell,
};
use ambush_predation_candidate::*;

#[test]
fn test_stationary_adjacent_to_cover_enters_ambush_and_pays_focus_energy() {
    let genome = Genome {
        mobility: 0,
        vision: 1,
        stamina: 2,
    };
    let mut predator = PredatorState {
        id: 10,
        pos: (4, 4),
        energy: 20,
        stamina: 15,
        ambush_state: AmbushState::None,
        specialization_profile: SpecializationProfile::from_genome(&genome),
    };

    let focus_cost = predator.specialization_profile.ambush_focus_energy_cost;

    // 1. Enter ambush when stationary adjacent to Burrow
    let result = update_ambush_state(&mut predator, true, Some(CoverKind::Burrow));
    assert_eq!(
        result,
        AmbushUpdateResult::Activated {
            cover: CoverKind::Burrow,
            energy_paid: focus_cost,
        }
    );
    assert_eq!(predator.energy, 20 - focus_cost);
    assert!(matches!(
        predator.ambush_state,
        AmbushState::Active {
            ticks_focused: 1,
            cover: CoverKind::Burrow
        }
    ));

    // 2. Maintain ambush on subsequent tick
    let maintain = update_ambush_state(&mut predator, true, Some(CoverKind::Burrow));
    assert_eq!(
        maintain,
        AmbushUpdateResult::Maintained {
            cover: CoverKind::Burrow,
            ticks_focused: 2,
            energy_paid: focus_cost,
        }
    );
    assert_eq!(predator.energy, 20 - 2 * focus_cost);
}

#[test]
fn test_movement_or_absence_of_cover_or_energy_breaks_ambush() {
    let genome = Genome {
        mobility: 1,
        vision: 1,
        stamina: 1,
    };
    let mut predator = PredatorState {
        id: 11,
        pos: (2, 2),
        energy: 10,
        stamina: 10,
        ambush_state: AmbushState::Active {
            ticks_focused: 3,
            cover: CoverKind::Water,
        },
        specialization_profile: SpecializationProfile::from_genome(&genome),
    };

    // 1. Movement (is_stationary = false) breaks ambush
    let res_move = update_ambush_state(&mut predator, false, Some(CoverKind::Water));
    assert_eq!(res_move, AmbushUpdateResult::BrokenNotStationary);
    assert_eq!(predator.ambush_state, AmbushState::None);

    // Reactivate
    let _ = update_ambush_state(&mut predator, true, Some(CoverKind::Food));
    assert!(matches!(predator.ambush_state, AmbushState::Active { .. }));

    // 2. Leaving cover (no adjacent cover) breaks ambush
    let res_no_cover = update_ambush_state(&mut predator, true, None);
    assert_eq!(res_no_cover, AmbushUpdateResult::BrokenNoAdjacentCover);
    assert_eq!(predator.ambush_state, AmbushState::None);

    // 3. Insufficient energy breaks ambush
    predator.energy = 0;
    let res_no_energy = update_ambush_state(&mut predator, true, Some(CoverKind::Burrow));
    assert_eq!(res_no_energy, AmbushUpdateResult::BrokenInsufficientEnergy);
    assert_eq!(predator.ambush_state, AmbushState::None);
}

#[test]
fn test_ambush_state_reduces_effective_detectability_radius_bounded_to_one() {
    let genome = Genome {
        mobility: 0,
        vision: 1,
        stamina: 2, // AmbushSpecialist with stealth_reduction = 3
    };
    let mut predator = PredatorState {
        id: 12,
        pos: (0, 0),
        energy: 20,
        stamina: 20,
        ambush_state: AmbushState::None,
        specialization_profile: SpecializationProfile::from_genome(&genome),
    };

    let prey_pos_dist_4 = (4, 0); // distance 4
    let prey_pos_dist_2 = (2, 0); // distance 2
    let prey_vision = 5;

    // Outside ambush: prey spots predator at distance 4 (4 <= 5)
    assert!(is_predator_spotted_by_prey(&predator, prey_pos_dist_4, prey_vision));

    // Enter ambush: stealth_reduction = 3 -> effective radius = (5 - 3).max(1) = 2
    predator.ambush_state = AmbushState::Active {
        ticks_focused: 2,
        cover: CoverKind::Burrow,
    };

    // Prey at distance 4 does NOT spot ambushing predator (4 > 2)
    assert!(!is_predator_spotted_by_prey(&predator, prey_pos_dist_4, prey_vision));
    // Prey at distance 2 DOES spot ambushing predator (2 <= 2)
    assert!(is_predator_spotted_by_prey(&predator, prey_pos_dist_2, prey_vision));

    // Floor bound test: even with high stealth reduction, radius never drops below 1
    assert_eq!(effective_detectability_radius(2, 5), 1);
    let prey_pos_dist_1 = (1, 0);
    assert!(is_predator_spotted_by_prey(&predator, prey_pos_dist_1, 2));
}

#[test]
fn test_first_strike_deals_bonus_damage_reduced_stamina_and_clears_ambush() {
    let genome = Genome {
        mobility: 0,
        vision: 1,
        stamina: 2, // AmbushSpecialist with bonus_ambush_damage = 4
    };
    let mut predator = PredatorState {
        id: 13,
        pos: (1, 1),
        energy: 20,
        stamina: 15,
        ambush_state: AmbushState::Active {
            ticks_focused: 2,
            cover: CoverKind::Food,
        },
        specialization_profile: SpecializationProfile::from_genome(&genome),
    };

    let base_damage = 5;
    let base_stamina_cost = 4;

    let outcome = resolve_attack(&mut predator, base_damage, base_stamina_cost).unwrap();

    assert!(outcome.is_ambush_first_strike);
    // Bonus damage applied: 5 + 4 = 9
    assert_eq!(outcome.bonus_ambush_damage, 4);
    assert_eq!(outcome.total_damage, 9);
    // Reduced stamina cost: base_stamina_cost - 1 = 3
    assert_eq!(outcome.stamina_paid, 3);
    assert_eq!(predator.stamina, 12);
    // Ambush state is immediately cleared!
    assert_eq!(predator.ambush_state, AmbushState::None);
}

#[test]
fn test_normal_attack_outside_ambush_deals_standard_damage_and_full_stamina() {
    let genome = Genome {
        mobility: 1,
        vision: 1,
        stamina: 1,
    };
    let mut predator = PredatorState {
        id: 14,
        pos: (1, 1),
        energy: 20,
        stamina: 15,
        ambush_state: AmbushState::None,
        specialization_profile: SpecializationProfile::from_genome(&genome),
    };

    let base_damage = 5;
    let base_stamina_cost = 4;

    let outcome = resolve_attack(&mut predator, base_damage, base_stamina_cost).unwrap();

    assert!(!outcome.is_ambush_first_strike);
    assert_eq!(outcome.bonus_ambush_damage, 0);
    assert_eq!(outcome.total_damage, base_damage);
    assert_eq!(outcome.stamina_paid, base_stamina_cost);
    assert_eq!(predator.stamina, 11);
    assert_eq!(predator.ambush_state, AmbushState::None);
}

#[test]
fn test_k6_alleles_classify_pursuit_vs_ambush_specialists_with_tradeoffs() {
    // 1. Pursuit Specialist (mobility == 2)
    let pursuit_genome = Genome {
        mobility: 2,
        vision: 1,
        stamina: 0,
    };
    let pursuit_profile = SpecializationProfile::from_genome(&pursuit_genome);
    assert_eq!(pursuit_profile.specialization, PredatorSpecialization::PursuitSpecialist);
    assert_eq!(pursuit_profile.pursuit_stamina_cost_per_step, 1); // low stamina cost per step
    assert_eq!(pursuit_profile.max_pursuit_steps, 3);            // multi-step chase
    assert_eq!(pursuit_profile.stealth_reduction, 1);             // lower stealth

    // 2. Ambush Specialist (mobility == 0 && stamina == 2)
    let ambush_genome = Genome {
        mobility: 0,
        vision: 1,
        stamina: 2,
    };
    let ambush_profile = SpecializationProfile::from_genome(&ambush_genome);
    assert_eq!(ambush_profile.specialization, PredatorSpecialization::AmbushSpecialist);
    assert_eq!(ambush_profile.ambush_focus_energy_cost, 1);       // low waiting cost
    assert_eq!(ambush_profile.stealth_reduction, 3);             // high stealth
    assert_eq!(ambush_profile.bonus_ambush_damage, 4);           // high burst damage
    assert_eq!(ambush_profile.pursuit_stamina_cost_per_step, 3); // poor sprint

    // 3. Generalist (other combinations)
    let generalist_genome = Genome {
        mobility: 1,
        vision: 1,
        stamina: 1,
    };
    let gen_profile = SpecializationProfile::from_genome(&generalist_genome);
    assert_eq!(gen_profile.specialization, PredatorSpecialization::Generalist);

    // 4. Verify cover cell classification
    assert_eq!(is_cover_cell(&Cell::Burrow), Some(CoverKind::Burrow));
    assert_eq!(is_cover_cell(&Cell::Water { amount: 5 }), Some(CoverKind::Water));
    assert_eq!(is_cover_cell(&Cell::Food { amount: 10 }), Some(CoverKind::Food));
    assert_eq!(is_cover_cell(&Cell::Empty), None);
    assert_eq!(is_cover_cell(&Cell::Wall), None);
}
