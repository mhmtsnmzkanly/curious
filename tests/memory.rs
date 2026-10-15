use curious::{
    creatures::herbivore::HerbivoreEntity,
    entity::{
        Entity,
        decision::{Decision, Reason},
        intent::Intent,
        memory::Memory,
        perception::Perception,
        species::Species,
    },
    map::{
        cell::Cell,
        movement::{Direction, Position},
    },
    persistence::State,
    scenario::Scenario,
};
#[test]
fn observed_water_is_recalled_and_depleted_or_stale_knowledge_is_removed() {
    let origin = Position::new(0, 0);
    let target = Position::new(3, 0);
    let mut m = Memory::default();
    let mut view = Perception::empty();
    view.cells.push((target, Cell::Water { amount: 10 }));
    m.observe(origin, &view, 1, Species::Herbivore);
    let mut unseen = Perception::empty();
    unseen.add_direction(Direction::Right, 3);
    let life = HerbivoreEntity::default();
    let d = m.recall(
        origin,
        life.life(),
        &unseen,
        Decision::new(Intent::Idle { duration: 1 }, Reason::SearchWater),
    );
    assert_eq!(d.reason, Reason::RecallWater);
    unseen.cells.push((target, Cell::Empty));
    m.observe(origin, &unseen, 2, Species::Herbivore);
    assert!(m.sites.is_empty());
    m.observe(origin, &view, 3, Species::Herbivore);
    m.observe(origin, &Perception::empty(), 104, Species::Herbivore);
    assert!(m.sites.is_empty());
}
#[test]
fn knowledge_is_bounded_and_survives_restart_without_changing_the_future() {
    let mut w = serde_json::from_str::<Scenario>(include_str!("../scenarios/meadow.json"))
        .unwrap()
        .build(5)
        .unwrap();
    for _ in 0..100 {
        w.tick();
    }
    assert!(w.entities.iter().any(|e| !e.memory.sites.is_empty()));
    let mut resumed = State::capture(&w).restore().unwrap();
    for _ in 0..30 {
        w.tick();
        resumed.tick();
        assert_eq!(
            serde_json::to_value(State::capture(&w)).unwrap(),
            serde_json::to_value(State::capture(&resumed)).unwrap()
        );
    }
    let mut m = Memory::default();
    let mut view = Perception::empty();
    for x in 0..100 {
        view.cells
            .push((Position::new(x, 0), Cell::Water { amount: 1 }));
    }
    m.observe(Position::new(0, 0), &view, 1, Species::Herbivore);
    assert_eq!(m.sites.len(), 32);
}
