use curious::{
    creatures::herbivore::HerbivoreEntity,
    entity::{
        Entity, behavior, decision::Reason, intent::Intent, perception::Perception,
        species::Species,
    },
    map::movement::{Direction, Steps},
    random::Random,
};
fn threatened() -> Perception {
    let mut p = Perception::empty();
    p.add_entity(
        9,
        Species::Carnivore,
        200,
        Steps(vec![Direction::Left]),
        false,
    );
    p.add_direction(Direction::Down, 3);
    p.add_direction(Direction::Right, 3);
    p
}
#[test]
fn a_known_occupied_refuge_is_skipped_for_a_visible_empty_one() {
    let mut p = threatened();
    p.shelters = vec![
        Steps(vec![Direction::Right]),
        Steps(vec![Direction::Down; 2]),
    ];
    p.add_entity(
        2,
        Species::Herbivore,
        100,
        Steps(vec![Direction::Right]),
        false,
    );
    let e = HerbivoreEntity::default();
    let d = behavior::decide(Species::Herbivore, e.life(), p, &mut Random::new(1));
    assert_eq!(d.reason, Reason::SeekShelter);
    assert_eq!(
        d.intent,
        Intent::Move {
            steps: Steps(vec![Direction::Down; 2])
        }
    );
}
#[test]
fn occupancy_is_compared_by_endpoint_not_path_spelling() {
    let mut p = threatened();
    p.shelters = vec![
        Steps(vec![Direction::DownRight]),
        Steps(vec![Direction::Down; 2]),
    ];
    p.add_entity(
        2,
        Species::Herbivore,
        100,
        Steps(vec![Direction::Right, Direction::Down]),
        false,
    );
    let e = HerbivoreEntity::default();
    let d = behavior::decide(Species::Herbivore, e.life(), p, &mut Random::new(1));
    assert_eq!(
        d.intent,
        Intent::Move {
            steps: Steps(vec![Direction::Down; 2])
        }
    );
}
#[test]
fn when_all_refuges_are_occupied_the_actor_uses_its_escape_decision() {
    let mut p = threatened();
    p.shelters = vec![Steps(vec![Direction::Right])];
    p.add_entity(
        2,
        Species::Herbivore,
        100,
        Steps(vec![Direction::Right]),
        false,
    );
    let e = HerbivoreEntity::default();
    let d = behavior::decide(Species::Herbivore, e.life(), p, &mut Random::new(1));
    assert_eq!(d.reason, Reason::EscapeThreat);
    assert_eq!(d.intent, Intent::Flee { target_id: 9 });
}

#[test]
fn the_empty_refuge_is_reached_in_the_real_resolver_with_the_real_cost() {
    use curious::{
        entity::controller::EntityController,
        map::{cell::Cell, movement::Position},
        scenario::Scenario,
    };
    let scenario = serde_json::json!({"name":"occupied refuge","bounds":[-3,3,-3,3],"founders":[
        {"species":"Herbivore","pos":{"x":0,"y":0}},
        {"species":"Herbivore","pos":{"x":1,"y":0}},
        {"species":"Carnivore","pos":{"x":-1,"y":0}}],"sites":[],"resources":[[{"x":1,"y":0},"Burrow"],[{"x":0,"y":2},"Burrow"]]});
    let mut w = serde_json::from_value::<Scenario>(scenario)
        .unwrap()
        .build(1)
        .unwrap();
    w.entities[1].controller = EntityController::Player(1);
    w.entities[2].controller = EntityController::Player(2);
    let events = w.tick();
    assert_eq!(w.entities[0].pos, Position::new(0, 2));
    assert!(matches!(w.map.cell(w.entities[0].pos), Some(Cell::Burrow)));
    assert!(events.iter().any(|e| matches!(
        e,
        curious::simulation::SimulationEvent::EntityMoved {
            id: 1,
            energy_cost: 2,
            ..
        }
    )));
    assert!(!events.iter().any(|e| matches!(
        e,
        curious::simulation::SimulationEvent::ActionRejected { id: 1, .. }
    )));
}
