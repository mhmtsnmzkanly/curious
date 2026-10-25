#[path = "support/refuge_risk_baseline.rs"]
mod refuge_risk_baseline;

use curious::{
    creatures::herbivore::HerbivoreEntity,
    entity::{Entity, perception::Perception, species::Species},
    map::{
        cell::Cell,
        movement::{Direction, Position, Steps},
    },
};
use refuge_risk_baseline::{DamageBaseline, baseline_choice, estimate_effort, route_baselines};

fn life() -> curious::entity::lifestate::LifeState {
    HerbivoreEntity::default().life().clone()
}

#[test]
fn route_effort_is_full_route_cost_and_nominal_speed_ticks() {
    let estimate = estimate_effort(5, 3, 2);
    assert_eq!(estimate.nominal_paid_movement_energy, Some(10));
    assert_eq!(estimate.nominal_elapsed_ticks, Some(2));
    assert_eq!(estimate.status, "conditional_nominal_estimate");
}

#[test]
fn zero_or_missing_movement_inputs_are_unknown() {
    for (steps, speed, cost) in [(0, 1, 1), (3, 0, 1)] {
        let estimate = estimate_effort(steps, speed, cost);
        assert_eq!(estimate.nominal_paid_movement_energy, None);
        assert_eq!(estimate.nominal_elapsed_ticks, None);
        assert_eq!(estimate.status, "unknown_missing_path_or_speed");
    }
    let zero_cost = estimate_effort(3, 1, 0);
    assert_eq!(zero_cost.nominal_paid_movement_energy, Some(0));
    assert_eq!(zero_cost.nominal_elapsed_ticks, Some(3));
}

#[test]
fn fallback_chooses_shortest_then_original_perception_order() {
    let mut view = Perception::empty();
    view.shelters = vec![
        Steps(vec![Direction::Right, Direction::Right]),
        Steps(vec![Direction::Up]),
        Steps(vec![Direction::Left]),
    ];
    view.cells = vec![
        (Position::new(1, 0), Cell::Empty),
        (Position::new(2, 0), Cell::Burrow),
        (Position::new(0, -1), Cell::Burrow),
        (Position::new(-1, 0), Cell::Burrow),
    ];
    let routes = route_baselines(Position::new(0, 0), &life(), &view);
    assert_eq!(routes.len(), 3);
    assert_eq!(routes[0].perception_order, 0);
    assert_eq!(routes[1].perception_order, 1);
    assert_eq!(routes[2].perception_order, 2);
    assert_eq!(baseline_choice(&routes), Some(1));
    assert_eq!(routes[1].endpoint, Position::new(0, -1));
}

#[test]
fn ordinary_route_audit_keeps_more_than_eight_valid_paths() {
    let mut view = Perception::empty();
    view.shelters = vec![
        Steps(vec![Direction::Up]),
        Steps(vec![Direction::UpRight]),
        Steps(vec![Direction::Right]),
        Steps(vec![Direction::DownRight]),
        Steps(vec![Direction::Down]),
        Steps(vec![Direction::DownLeft]),
        Steps(vec![Direction::Left]),
        Steps(vec![Direction::UpLeft]),
        Steps(vec![Direction::Right, Direction::Right]),
    ];
    let routes = route_baselines(Position::new(0, 0), &life(), &view);
    assert_eq!(routes.len(), 9);
    assert_eq!(routes[8].perception_order, 8);
}

#[test]
fn occupied_endpoint_is_excluded_and_damage_stays_unknown() {
    let mut view = Perception::empty();
    view.shelters = vec![Steps(vec![Direction::Right]), Steps(vec![Direction::Up])];
    view.cells = vec![
        (Position::new(1, 0), Cell::Burrow),
        (Position::new(0, -1), Cell::Burrow),
    ];
    view.add_entity(
        40,
        Species::Carnivore,
        60,
        Steps(vec![Direction::Right]),
        false,
    );
    let routes = route_baselines(Position::new(0, 0), &life(), &view);
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].endpoint, Position::new(0, -1));
    assert_eq!(routes[0].damage.expected_threat_damage, None);
    assert_eq!(routes[0].damage.calibration_status, "unknown_uncalibrated");
    assert_eq!(routes[0].damage.descriptive_species_attack_damage, Some(20));
    assert!(routes[0].selected_threat.is_some());
}

#[test]
fn optional_threat_health_is_preserved_and_never_becomes_zero_damage() {
    let mut view = Perception::empty();
    view.shelters = vec![Steps(vec![Direction::Right])];
    view.cells = vec![(Position::new(1, 0), Cell::Burrow)];
    view.add_entity(
        40,
        Species::Carnivore,
        60,
        Steps(vec![Direction::Left]),
        false,
    );
    let routes = route_baselines(Position::new(0, 0), &life(), &view);
    let threat = routes[0].selected_threat.as_ref().unwrap();
    assert_eq!(threat.id, 40);
    assert_eq!(threat.observed_power, 60);
    assert_eq!(threat.observed_health, None);
    assert_eq!(routes[0].damage.expected_threat_damage, None);
    assert_eq!(routes[0].damage.descriptive_species_attack_damage, Some(20));
}

#[test]
fn ordinary_threat_instinct_keeps_an_empty_perceived_path() {
    let mut view = Perception::empty();
    view.shelters = vec![Steps(vec![Direction::Right])];
    view.cells = vec![(Position::new(1, 0), Cell::Burrow)];
    view.add_entity(41, Species::Carnivore, 60, Steps(vec![]), false);
    let routes = route_baselines(Position::new(0, 0), &life(), &view);
    assert_eq!(routes.len(), 1);
    let threat = routes[0].selected_threat.as_ref().unwrap();
    assert_eq!(threat.id, 41);
    assert!(threat.observed_steps.is_empty());
}

#[test]
fn contact_is_static_observed_geometry_not_a_damage_estimate() {
    let mut view = Perception::empty();
    view.shelters = vec![Steps(vec![Direction::Right, Direction::Right])];
    view.cells = vec![
        (Position::new(1, 0), Cell::Empty),
        (Position::new(2, 0), Cell::Burrow),
    ];
    view.add_entity(
        40,
        Species::Carnivore,
        60,
        Steps(vec![Direction::Up]),
        false,
    );
    let routes = route_baselines(Position::new(0, 0), &life(), &view);
    let contact = &routes[0]
        .selected_threat
        .as_ref()
        .unwrap()
        .static_contact_geometry;
    assert_eq!(contact.minimum_current_clearance, Some(1));
    assert_eq!(contact.route_cells_currently_within_one_cell, Some(1));
    assert_eq!(
        routes[0].damage,
        DamageBaseline::unknown(Some(&Species::Carnivore))
    );
}

#[test]
fn missing_observed_route_cell_is_preserved_as_unknown() {
    let mut view = Perception::empty();
    view.shelters = vec![Steps(vec![Direction::Right, Direction::Right])];
    view.cells = vec![(Position::new(2, 0), Cell::Burrow)];
    let routes = route_baselines(Position::new(0, 0), &life(), &view);
    assert_eq!(routes.len(), 1);
    assert_eq!(
        routes[0].observed_route_cells,
        vec![None, Some(refuge_risk_baseline::ObservedCellKind::Burrow)]
    );
    assert_eq!(routes[0].effort.nominal_paid_movement_energy, Some(2));
    assert_eq!(routes[0].damage.expected_threat_damage, None);
}
