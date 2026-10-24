use curious::{
    creatures::{carnivore::CarnivoreEntity, herbivore::HerbivoreEntity},
    entity::{
        Entity, behavior, controller::EntityController, decision::Reason, intent::Intent,
        perception::Perception, phase::EntityPhase, species::Species,
    },
    map::{
        Map,
        movement::{Direction, Position, Steps},
    },
    persistence::State,
    random::Random,
    scenario::Scenario,
    simulation::{SimulationEvent, event::RejectionReason},
    world::{EntitySlot, World},
};
fn pair() -> World {
    let slots = vec![
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
    let mut w = World::with_seed(0, 8, 0, 0, slots, 1);
    w.map = Map::new(0, 8, 0, 0);
    w.environment = Default::default();
    w.logger = curious::logger::Logger::disabled();
    w
}
#[test]
fn attacks_use_capabilities_and_cannot_bypass_fatigue() {
    let mut w = pair();
    w.entities[0].base.life_mut().stamina = 0;
    w.entities[0].base.life_mut().exertion = 1;
    w.push_command(1, Intent::Attack { target_id: 2 });
    let events = w.tick();
    assert!(events.iter().any(|e| matches!(
        e,
        SimulationEvent::ActionRejected {
            reason: RejectionReason::Fatigued,
            ..
        }
    )));
    assert_eq!(w.entities[1].base.life().health, 120);
    w.push_command(1, Intent::Attack { target_id: 2 });
    let energy = w.entities[0].base.life().energy;
    let events = w.tick();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, SimulationEvent::EntityAttacked { damage: 20, .. }))
    );
    assert_eq!(w.entities[0].base.life().energy, energy - 4);
    assert_eq!(w.entities[0].base.life().exertion, 2);
}
#[test]
fn resting_recovers_stamina_without_creating_food_energy() {
    let mut w = pair();
    w.entities[0].base.life_mut().stamina = 0;
    w.entities[0].base.life_mut().exertion = 0;
    let energy = w.entities[0].base.life().energy;
    w.tick();
    assert_eq!(w.entities[0].base.life().stamina, 4);
    assert_eq!(w.entities[0].base.life().energy, energy - 1);
    w.entities[0].base.life_mut().exertion = 2;
    w.tick();
    assert_eq!(w.entities[0].base.life().stamina, 5);
}
#[test]
fn hunting_can_be_abandoned_for_body_limits() {
    let mut carn = CarnivoreEntity::default();
    let mut view = Perception::empty();
    view.add_entity(
        2,
        Species::Herbivore,
        200,
        Steps(vec![Direction::Right]),
        false,
    );
    carn.life_mut().energy = 30;
    carn.life_mut().stamina = 0;
    let mut rng = Random::new(1);
    assert_eq!(
        behavior::decide(Species::Carnivore, carn.life(), view.clone(), &mut rng).reason,
        Reason::RecoverStamina
    );
    carn.life_mut().energy = 2;
    assert_eq!(
        behavior::decide(Species::Carnivore, carn.life(), view, &mut rng).reason,
        Reason::HuntLowEnergy
    );
}
#[test]
fn constrained_prey_can_defend_and_hunter_can_feed_after_kill() {
    let mut w = serde_json::from_str::<Scenario>(include_str!("../scenarios/hunt-corridor.json"))
        .unwrap()
        .build(1)
        .unwrap();
    let mut killed = false;
    let mut fed = false;
    let mut defended = false;
    for _ in 0..50 {
        let events = w.tick();
        killed |= events.iter().any(|e| {
            matches!(
                e,
                SimulationEvent::EntityDied {
                    cause: Some(curious::simulation::event::DeathCause::Combat),
                    ..
                }
            )
        });
        fed |= killed
            && events
                .iter()
                .any(|e| matches!(e, SimulationEvent::EntityAte { id: 1, .. }));
        defended |= w
            .last_decisions
            .iter()
            .any(|d| d.id == 2 && d.decision.reason == Reason::CorneredDefense);
    }
    assert!(killed && fed && defended);
}
#[test]
fn version_two_bodies_receive_explicit_stamina_defaults() {
    let mut state = serde_json::to_value(State::capture(&pair())).unwrap();
    state["version"] = 2.into();
    for e in state["entities"].as_array_mut().unwrap() {
        for key in ["stamina", "max_stamina", "exertion"] {
            e["life"].as_object_mut().unwrap().remove(key);
        }
    }
    let w = serde_json::from_value::<State>(state)
        .unwrap()
        .restore()
        .unwrap();
    assert_eq!(w.entities[0].base.life().stamina, 16);
    assert_eq!(w.entities[0].base.life().max_stamina, 16);
    assert_eq!(w.entities[0].base.life().exertion, 0);
    assert_eq!(State::capture(&w).version, 18);
}

#[test]
fn local_mating_opportunity_obeys_body_safety_limits() {
    use curious::entity::instinct::{Instinct, InstinctEvaluator};
    let mut herb = HerbivoreEntity::default();
    herb.life_mut().age = 30;
    herb.life_mut().energy = herb.life().max_energy / 2;
    herb.life_mut().water = herb.life().max_water / 2;
    let mut view = Perception::empty();
    view.add_entity(
        2,
        Species::Herbivore,
        200,
        Steps(vec![Direction::Right]),
        true,
    );
    assert_eq!(
        InstinctEvaluator::evaluate(herb.life(), &view, Species::Herbivore).instinct,
        Instinct::Mating
    );
    herb.life_mut().water = 0;
    assert_eq!(
        InstinctEvaluator::evaluate(herb.life(), &view, Species::Herbivore).instinct,
        Instinct::Thirst
    );
    herb.life_mut().water = 60;
    view.add_entity(
        3,
        Species::Carnivore,
        200,
        Steps(vec![Direction::Left]),
        false,
    );
    assert_eq!(
        InstinctEvaluator::evaluate(herb.life(), &view, Species::Herbivore).instinct,
        Instinct::Threat
    );
}

#[test]
fn healing_does_not_spend_a_hungry_juveniles_food_reserve() {
    let mut w = pair();
    let life = w.entities[0].base.life_mut();
    life.health = 70;
    life.energy = 44;
    w.tick();
    assert_eq!(w.entities[0].base.life().energy, 43);
    assert_eq!(w.entities[0].base.life().health, 70);
    w.entities[0].base.life_mut().energy = 90;
    w.tick();
    assert_eq!(w.entities[0].base.life().health, 71);
    assert_eq!(w.entities[0].base.life().energy, 87);
}

#[test]
fn hunting_prefers_an_affordable_wounded_prey() {
    let carn = CarnivoreEntity::default();
    let mut view = Perception::empty();
    view.add_entity(
        2,
        Species::Herbivore,
        200,
        Steps(vec![Direction::Right]),
        false,
    );
    view.entities[0].health = Some(120);
    view.add_entity(
        3,
        Species::Herbivore,
        120,
        Steps(vec![Direction::Left, Direction::Left]),
        false,
    );
    view.entities[1].health = Some(20);
    let d = curious::entity::hunting::decide(Species::Carnivore, carn.life(), &view).unwrap();
    assert_eq!(
        d.intent,
        Intent::Move {
            steps: Steps(vec![Direction::Left, Direction::Left])
        }
    );
    let mut low = CarnivoreEntity::default();
    low.life_mut().energy = 10;
    view.entities.remove(1);
    view.entities[0].steps = Steps(vec![Direction::Right; 3]);
    assert_eq!(
        curious::entity::hunting::decide(Species::Carnivore, low.life(), &view)
            .unwrap()
            .reason,
        Reason::HuntTooCostly
    );
}
