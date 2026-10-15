use curious::{
    creatures::herbivore::HerbivoreEntity,
    entity::{controller::EntityController, intent::Intent, phase::EntityPhase},
    map::{
        Map,
        cell::Cell,
        movement::{Direction, Position, Steps},
    },
    persistence::State,
    simulation::SimulationEvent,
    world::{EntitySlot, World},
};
#[test]
fn walls_survive_checkpoint_and_diagonal_corners_are_blocked() {
    let mut w = World::with_seed(0, 3, 0, 3, vec![], 1);
    w.map = Map::new(0, 3, 0, 3);
    w.map.set_cell(Position::new(1, 0), Cell::Wall);
    w.map.set_cell(Position::new(0, 1), Cell::Wall);
    assert!(!w.map.can_step(Position::new(0, 0), Position::new(1, 1)));
    let resumed = State::capture(&w).restore().unwrap();
    assert_eq!(resumed.map.cell(Position::new(1, 0)), Some(&Cell::Wall));
    assert_eq!(
        resumed
            .map
            .paths_within(Position::new(0, 0), 3, &Default::default())
            .len(),
        1
    );
}
#[test]
fn resolution_cannot_pass_through_a_competitor_who_just_moved() {
    let entities = vec![(1, 0, 1), (2, 2, 0)]
        .into_iter()
        .map(|(id, x, y)| {
            EntitySlot::new(
                id,
                Position::new(x, y),
                EntityPhase::Active,
                EntityController::Player(0),
                Box::new(HerbivoreEntity::default()),
            )
        })
        .collect();
    let mut w = World::with_seed(0, 4, 0, 4, entities, 42);
    w.map = Map::new(0, 4, 0, 4);
    w.environment = Default::default();
    // At tick one rotating priority puts actor 2 first: its new cell blocks actor 1's intermediate step.
    w.push_command(
        1,
        Intent::Move {
            steps: Steps(vec![Direction::Right; 3]),
        },
    );
    w.push_command(
        2,
        Intent::Move {
            steps: Steps(vec![Direction::Down]),
        },
    );
    let events = w.tick();
    assert_eq!(w.entities[0].pos, Position::new(1, 1));
    assert_eq!(w.entities[1].pos, Position::new(2, 1));
    assert_eq!(w.entities[0].base.life().energy, 78);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, SimulationEvent::ActionRejected { id: 1, .. }))
    );
}
