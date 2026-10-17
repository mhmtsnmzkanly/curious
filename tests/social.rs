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
    map::movement::{Direction, Position, Steps},
    scenario::Scenario,
    simulation::checkpoint::State,
};
fn adult() -> HerbivoreEntity {
    let mut e = HerbivoreEntity::default();
    let l = e.life_mut();
    l.age = 30;
    l.energy = l.max_energy;
    l.water = l.max_water;
    e
}
fn base(reason: Reason) -> Decision {
    Decision::new(Intent::Idle { duration: 1 }, reason)
}
fn view(distance: usize, ready: bool) -> Perception {
    let mut p = Perception::empty();
    p.add_direction(Direction::Right, 6);
    p.add_direction(Direction::Left, 6);
    p.add_entity(
        2,
        Species::Herbivore,
        100,
        Steps(vec![Direction::Right; distance]),
        ready,
    );
    p.entities[0].mature = true;
    p
}
fn choose(m: &mut Memory, e: &HerbivoreEntity, p: &Perception, tick: usize, r: Reason) -> Decision {
    m.observe(Position::new(0, 0), p, tick, Species::Herbivore);
    m.social.decide(
        Position::new(0, 0),
        e.life(),
        p,
        &m.companions,
        &m.dangers,
        tick,
        base(r),
    )
}
#[test]
fn an_observed_adult_can_be_waited_for_without_claiming_it_is_ready() {
    let mut m = Memory::default();
    let e = adult();
    let p = view(1, false);
    assert_eq!(
        choose(&mut m, &e, &p, 1, Reason::NoEligibleMate).reason,
        Reason::WaitForMate
    );
    assert_eq!(m.social.target.as_ref().unwrap().id, 2);
    assert!(!p.entities[0].reproductive);
}
#[test]
fn no_invented_destination_and_real_priorities_are_preserved() {
    let mut m = Memory::default();
    let e = adult();
    assert_eq!(
        choose(&mut m, &e, &Perception::empty(), 1, Reason::NoEligibleMate).reason,
        Reason::NoEligibleMate
    );
    for r in [
        Reason::SeekFood,
        Reason::SeekWater,
        Reason::SeekShelter,
        Reason::EscapeThreat,
        Reason::RecoverStamina,
        Reason::Mate,
        Reason::CustomPolicy,
        Reason::ExternalCommand,
    ] {
        assert_eq!(choose(&mut m, &e, &view(1, false), 2, r).reason, r);
    }
}
#[test]
fn unsafe_paths_and_retry_cannot_fall_back_to_old_unsafe_recall() {
    let mut m = Memory::default();
    let e = adult();
    let p = view(4, false);
    m.observe(Position::new(0, 0), &p, 1, Species::Herbivore);
    // Diagonal danger would be missed by a Manhattan-only radius check.
    m.dangers.push((Position::new(2, 2), 1));
    let d = m.social.decide(
        Position::new(0, 0),
        e.life(),
        &p,
        &m.companions,
        &m.dangers,
        1,
        base(Reason::RecallMate),
    );
    assert_eq!(d.reason, Reason::WaitForMate);
    assert!(m.social.target.is_none());
    let d = m.social.decide(
        Position::new(0, 0),
        e.life(),
        &p,
        &m.companions,
        &m.dangers,
        2,
        base(Reason::RecallMate),
    );
    assert_eq!(d.reason, Reason::WaitForMate);
}
#[test]
fn approaching_is_one_affordable_step_and_not_an_occupied_cell() {
    let mut m = Memory::default();
    let e = adult();
    let p = view(4, false);
    let d = choose(&mut m, &e, &p, 1, Reason::Explore);
    assert_eq!(
        d.intent,
        Intent::Move {
            steps: Steps(vec![Direction::Right])
        }
    );
    let mut expensive = adult();
    expensive.life_mut().move_energy_cost = 100;
    let d = choose(&mut Memory::default(), &expensive, &p, 1, Reason::Explore);
    assert_ne!(d.reason, Reason::SeekCompanion);
    let mut blocked = p;
    blocked.entities[0].steps = Steps(vec![Direction::Right]);
    assert_eq!(
        choose(&mut m, &e, &blocked, 2, Reason::NoEligibleMate).intent,
        Intent::Idle { duration: 1 }
    );
}
#[test]
fn a_lost_contact_is_searched_locally_but_visible_refresh_does_not_reset_deadline() {
    let mut m = Memory::default();
    let e = adult();
    let p = view(1, false);
    choose(&mut m, &e, &p, 1, Reason::NoEligibleMate);
    let mut empty = Perception::empty();
    empty.add_direction(Direction::Right, 6);
    empty.add_direction(Direction::Left, 6);
    empty
        .cells
        .push((Position::new(1, 0), curious::map::cell::Cell::Empty));
    assert_eq!(
        choose(&mut m, &e, &empty, 2, Reason::NoEligibleMate).reason,
        Reason::SeekCompanion
    );
    assert!(m.companions.is_empty());
    assert!(m.social.target.as_ref().unwrap().searching);
    choose(&mut m, &e, &p, 30, Reason::NoEligibleMate);
    assert_eq!(m.social.target.as_ref().unwrap().started, 1);
    choose(&mut m, &e, &p, 31, Reason::NoEligibleMate);
    assert!(m.social.target.is_none());
    assert_eq!(m.social.retry_after, 41);
}
#[test]
fn search_does_not_start_too_late_or_keep_an_immature_actor_committed() {
    let mut m = Memory::default();
    let mut e = adult();
    e.life_mut().age = e.life().max_age - 3;
    assert_ne!(
        choose(&mut m, &e, &view(4, false), 1, Reason::NoEligibleMate).reason,
        Reason::SeekCompanion
    );
    e.life_mut().age = 30;
    choose(&mut m, &e, &view(1, false), 2, Reason::NoEligibleMate);
    e.life_mut().age = 0;
    choose(&mut m, &e, &view(1, false), 3, Reason::Explore);
    assert!(m.social.target.is_none());
}
#[test]
fn old_checkpoints_stay_opt_out_and_active_goals_resume_exactly() {
    let mut w = serde_json::from_str::<Scenario>(include_str!("../scenarios/meadow.json"))
        .unwrap()
        .build(7)
        .unwrap();
    assert!(!w.policy.mate_rendezvous);
    let mut old = serde_json::to_value(State::capture(&w)).unwrap();
    old["version"] = 11.into();
    old["policy"]["mate_rendezvous"] = true.into();
    assert!(
        !serde_json::from_value::<State>(old)
            .unwrap()
            .restore()
            .unwrap()
            .policy
            .mate_rendezvous
    );
    w.policy.mate_rendezvous = true;
    for _ in 0..40 {
        w.tick();
        if w.entities.iter().any(|e| e.memory.social.target.is_some()) {
            break;
        }
    }
    assert!(w.entities.iter().any(|e| e.memory.social.target.is_some()));
    let mut resumed = State::capture(&w).restore().unwrap();
    for _ in 0..40 {
        w.tick();
        resumed.tick();
        assert_eq!(
            serde_json::to_value(State::capture(&w)).unwrap(),
            serde_json::to_value(State::capture(&resumed)).unwrap()
        );
    }
}
#[test]
fn bounded_social_memory_rejects_corrupt_goals() {
    let mut m = Memory::default();
    choose(&mut m, &adult(), &view(1, false), 1, Reason::Explore);
    m.social.target.as_mut().unwrap().visited = vec![Position::new(0, 0); 33];
    assert!(m.validate().is_err());
}

