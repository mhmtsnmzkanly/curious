use curious::{
    creatures::{carnivore::CarnivoreEntity, herbivore::HerbivoreEntity},
    entity::{controller::EntityController, intent::Intent, phase::EntityPhase},
    map::{
        Map,
        cell::Cell,
        movement::{Direction, Position, Steps},
    },
    persistence::State,
    simulation::{SimulationEvent, event::RejectionReason},
    world::{EntitySlot, World},
};
#[test]
fn shelter_is_a_physical_capability_not_a_population_or_health_bonus() {
    let actors = vec![
        EntitySlot::new(
            1,
            Position::new(0, 0),
            EntityPhase::Active,
            EntityController::Player(0),
            Box::new(CarnivoreEntity::default()),
        ),
        EntitySlot::new(
            2,
            Position::new(1, 0),
            EntityPhase::Active,
            EntityController::Player(0),
            Box::new(HerbivoreEntity::default()),
        ),
    ];
    let mut w = World::with_seed(0, 3, 0, 3, actors, 1);
    w.map = Map::new(0, 3, 0, 3);
    w.map.set_cell(Position::new(1, 0), Cell::Burrow);
    w.environment = Default::default();
    w.push_command(1, Intent::Attack { target_id: 2 });
    let events = w.tick();
    assert_eq!(w.entities[1].base.life().health, 120);
    assert_eq!(w.entities[1].base.life().energy, 79);
    assert!(events.iter().any(|e| matches!(
        e,
        SimulationEvent::ActionRejected {
            reason: RejectionReason::NoPath,
            ..
        }
    )));
    assert!(
        !w.map
            .can_step_for(Position::new(0, 0), Position::new(1, 0), false)
    );
    assert!(
        w.map
            .can_step_for(Position::new(0, 0), Position::new(1, 0), true)
    );
    let restored = State::capture(&w).restore().unwrap();
    assert_eq!(restored.map.cell(Position::new(1, 0)), Some(&Cell::Burrow));
    w.push_command(
        2,
        Intent::Move {
            steps: Steps(vec![Direction::Right]),
        },
    );
    w.tick();
    assert_eq!(w.entities[1].pos, Position::new(2, 0));
    w.entities[0].pos = Position::new(1, 0);
    w.map.clear_cell(Position::new(1, 0));
    w.push_command(1, Intent::Attack { target_id: 2 });
    assert!(
        w.tick()
            .iter()
            .any(|e| matches!(e, SimulationEvent::EntityAttacked { .. }))
    );
}
