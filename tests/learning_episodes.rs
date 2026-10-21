use curious::{
    creatures::herbivore::HerbivoreEntity,
    entity::{
        controller::EntityController,
        decision::{Decision, Reason},
        intent::Intent,
        learning::{Learner, MAX_WATER_TICKS, WaterEnd, WaterEpisode},
        perception::Perception,
        phase::EntityPhase,
    },
    map::{Map, cell::Cell, movement::Position},
    random::Random,
    simulation::{SimulationEvent, checkpoint::State, event::RejectionReason},
    world::{EntitySlot, World},
};
fn water_world() -> World {
    let mut herb = HerbivoreEntity::default();
    herb.life_state.speed = 1;
    herb.life_state.water = 10;
    let mut w = World::with_seed(
        0,
        6,
        0,
        6,
        vec![EntitySlot::new(
            1,
            Position::new(0, 0),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(herb),
        )],
        42,
    );
    w.logger = curious::logger::Logger::disabled();
    w.environment = Default::default();
    w.map = Map::new(0, 6, 0, 6);
    w.map
        .set_cell(Position::new(5, 0), Cell::Water { amount: 30 });
    w.map
        .set_cell(Position::new(0, 5), Cell::Water { amount: 30 });
    w
}
fn attempting() -> Learner {
    let mut learner = Learner::default();
    learner.episode = Some(WaterEpisode {
        target: Position::new(5, 0),
        ticks: 0,
        movement_energy: 0,
        incoming_damage: 0,
    });
    learner
}
#[test]
fn slow_trip_is_one_experience_and_resumes_identically() {
    let mut w = water_world();
    let events = w.tick();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, SimulationEvent::EntityMoved { energy_cost: 1, .. }))
    );
    assert!(w.entities[0].learner.values.is_empty());
    let target = w.entities[0].learner.episode.as_ref().unwrap().target;
    let mut resumed = State::capture(&w).restore().unwrap();
    for _ in 0..3 {
        w.tick();
        resumed.tick();
        assert_eq!(
            serde_json::to_value(State::capture(&w)).unwrap(),
            serde_json::to_value(State::capture(&resumed)).unwrap()
        );
    }
    let l = &w.entities[0].learner;
    assert!(l.episode.is_none());
    assert_eq!(l.values.len(), 1);
    assert_eq!(l.values[0].visits, 1);
    assert_eq!(l.values[0].pos, target);
    let result = l.last_episode.as_ref().unwrap();
    assert_eq!(result.end, WaterEnd::Reached);
    assert_eq!(result.attempt.ticks, 4);
    assert_eq!(result.attempt.movement_energy, 4);
    assert_eq!(result.water, 10);
    assert_eq!(result.reward, Some(88));
}
#[test]
fn accounting_uses_actor_and_target_events_not_net_body_deltas() {
    let mut learner = attempting();
    learner.finish(
        1,
        true,
        &[
            SimulationEvent::EntityMoved {
                id: 1,
                from: Position::new(0, 0),
                to: Position::new(1, 0),
                energy_cost: 3,
            },
            SimulationEvent::EntityMoved {
                id: 9,
                from: Position::new(0, 0),
                to: Position::new(1, 0),
                energy_cost: 100,
            },
            SimulationEvent::EntityAttacked {
                energy_cost: 1,
                killed: false,
                attacker: 9,
                target: 1,
                damage: 7,
            },
            SimulationEvent::EntityDrank {
                id: 1,
                pos: Position::new(0, 5),
                amount: 10,
            },
        ],
    );
    assert!(learner.values.is_empty());
    learner.finish(
        1,
        true,
        &[
            SimulationEvent::EntityMoved {
                id: 1,
                from: Position::new(1, 0),
                to: Position::new(2, 0),
                energy_cost: 2,
            },
            SimulationEvent::EntityDrank {
                id: 1,
                pos: Position::new(5, 0),
                amount: 10,
            },
        ],
    );
    let r = learner.last_episode.unwrap();
    assert_eq!(r.reward, Some(88));
    assert_eq!(r.attempt.incoming_damage, 7);
}
#[test]
fn threats_and_external_commands_do_not_poison_source_values() {
    let mut l = attempting();
    l.choose(
        Position::new(0, 0),
        &Perception::empty(),
        Decision::new(Intent::Idle { duration: 1 }, Reason::EscapeThreat),
        &mut Random::new(1),
    );
    assert!(l.values.is_empty());
    assert_eq!(l.last_episode.unwrap().end, WaterEnd::Interrupted);
    let mut w = water_world();
    w.tick();
    w.push_command(1, Intent::Idle { duration: 1 });
    w.tick();
    assert!(w.entities[0].learner.episode.is_none());
    assert!(w.entities[0].learner.values.is_empty());
}
#[test]
fn retries_are_bounded_and_failure_is_not_approach_delay() {
    let mut l = attempting();
    for _ in 1..MAX_WATER_TICKS {
        l.finish(1, true, &[]);
        assert!(l.values.is_empty());
    }
    l.finish(1, true, &[]);
    assert_eq!(l.last_episode.as_ref().unwrap().end, WaterEnd::TimedOut);
    assert_eq!(l.values[0].value, -30);
    let mut l = attempting();
    l.finish(
        1,
        true,
        &[SimulationEvent::ActionRejected {
            id: 1,
            action: "Approach".into(),
            target: None,
            reason: RejectionReason::OutOfRange,
        }],
    );
    assert!(l.values.is_empty());
    l.finish(
        1,
        true,
        &[SimulationEvent::ActionRejected {
            id: 1,
            action: "Drink".into(),
            target: None,
            reason: RejectionReason::MissingResource,
        }],
    );
    assert_eq!(l.last_episode.unwrap().end, WaterEnd::Failed);
    assert_eq!(l.values[0].value, -12);
}
#[test]
fn disabling_or_death_closes_without_false_training() {
    let mut l = attempting();
    l.enabled = false;
    l.finish(1, true, &[]);
    assert!(l.values.is_empty());
    assert_eq!(l.last_episode.unwrap().end, WaterEnd::Disabled);
    let mut w = water_world();
    w.tick();
    w.entities[0].base.life_mut().health = 0;
    w.tick();
    assert!(w.entities[0].learner.episode.is_none());
    assert!(w.entities[0].learner.values.is_empty());
    assert_eq!(
        w.entities[0].learner.last_episode.as_ref().unwrap().end,
        WaterEnd::Died
    );
}
#[test]
fn legacy_state_defaults_and_episode_limits_are_checked() {
    let mut old = serde_json::to_value(State::capture(&water_world())).unwrap();
    old["version"] = 9.into();
    let actor = &mut old["entities"][0]["learner"];
    actor.as_object_mut().unwrap().remove("episode");
    actor.as_object_mut().unwrap().remove("last_episode");
    actor["pending"] = serde_json::json!({"x":5,"y":0});
    let resumed = serde_json::from_value::<State>(old)
        .unwrap()
        .restore()
        .unwrap();
    assert!(resumed.entities[0].learner.episode.is_none());
    let mut w = water_world();
    w.entities[0].learner = attempting();
    w.entities[0].learner.episode.as_mut().unwrap().ticks = MAX_WATER_TICKS;
    assert!(State::capture(&w).restore().is_err());
}

