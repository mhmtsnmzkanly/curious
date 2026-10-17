use curious::{
    creatures::carnivore::CarnivoreEntity,
    entity::{Entity, controller::EntityController, phase::EntityPhase},
    map::{Map, movement::Position},
    simulation::SimulationEvent,
    world::{EntitySlot, World},
};

#[test]
fn injured_forager_eats_above_healing_threshold_and_pays_for_recovery() {
    let mut actor = CarnivoreEntity::default();
    actor.life_mut().health = 69; // Below the actual reproductive health gate.
    actor.life_mut().energy = 70; // Above the old feeding target (67).
    actor.life_mut().age = 30;
    let mut carcass = CarnivoreEntity::default();
    carcass.life_mut().health = 0;
    let mut dead = EntitySlot::new(
        2,
        Position::new(1, 0),
        EntityPhase::Corpse { remaining: 5 },
        EntityController::AI,
        Box::new(carcass),
    );
    dead.corpse_energy = 40;
    let actor = EntitySlot::new(
        1,
        Position::new(0, 0),
        EntityPhase::Active,
        EntityController::AI,
        Box::new(actor),
    );
    let mut w = World::with_seed(0, 2, 0, 2, vec![actor, dead], 1);
    w.map = Map::new(0, 2, 0, 2);
    w.environment = Default::default();
    let events = w.tick();
    assert!(events.iter().any(|e| matches!(
        e,
        SimulationEvent::EntityAte {
            id: 1,
            amount: 10,
            ..
        }
    )));
    assert_eq!(w.entities[0].base.life().energy, 77); // basal 1, healing 2, food 10
    assert_eq!(w.entities[0].base.life().health, 70);
    assert_eq!(w.entities[1].corpse_energy, 30);
}

#[test]
fn immediate_small_meal_can_outweigh_a_costly_distant_pile() {
    use curious::{
        entity::{
            behavior, decision::Reason, intent::Intent, perception::Perception, species::Species,
        },
        map::movement::{Direction, Steps},
    };
    let mut actor = CarnivoreEntity::default();
    actor.life_mut().energy = 20;
    let mut view = Perception::empty();
    view.add_food(4, true, Steps(vec![]));
    view.add_food(60, true, Steps(vec![Direction::Right; 7]));
    let d = behavior::decide(
        Species::Carnivore,
        actor.life(),
        view,
        &mut curious::random::Random::new(1),
    );
    assert_eq!(d.reason, Reason::SeekFood);
    assert!(matches!(d.intent, Intent::Eat { at, .. } if at.is_empty()));
}

#[test]
fn larger_meals_consume_the_same_finite_carcass_and_offspring_keep_body_timing() {
    use curious::{
        entity::intent::Intent,
        map::movement::{Direction, Steps},
    };
    let mut actor = CarnivoreEntity::default();
    actor.life_mut().max_meal = 30;
    actor.life_mut().energy = 10;
    actor.life_mut().reproduction_interval = 75;
    actor.life_mut().on_reproduce();
    assert_eq!(actor.life().reproduction_cooldown, 75);
    let child = actor.reproduce();
    assert_eq!(child.life().max_meal, 30);
    assert_eq!(child.life().reproduction_interval, 75);
    assert_eq!(child.life().reproduction_cooldown, 0);
    actor.life_mut().energy = 10;
    let actor = EntitySlot::new(
        1,
        Position::new(0, 0),
        EntityPhase::Active,
        EntityController::Player(0),
        Box::new(actor),
    );
    let mut carcass = CarnivoreEntity::default();
    carcass.life_mut().health = 0;
    let mut dead = EntitySlot::new(
        2,
        Position::new(1, 0),
        EntityPhase::Corpse { remaining: 5 },
        EntityController::AI,
        Box::new(carcass),
    );
    dead.corpse_energy = 40;
    let mut w = World::with_seed(0, 2, 0, 2, vec![actor, dead], 1);
    w.map = Map::new(0, 2, 0, 2);
    w.environment = Default::default();
    w.push_command(
        1,
        Intent::Eat {
            at: Steps(vec![Direction::Right]),
            corpse_id: Some(2),
        },
    );
    w.tick();
    assert_eq!(w.entities[0].base.life().energy, 39);
    assert_eq!(w.entities[1].corpse_energy, 10);
}

#[test]
fn version_eight_keeps_original_body_limits_when_new_fields_are_absent() {
    use curious::{persistence::State, scenario::Scenario};
    let w = serde_json::from_str::<Scenario>(include_str!("../scenarios/mixed-refuge.json"))
        .unwrap()
        .build(1)
        .unwrap();
    let mut value = serde_json::to_value(State::capture(&w)).unwrap();
    value["version"] = 8.into();
    for e in value["entities"].as_array_mut().unwrap() {
        e["life"].as_object_mut().unwrap().remove("max_meal");
        e["life"]
            .as_object_mut()
            .unwrap()
            .remove("reproduction_interval");
        e["life"]
            .as_object_mut()
            .unwrap()
            .remove("reproduction_reserve_per_mille");
        e["memory"].as_object_mut().unwrap().remove("companions");
        e["memory"]
            .as_object_mut()
            .unwrap()
            .remove("companions_enabled");
    }
    let restored = serde_json::from_value::<State>(value)
        .unwrap()
        .restore()
        .unwrap();
    assert!(
        restored
            .entities
            .iter()
            .all(|e| e.base.life().max_meal == 10
                && e.base.life().reproduction_interval == 25
                && e.memory.companions.is_empty())
    );
}

