use curious::{
    entity::{
        decision::{Decision, Reason},
        hunt_learning::{HuntEnd, HuntLearner, HuntLedger},
        intent::Intent,
        lifestate::LifeState,
        perception::Perception,
        species::Species,
    },
    map::movement::{Direction, Steps},
    random::Random,
};
#[test]
fn two_endings_in_one_tick_are_audited_once_in_order() {
    let life = curious::creatures::carnivore::CarnivoreEntity::default().life_state;
    let mut view = Perception::empty();
    view.add_entity(
        2,
        Species::Herbivore,
        20,
        Steps(vec![Direction::Right]),
        false,
    );
    let mut l = HuntLearner {
        enabled: true,
        ..Default::default()
    };
    let mut rng = Random::new(5);
    l.choose(
        Species::Carnivore,
        &life,
        &view,
        Decision::new(Intent::Attack { target_id: 2 }, Reason::HuntPrey),
        &mut rng,
    );
    l.finish(1, &HuntLedger::default());
    assert!(l.take_results().is_empty());
    view.entities[0].id = 3;
    l.choose(
        Species::Carnivore,
        &life,
        &view,
        Decision::new(Intent::Attack { target_id: 3 }, Reason::HuntPrey),
        &mut rng,
    );
    l.finish(
        1,
        &HuntLedger {
            alive: false,
            ..Default::default()
        },
    );
    let results = l.take_results();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].end, HuntEnd::TargetLost);
    assert_eq!(results[1].end, HuntEnd::Died);
    assert_eq!(results[0].attempt.target, 2);
    assert_eq!(results[1].attempt.target, 3);
    assert!(results.iter().all(|r| r.reward.is_none()));
    assert!(l.take_results().is_empty());
    assert!(
        serde_json::to_value(&l)
            .unwrap()
            .get("pending_results")
            .is_none()
    );
}
#[test]
fn audit_never_creates_learning_or_rng_changes_when_disabled() {
    let life: LifeState = curious::creatures::carnivore::CarnivoreEntity::default().life_state;
    let mut l = HuntLearner::default();
    let before = serde_json::to_value(&l).unwrap();
    let mut rng = Random::new(42);
    let d = l.choose(
        Species::Carnivore,
        &life,
        &Perception::empty(),
        Decision::new(Intent::Idle { duration: 1 }, Reason::Rest),
        &mut rng,
    );
    l.finish(
        1,
        &HuntLedger {
            alive: false,
            ..Default::default()
        },
    );
    assert_eq!(d.reason, Reason::Rest);
    assert!(l.take_results().is_empty());
    assert_eq!(rng.state(), 42);
    assert_eq!(serde_json::to_value(l).unwrap(), before);
}
