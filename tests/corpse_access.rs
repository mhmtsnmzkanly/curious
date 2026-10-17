use curious::{
    entity::{intent::Intent, phase::EntityPhase},
    map::movement::{Direction, Position, Steps},
    scenario::Scenario,
    simulation::{SimulationEvent, checkpoint::State},
    systems::perception::PerceptionSystem,
    world::World,
};
fn world(target: Position) -> World {
    let s = serde_json::json!({"name":"interaction access","bounds":[-5,5,-5,5],"founders":[{"species":"Carnivore","pos":{"x":0,"y":0}},{"species":"Herbivore","pos":target}],"sites":[],"resources":[[target,"Burrow"]]});
    let mut w = serde_json::from_value::<Scenario>(s)
        .unwrap()
        .build(7)
        .unwrap();
    w.entities[0].base.life_mut().energy = 10;
    w.entities[1].base.life_mut().health = 0;
    w.entities[1].phase = EntityPhase::Corpse { remaining: 100 };
    w.entities[1].corpse_energy = 60;
    w
}
#[test]
fn a_scavenger_can_sense_and_eat_an_adjacent_refuge_corpse_without_entering() {
    let mut w = world(Position::new(1, 0));
    let p = PerceptionSystem::build(&w, &w.entities[0]);
    let f = p
        .foods
        .iter()
        .find(|f| f.corpse_id == Some(2))
        .expect("physically edible corpse must be sensed");
    assert_eq!(f.steps, Steps(vec![Direction::Right]));
    let events = w.tick();
    assert_eq!(w.entities[0].pos, Position::new(0, 0));
    assert!(events.iter().any(|e| matches!(
        e,
        SimulationEvent::EntityAte {
            id: 1,
            amount: 10,
            ..
        }
    )));
    assert_eq!(w.entities[1].corpse_energy, 50);
}
#[test]
fn autonomous_sensed_meal_matches_the_existing_forced_physical_action() {
    let mut a = world(Position::new(1, 0));
    let mut b = State::capture(&a).restore().unwrap();
    b.command_inputs.insert(
        1,
        Intent::Eat {
            at: Steps(vec![Direction::Right]),
            corpse_id: Some(2),
        },
    );
    a.tick();
    b.tick();
    assert_eq!(
        serde_json::to_value(State::capture(&a)).unwrap(),
        serde_json::to_value(State::capture(&b)).unwrap()
    );
}
#[test]
fn live_refuge_occupants_are_still_hidden_and_cannot_be_attacked() {
    let mut w = world(Position::new(1, 0));
    w.entities[1].phase = EntityPhase::Active;
    w.entities[1].base.life_mut().health = 120;
    w.entities[1].corpse_energy = 0;
    w.entities[1].controller = curious::entity::controller::EntityController::Player(1);
    let p = PerceptionSystem::build(&w, &w.entities[0]);
    assert!(p.entities.is_empty());
    assert!(p.foods.is_empty());
    w.command_inputs.insert(1, Intent::Attack { target_id: 2 });
    assert!(w.tick().iter().any(|e| matches!(
        e,
        SimulationEvent::ActionRejected {
            id: 1,
            reason: curious::simulation::event::RejectionReason::NoPath,
            ..
        }
    )));
}
#[test]
fn sensing_is_radius_bounded_and_does_not_cross_an_unreachable_boundary() {
    let mut w = world(Position::new(4, 0));
    w.entities[0].base.life_mut().vision_range = 2;
    assert!(PerceptionSystem::build(&w, &w.entities[0]).foods.is_empty());
    w.entities[0].base.life_mut().vision_range = 7;
    for y in -5..=5 {
        w.map
            .set_cell(Position::new(2, y), curious::map::cell::Cell::Wall);
    }
    assert!(PerceptionSystem::build(&w, &w.entities[0]).foods.is_empty());
}
#[test]
fn an_unwalkable_interaction_endpoint_is_not_used_as_a_path_through_a_refuge() {
    let mut w = world(Position::new(2, 0));
    for y in -5..=5 {
        w.map
            .set_cell(Position::new(1, y), curious::map::cell::Cell::Wall);
    }
    // The corpse is two cells away behind a solid barrier, not an adjacent meal.
    assert!(PerceptionSystem::build(&w, &w.entities[0]).foods.is_empty());
}