#[test]
fn a_visible_chosen_target_continues_even_when_only_one_source_remains() {
    use curious::map::movement::{Direction, Steps};
    let mut learner = attempting();
    let mut view = Perception::empty();
    view.add_water(10, Steps(vec![Direction::Right; 4]));
    let mut rng = Random::new(99);
    let before = rng.state();
    let decision = learner.choose(
        Position::new(1, 0),
        &view,
        Decision::new(
            Intent::Drink {
                at: Steps(vec![Direction::Right; 4]),
            },
            Reason::SeekWater,
        ),
        &mut rng,
    );
    assert_eq!(decision.reason, Reason::LearnedWater);
    assert_eq!(rng.state(), before);
    assert_eq!(
        learner.episode.as_ref().unwrap().target,
        Position::new(5, 0)
    );
    learner.choose(
        Position::new(1, 0),
        &Perception::empty(),
        Decision::new(Intent::Idle { duration: 1 }, Reason::SearchWater),
        &mut rng,
    );
    assert!(learner.episode.is_none());
    assert!(learner.values.is_empty());
}

#[test]
fn movement_energy_is_actual_traversal_cost_even_for_a_loop() {
    use curious::map::movement::{Direction, Steps};
    let mut w = water_world();
    w.entities[0].base.life_mut().speed = 4;
    w.entities[0].base.life_mut().move_energy_cost = 2;
    w.push_command(
        1,
        Intent::Move {
            steps: Steps(vec![
                Direction::Right,
                Direction::Down,
                Direction::Left,
                Direction::Up,
            ]),
        },
    );
    let events = w.tick();
    assert_eq!(w.entities[0].pos, Position::new(0, 0));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, SimulationEvent::EntityMoved { energy_cost: 8, .. }))
    );
}
