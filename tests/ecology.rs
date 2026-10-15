use curious::{
    creatures::{carnivore::CarnivoreEntity, herbivore::HerbivoreEntity, omnivore::OmnivoreEntity},
    entity::{
        Entity,
        controller::EntityController,
        instinct::{Instinct, InstinctEvaluator},
        intent::Intent,
        perception::Perception,
        phase::EntityPhase,
        species::Species,
    },
    map::{Map, movement::Position},
    persistence::State,
    simulation::SimulationEvent,
    simulation::event::{DeathCause, RejectionReason},
    systems::perception::PerceptionSystem,
    world::{EntitySlot, World},
};
fn slot(id: usize, x: isize, base: Box<dyn Entity>) -> EntitySlot {
    EntitySlot::new(
        id,
        Position::new(x, 0),
        EntityPhase::Active,
        EntityController::Player(0),
        base,
    )
}
fn world(entities: Vec<EntitySlot>) -> World {
    let mut w = World::with_seed(-8, 8, -8, 8, entities, 42);
    w.map = Map::new(-8, 8, -8, 8);
    w.logger = curious::logger::Logger::disabled();
    w
}
#[test]
fn offspring_does_not_inherit_parent_cooldown_or_injury_cause() {
    let parents: Vec<Box<dyn Entity>> = vec![
        Box::new(HerbivoreEntity::default()),
        Box::new(CarnivoreEntity::default()),
        Box::new(OmnivoreEntity::default()),
    ];
    for mut parent in parents {
        parent.life_mut().on_reproduce();
        parent.life_mut().moves_used = 2;
        parent.life_mut().death_cause = Some(DeathCause::Combat);
        let child = parent.reproduce();
        assert_eq!(child.life().age, 0);
        assert_eq!(child.life().reproduction_cooldown, 0);
        assert_eq!(child.life().moves_used, 0);
        assert_eq!(child.life().death_cause, None);
    }
}
#[test]
fn injury_does_not_prevent_thirst_decision() {
    let mut e = HerbivoreEntity::default();
    e.life_mut().health = 1;
    e.life_mut().water = 0;
    assert_eq!(
        InstinctEvaluator::evaluate(e.life(), &Perception::empty(), Species::Herbivore).instinct,
        Instinct::Thirst
    );
}
#[test]
fn sleeping_cannot_stop_metabolism_or_hide_a_death() {
    let mut s = slot(1, 0, Box::new(HerbivoreEntity::default()));
    s.phase = EntityPhase::Sleeping { remaining: 10 };
    s.base.life_mut().water = 0;
    s.base.life_mut().health = 1;
    let mut w = world(vec![s]);
    let events = w.tick();
    assert!(matches!(w.entities[0].phase, EntityPhase::Corpse { .. }));
    assert_eq!(w.entities[0].base.life().age, 1);
    assert!(events.iter().any(|e| matches!(
        e,
        SimulationEvent::EntityDied {
            cause: Some(DeathCause::Dehydration),
            ..
        }
    )));
    assert!(w.last_intents.is_empty());
    assert!(
        !w.tick()
            .iter()
            .any(|e| matches!(e, SimulationEvent::EntityDied { .. }))
    );
}
#[test]
fn dead_entities_are_not_perceived_as_threats_or_mates() {
    let mut dead = slot(2, 1, Box::new(CarnivoreEntity::default()));
    dead.phase = EntityPhase::Corpse { remaining: 5 };
    dead.base.life_mut().health = 0;
    let w = world(vec![slot(1, 0, Box::new(HerbivoreEntity::default())), dead]);
    let perception = PerceptionSystem::build(&w, &w.entities[0]);
    assert!(perception.entities.is_empty());
}
#[test]
fn mating_validates_species_and_reports_rejection() {
    let mut w = world(vec![
        slot(1, 0, Box::new(HerbivoreEntity::default())),
        slot(2, 1, Box::new(CarnivoreEntity::default())),
    ]);
    w.push_command(1, Intent::Mate { target_id: 2 });
    let events = w.tick();
    assert!(events.iter().any(|e| matches!(
        e,
        SimulationEvent::ActionRejected {
            reason: RejectionReason::DifferentSpecies,
            ..
        }
    )));
    assert_eq!(w.entities.len(), 2);
}
#[test]
fn simultaneous_mating_creates_one_young_child() {
    let mut w = world(vec![
        slot(1, 0, Box::new(HerbivoreEntity::default())),
        slot(2, 1, Box::new(HerbivoreEntity::default())),
    ]);
    for s in &mut w.entities {
        s.base.life_mut().age = 30;
    }
    w.push_command(1, Intent::Mate { target_id: 2 });
    w.push_command(2, Intent::Mate { target_id: 1 });
    let events = w.tick();
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, SimulationEvent::EntityBorn { .. }))
            .count(),
        1
    );
    let child = &w.entities[2];
    assert_eq!(child.base.life().age, 0);
    assert_eq!(child.base.life().reproduction_cooldown, 0);
    assert!(events.iter().any(|e| matches!(
        e,
        SimulationEvent::ActionRejected {
            reason: RejectionReason::Cooldown,
            ..
        }
    )));
}
#[test]
fn independent_worlds_and_checkpoints_have_independent_random_streams() {
    let make = || {
        let mut w = World::with_seed(
            -8,
            8,
            -8,
            8,
            vec![EntitySlot::new(
                1,
                Position::new(0, 0),
                EntityPhase::Active,
                EntityController::AI,
                Box::new(HerbivoreEntity::default()),
            )],
            27,
        );
        w.logger = curious::logger::Logger::disabled();
        w
    };
    let mut a = make();
    let mut b = make();
    let mut other = world(vec![slot(7, 0, Box::new(OmnivoreEntity::default()))]);
    for _ in 0..30 {
        let a_events = a.tick();
        other.tick();
        let b_events = b.tick();
        assert_eq!(
            serde_json::to_value(a_events).unwrap(),
            serde_json::to_value(b_events).unwrap()
        );
        assert_eq!(
            serde_json::to_value(State::capture(&a)).unwrap(),
            serde_json::to_value(State::capture(&b)).unwrap()
        );
    }
    let mut resumed = State::capture(&a).restore().unwrap();
    for _ in 0..30 {
        a.tick();
        other.tick();
        resumed.tick();
        assert_eq!(
            serde_json::to_value(State::capture(&a)).unwrap(),
            serde_json::to_value(State::capture(&resumed)).unwrap()
        );
    }
}