#[test]
fn two_adults_with_different_cooldowns_stay_together_then_really_reproduce() {
    let mut scenario =
        serde_json::from_str::<Scenario>(include_str!("../scenarios/meadow.json")).unwrap();
    scenario.founders.truncate(2);
    let mut w = scenario.build(7).unwrap();
    w.policy.mate_rendezvous = true;
    for e in &mut w.entities {
        let l = e.base.life_mut();
        l.age = 30;
        l.energy = l.max_energy;
        l.water = l.max_water;
    }
    w.entities[0].base.life_mut().reproduction_cooldown = 6;
    let mut waited = false;
    let mut births = 0;
    for _ in 0..10 {
        for event in w.tick() {
            if matches!(
                event,
                curious::simulation::SimulationEvent::EntityBorn { .. }
            ) {
                births += 1;
            }
        }
        waited |= w
            .last_decisions
            .iter()
            .any(|d| d.decision.reason == Reason::WaitForMate);
    }
    assert!(waited);
    assert!(
        births > 0,
        "waiting must lead to physically resolved reproduction"
    );
}
#[test]
fn carnivore_decisions_are_unchanged_by_herbivore_rendezvous() {
    let make = || {
        let mut s =
            serde_json::from_str::<Scenario>(include_str!("../scenarios/mixed-refuge.json"))
                .unwrap();
        s.founders.retain(|e| e.species == Species::Carnivore);
        s.build(7).unwrap()
    };
    let (mut a, mut b) = (make(), make());
    b.policy.mate_rendezvous = true;
    for _ in 0..100 {
        assert_eq!(
            serde_json::to_value(a.tick()).unwrap(),
            serde_json::to_value(b.tick()).unwrap()
        );
        let mut state = serde_json::to_value(State::capture(&b)).unwrap();
        state["policy"]["mate_rendezvous"] = false.into();
        assert_eq!(serde_json::to_value(State::capture(&a)).unwrap(), state);
    }
}

#[test]
fn an_available_mate_is_not_ignored_to_wait_for_an_unready_contact() {
    let mut m = Memory::default();
    let e = adult();
    let mut p = view(1, false);
    choose(&mut m, &e, &p, 1, Reason::NoEligibleMate);
    p.add_entity(
        3,
        Species::Herbivore,
        100,
        Steps(vec![Direction::Left; 3]),
        true,
    );
    p.entities.last_mut().unwrap().mature = true;
    assert_eq!(
        choose(&mut m, &e, &p, 2, Reason::ApproachMate).reason,
        Reason::SeekCompanion
    );
    assert_eq!(m.social.target.as_ref().unwrap().id, 3);
}
