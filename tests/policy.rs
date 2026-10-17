use curious::{
    creatures::carnivore::CarnivoreEntity,
    entity::{
        Entity,
        decision::{Decision, Reason},
        intent::Intent,
        perception::Perception,
        species::Species,
    },
    map::movement::{Direction, Steps},
    policy::prepare_reserve,
    scenario::Scenario,
    simulation::checkpoint::State,
};

#[test]
fn preparation_never_leaves_a_nearby_mate_for_a_distant_or_insufficient_meal() {
    let mut actor = CarnivoreEntity::default();
    let life = actor.life_mut();
    life.max_energy = 270;
    life.energy = 220;
    life.max_meal = 60;
    life.reproduction_reserve_per_mille = 885;
    life.age = 30;
    let mut view = Perception::empty();
    view.add_entity(
        2,
        Species::Carnivore,
        100,
        Steps(vec![Direction::Down]),
        true,
    );
    view.add_food(60, true, Steps(vec![Direction::Right; 5]));
    let base = || Decision::new(Intent::Idle { duration: 1 }, Reason::Rest);
    assert_eq!(
        prepare_reserve(Species::Carnivore, life, &view, base()).reason,
        Reason::Rest
    );
    view.foods.clear();
    view.add_food(18, true, Steps(vec![Direction::Right]));
    assert_eq!(
        prepare_reserve(Species::Carnivore, life, &view, base()).reason,
        Reason::Rest
    );
    view.foods[0].amount = 19;
    assert_eq!(
        prepare_reserve(Species::Carnivore, life, &view, base()).reason,
        Reason::PrepareReproduction
    );
    view.entities.clear();
    assert_eq!(
        prepare_reserve(Species::Carnivore, life, &view, base()).reason,
        Reason::Rest
    );
}

#[test]
fn settings_are_opt_in_and_preserved_only_when_explicitly_saved() {
    let mut w = serde_json::from_str::<Scenario>(include_str!("../scenarios/mixed-refuge.json"))
        .unwrap()
        .build(1)
        .unwrap();
    assert!(!w.policy.reserve_opportunities);
    let mut legacy = serde_json::to_value(State::capture(&w)).unwrap();
    legacy["version"] = 10.into();
    legacy.as_object_mut().unwrap().remove("policy");
    assert!(
        !serde_json::from_value::<State>(legacy)
            .unwrap()
            .restore()
            .unwrap()
            .policy
            .reserve_opportunities
    );
    w.policy.reserve_opportunities = true;
    let mut resumed = State::capture(&w).restore().unwrap();
    assert!(resumed.policy.reserve_opportunities);
    w.tick();
    resumed.tick();
    assert_eq!(
        serde_json::to_value(State::capture(&w)).unwrap(),
        serde_json::to_value(State::capture(&resumed)).unwrap()
    );
}

#[test]
fn adjacent_mature_partners_can_prepare_before_either_is_reproductively_ready() {
    let mut actor = CarnivoreEntity::default();
    let life = actor.life_mut();
    life.max_energy = 270;
    life.energy = 220;
    life.max_meal = 60;
    life.reproduction_reserve_per_mille = 885;
    life.age = 30;
    let mut view = Perception::empty();
    view.add_entity(
        2,
        Species::Carnivore,
        100,
        Steps(vec![Direction::Down]),
        false,
    );
    view.entities[0].mature = true;
    view.add_food(40, true, Steps(vec![Direction::Right]));
    let base = || Decision::new(Intent::Idle { duration: 1 }, Reason::Rest);
    assert!(!life.can_reproduce());
    assert!(!view.entities[0].reproductive);
    assert_eq!(
        prepare_reserve(Species::Carnivore, life, &view, base()).reason,
        Reason::PrepareReproduction
    );
    view.entities[0].mature = false;
    assert_eq!(
        prepare_reserve(Species::Carnivore, life, &view, base()).reason,
        Reason::Rest
    );
}