#[test]
fn preparation_is_not_part_of_default_hunger_and_preserves_priority() {
    use curious::entity::{
        decision::{Decision, Reason},
        instinct::{Instinct, InstinctEvaluator},
        intent::Intent,
        perception::Perception,
        species::Species,
    };
    use curious::map::movement::Steps;
    let mut actor = CarnivoreEntity::default();
    let life = actor.life_mut();
    life.max_energy = 270;
    life.max_meal = 60;
    life.energy = 220;
    life.reproduction_reserve_per_mille = 885;
    life.age = life.maturity_age;
    life.water = life.max_water;
    let mut view = Perception::empty();
    view.add_food(60, true, Steps(vec![]));
    view.add_entity(2, Species::Carnivore, 100, Steps(vec![]), true);
    assert_eq!(life.reproduction_energy_required(), 238);
    assert_eq!(
        InstinctEvaluator::evaluate(life, &view, Species::Carnivore).instinct,
        Instinct::Idle
    );
    let base = || Decision::new(Intent::Idle { duration: 1 }, Reason::Rest);
    assert_eq!(
        curious::policy::prepare_reserve(Species::Carnivore, life, &view, base()).reason,
        Reason::PrepareReproduction
    );
    for reason in [
        Reason::SeekWater,
        Reason::EscapeThreat,
        Reason::SeekFood,
        Reason::ExternalCommand,
        Reason::CustomPolicy,
    ] {
        let decision = Decision::new(Intent::Idle { duration: 1 }, reason);
        assert_eq!(
            curious::policy::prepare_reserve(Species::Carnivore, life, &view, decision).reason,
            reason
        );
    }
    life.reproduction_cooldown = 1;
    assert_eq!(
        curious::policy::prepare_reserve(Species::Carnivore, life, &view, base()).reason,
        Reason::Rest
    );
    life.reproduction_cooldown = 0;
    life.age = 0;
    assert_eq!(
        curious::policy::prepare_reserve(Species::Carnivore, life, &view, base()).reason,
        Reason::Rest
    );
}

#[test]
fn adult_uses_existing_carcass_for_reserve_but_does_not_start_extra_hunt() {
    use curious::entity::{
        behavior, decision::Reason, intent::Intent, perception::Perception, species::Species,
    };
    use curious::map::movement::{Direction, Steps};
    let mut actor = CarnivoreEntity::default();
    let life = actor.life_mut();
    life.max_energy = 270;
    life.energy = 220;
    life.max_meal = 60;
    life.reproduction_reserve_per_mille = 885;
    life.age = 30;
    let mut prey = Perception::empty();
    prey.add_entity(
        9,
        Species::Herbivore,
        50,
        Steps(vec![Direction::Right]),
        false,
    );
    let decision = behavior::decide(
        Species::Carnivore,
        actor.life(),
        prey,
        &mut curious::random::Random::new(1),
    );
    assert!(!matches!(decision.intent, Intent::Attack { .. }));
    assert_ne!(decision.reason, Reason::HuntPrey);
    let mut dead = EntitySlot::new(
        2,
        Position::new(1, 0),
        EntityPhase::Corpse { remaining: 5 },
        EntityController::AI,
        Box::new(CarnivoreEntity::default()),
    );
    dead.base.life_mut().health = 0;
    dead.corpse_energy = 40;
    let actor = EntitySlot::new(
        1,
        Position::new(0, 0),
        EntityPhase::Active,
        EntityController::AI,
        Box::new(actor),
    );
    let mut partner = CarnivoreEntity::default();
    partner.life_mut().age = 30;
    let partner = EntitySlot::new(
        3,
        Position::new(0, 1),
        EntityPhase::Active,
        EntityController::Player(0),
        Box::new(partner),
    );
    let mut w = World::with_seed(0, 2, 0, 2, vec![actor, dead, partner], 1);
    w.map = Map::new(0, 2, 0, 2);
    w.environment = Default::default();
    assert!(!w.policy.reserve_opportunities);
    let mut control = curious::simulation::checkpoint::State::capture(&w)
        .restore()
        .unwrap();
    let control_events = control.tick();
    assert!(
        !control_events
            .iter()
            .any(|e| matches!(e, SimulationEvent::EntityAte { id: 1, .. }))
    );
    w.policy.reserve_opportunities = true;
    let events = w.tick();
    assert_eq!(
        w.last_decisions
            .iter()
            .find(|d| d.id == 1)
            .unwrap()
            .decision
            .reason,
        Reason::PrepareReproduction
    );
    assert!(events.iter().any(|e| matches!(
        e,
        SimulationEvent::EntityAte {
            id: 1,
            amount: 40,
            ..
        }
    )));
    assert!(w.entities[0].base.life().can_reproduce());
    assert_eq!(w.entities[1].corpse_energy, 0);
    let events = w.tick();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, SimulationEvent::EntityBorn { .. }))
    );
}
