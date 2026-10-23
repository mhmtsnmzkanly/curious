use curious::{
    creatures::herbivore::HerbivoreEntity,
    entity::{
        Entity,
        controller::EntityController,
        decision::{Decision, Reason},
        intent::Intent,
        memory::{
            MAX_REFUGE_EPISODES_PER_KEY, Memory, PendingRefugeClosure, RefugeClosure,
            RefugeEpisode, RefugeKey, RefugeOutcome,
        },
        perception::Perception,
        phase::EntityPhase,
        species::Species,
    },
    map::{
        Map,
        cell::Cell,
        movement::{Direction, Position, Steps},
    },
    simulation::{SimulationEvent, checkpoint::State},
    systems::refuge_outcomes,
    world::{EntitySlot, World},
};

fn life() -> curious::entity::lifestate::LifeState {
    HerbivoreEntity::default().life().clone()
}

fn view() -> Perception {
    let mut view = Perception::empty();
    view.shelters = vec![Steps(vec![Direction::Left]), Steps(vec![Direction::Right])];
    view.add_entity(2, Species::Carnivore, 50, Steps(vec![Direction::Up]), false);
    view
}

fn key(life: &curious::entity::lifestate::LifeState, endpoint: Position, bearing: u8) -> RefugeKey {
    RefugeKey {
        threat_id: 2,
        threat_species: Species::Carnivore,
        bearing,
        threat_distance: 0,
        energy_quartile: ((life.energy.min(life.max_energy) as u128 * 4)
            / life.max_energy.max(1) as u128)
            .min(3) as u8,
        stamina_quartile: ((life.stamina.min(life.max_stamina) as u128 * 4)
            / life.max_stamina.max(1) as u128)
            .min(3) as u8,
        refuge_options: vec![Position::new(2, 3), Position::new(4, 3)],
        endpoint,
    }
}

fn baseline_decision() -> Decision {
    Decision::new(
        Intent::Move {
            steps: Steps(vec![Direction::Left]),
        },
        Reason::SeekShelter,
    )
}

fn episode(key: RefugeKey) -> RefugeEpisode {
    RefugeEpisode {
        key: key.clone(),
        started_tick: 1,
        target: key.endpoint,
        threat_id: key.threat_id,
        threat_species: key.threat_species,
        movement_energy: 0,
        threat_damage: 0,
        other_damage: 0,
        unattributed_health_loss: 0,
        last_health: life().health,
        elapsed_ticks: 0,
        unseen_ticks: 0,
    }
}

fn shelter_world() -> (Vec<EntitySlot>, Map) {
    let slot = EntitySlot::new(
        1,
        Position::new(1, 0),
        EntityPhase::Active,
        EntityController::AI,
        Box::new(HerbivoreEntity::default()),
    );
    let mut map = Map::new(0, 4, 0, 4);
    map.set_cell(Position::new(1, 0), Cell::Burrow);
    (vec![slot], map)
}

#[test]
fn disabled_and_unobserved_choices_preserve_original_path() {
    let origin = Position::new(3, 3);
    let life = life();
    let view = view();
    let original = baseline_decision();
    let mut disabled = Memory::default();
    assert_eq!(
        serde_json::to_value(disabled.choose_refuge(origin, &life, &view, original.clone(), 1))
            .unwrap(),
        serde_json::to_value(&original).unwrap()
    );
    let mut enabled = Memory::default();
    enabled.refuge_outcomes_enabled = true;
    let decision = enabled.choose_refuge(origin, &life, &view, original.clone(), 1);
    assert_eq!(
        serde_json::to_value(decision).unwrap(),
        serde_json::to_value(original).unwrap()
    );
}

#[test]
fn observed_candidate_ranks_first_and_unknown_is_never_zero_risk() {
    let origin = Position::new(3, 3);
    let life = life();
    let view = view();
    let right = key(&life, Position::new(4, 3), 0);
    let mut memory = Memory::default();
    memory.refuge_outcomes_enabled = true;
    memory.refuge_outcomes.push(RefugeOutcome {
        key: right,
        last_observed_tick: 1,
        completed: 3,
        safe_arrivals: 3,
        threat_damage: 0,
        movement_energy: 9,
    });
    let decision = memory.choose_refuge(origin, &life, &view, baseline_decision(), 2);
    assert!(
        matches!(decision.intent, Intent::Move { steps: Steps(ref steps) } if steps == &[Direction::Right])
    );

    let mut memory = Memory::default();
    memory.refuge_outcomes_enabled = true;
    memory.refuge_outcomes.push(RefugeOutcome {
        key: key(&life, Position::new(2, 3), 0),
        last_observed_tick: 1,
        completed: 2,
        safe_arrivals: 0,
        threat_damage: 20,
        movement_energy: 9,
    });
    let decision = memory.choose_refuge(origin, &life, &view, baseline_decision(), 2);
    assert!(
        matches!(decision.intent, Intent::Move { steps: Steps(ref steps) } if steps == &[Direction::Left])
    );
}

#[test]
fn same_endpoint_keeps_starting_context_but_target_change_censors_without_cost_transfer() {
    let origin = Position::new(3, 3);
    let initial_life = life();
    let initial_view = view();
    let mut memory = Memory::default();
    memory.refuge_outcomes_enabled = true;
    memory.choose_refuge(origin, &initial_life, &initial_view, baseline_decision(), 1);
    let starting_key = memory.refuge_episode.as_ref().unwrap().key.clone();

    let mut shifted_view = view();
    shifted_view.entities[0].steps = Steps(vec![Direction::Right, Direction::Right]);
    memory.choose_refuge(origin, &initial_life, &shifted_view, baseline_decision(), 2);
    assert_eq!(memory.refuge_episode.as_ref().unwrap().key, starting_key);

    let mut right_key = key(&initial_life, Position::new(4, 3), 2);
    right_key.threat_distance = 1;
    memory.refuge_outcomes.push(RefugeOutcome {
        key: right_key,
        last_observed_tick: 2,
        completed: 1,
        safe_arrivals: 1,
        threat_damage: 0,
        movement_energy: 0,
    });
    let changed =
        memory.choose_refuge(origin, &initial_life, &shifted_view, baseline_decision(), 3);
    assert!(
        matches!(changed.intent, Intent::Move { steps: Steps(ref steps) } if steps == &[Direction::Right])
    );
    let pending = memory.pending_refuge_closures.first().unwrap();
    assert_eq!(pending.closure, RefugeClosure::TargetChanged);
    assert_eq!(pending.episode.key, starting_key);
    assert_eq!(pending.episode.movement_energy, 0);
    let new_episode = memory.refuge_episode.as_ref().unwrap();
    assert_eq!(new_episode.target, Position::new(4, 3));
    assert_eq!(new_episode.movement_energy, 0);
    assert_eq!(new_episode.started_tick, 3);
}

#[test]
fn resolver_arrival_and_damage_update_only_attributed_completed_records() {
    let (mut entities, map) = shelter_world();
    let subject = &mut entities[0];
    subject.memory.refuge_outcomes_enabled = true;
    let mut arrival_key = key(&life(), Position::new(1, 0), 0);
    arrival_key.refuge_options = vec![Position::new(1, 0)];
    subject.memory.refuge_episode = Some(episode(arrival_key));
    let events = refuge_outcomes::finish(&mut entities, &map, &[], 2);
    assert!(matches!(
        events.as_slice(),
        [SimulationEvent::RefugeEpisodeEnded {
            completed: true,
            safe_arrival: true,
            ..
        }]
    ));
    let learned = &entities[0].memory.refuge_outcomes[0];
    assert_eq!((learned.completed, learned.safe_arrivals), (1, 1));

    let (mut entities, map) = shelter_world();
    entities[0].memory.refuge_outcomes_enabled = true;
    let mut arrival_key = key(&life(), Position::new(1, 0), 0);
    arrival_key.refuge_options = vec![Position::new(1, 0)];
    entities[0].memory.refuge_episode = Some(episode(arrival_key));
    let attack = SimulationEvent::EntityAttacked {
        energy_cost: 1,
        killed: false,
        attacker: 2,
        target: 1,
        damage: 5,
    };
    let events = refuge_outcomes::finish(&mut entities, &map, &[attack], 2);
    assert!(matches!(
        events.as_slice(),
        [SimulationEvent::RefugeEpisodeEnded {
            completed: true,
            safe_arrival: false,
            threat_damage: 5,
            ..
        }]
    ));
    assert_eq!(entities[0].memory.refuge_outcomes[0].threat_damage, 5);
    assert_eq!(entities[0].memory.refuge_outcomes[0].safe_arrivals, 0);
}

#[test]
fn unobserved_health_loss_is_reported_and_censors_learning() {
    let (mut entities, map) = shelter_world();
    entities[0].memory.refuge_outcomes_enabled = true;
    let mut arrival_key = key(&life(), Position::new(1, 0), 0);
    arrival_key.refuge_options = vec![Position::new(1, 0)];
    entities[0].memory.refuge_episode = Some(episode(arrival_key));
    entities[0].base.life_mut().health -= 4;
    let events = refuge_outcomes::finish(&mut entities, &map, &[], 2);
    assert!(matches!(
        events.as_slice(),
        [SimulationEvent::RefugeEpisodeEnded {
            attribution_known: false,
            completed: true,
            unattributed_health_loss: 4,
            ..
        }]
    ));
    assert!(entities[0].memory.refuge_outcomes.is_empty());
}

#[test]
fn foreign_damage_is_censored_and_joint_counter_cap_freezes_measurements() {
    let (mut entities, map) = shelter_world();
    entities[0].memory.refuge_outcomes_enabled = true;
    let mut arrival_key = key(&life(), Position::new(1, 0), 0);
    arrival_key.refuge_options = vec![Position::new(1, 0)];
    entities[0].memory.refuge_episode = Some(episode(arrival_key));
    let attack = SimulationEvent::EntityAttacked {
        energy_cost: 1,
        killed: false,
        attacker: 3,
        target: 1,
        damage: 7,
    };
    let events = refuge_outcomes::finish(&mut entities, &map, &[attack], 2);
    assert!(matches!(
        events.as_slice(),
        [SimulationEvent::RefugeEpisodeEnded {
            attribution_known: false,
            completed: true,
            ..
        }]
    ));
    assert!(entities[0].memory.refuge_outcomes.is_empty());

    let (mut entities, map) = shelter_world();
    let mut cap_key = key(&life(), Position::new(1, 0), 0);
    cap_key.refuge_options = vec![Position::new(1, 0)];
    entities[0].memory.refuge_outcomes.push(RefugeOutcome {
        key: cap_key.clone(),
        last_observed_tick: 1,
        completed: MAX_REFUGE_EPISODES_PER_KEY,
        safe_arrivals: MAX_REFUGE_EPISODES_PER_KEY,
        threat_damage: 13,
        movement_energy: 17,
    });
    entities[0].memory.refuge_episode = Some(episode(cap_key));
    refuge_outcomes::finish(&mut entities, &map, &[], 2);
    let row = &entities[0].memory.refuge_outcomes[0];
    assert_eq!(
        (
            row.completed,
            row.safe_arrivals,
            row.threat_damage,
            row.movement_energy
        ),
        (
            MAX_REFUGE_EPISODES_PER_KEY,
            MAX_REFUGE_EPISODES_PER_KEY,
            13,
            17
        )
    );
}

#[test]
fn priority_interrupt_censors_old_route_and_does_not_charge_other_action() {
    let origin = Position::new(3, 3);
    let life = life();
    let view = view();
    let mut memory = Memory::default();
    memory.refuge_outcomes_enabled = true;
    memory.choose_refuge(origin, &life, &view, baseline_decision(), 1);
    let mut interrupted = Decision::new(Intent::Idle { duration: 1 }, Reason::SearchWater);
    interrupted = memory.choose_refuge(origin, &life, &view, interrupted, 2);
    assert_eq!(interrupted.reason, Reason::SearchWater);
    assert!(memory.refuge_episode.is_none());
    assert_eq!(memory.pending_refuge_closures.len(), 1);
    assert_eq!(
        memory.pending_refuge_closures[0].closure,
        RefugeClosure::PriorityInterrupted
    );

    let (mut entities, map) = shelter_world();
    entities[0].memory = memory;
    let movement = SimulationEvent::EntityMoved {
        id: 1,
        from: Position::new(2, 0),
        to: Position::new(1, 0),
        energy_cost: 77,
    };
    let events = refuge_outcomes::finish(&mut entities, &map, &[movement], 2);
    assert!(matches!(
        events.as_slice(),
        [SimulationEvent::RefugeEpisodeEnded {
            closure: RefugeClosure::PriorityInterrupted,
            completed: false,
            movement_energy: 0,
            ..
        }]
    ));
    assert!(entities[0].memory.refuge_outcomes.is_empty());
}

#[test]
fn saved_episode_cross_fields_and_unknown_endpoint_are_rejected() {
    let mut memory = Memory::default();
    let mut invalid = episode(key(&life(), Position::new(2, 3), 0));
    invalid.threat_id = 99;
    memory.refuge_episode = Some(invalid);
    assert!(memory.validate().is_err());

    let mut memory = Memory::default();
    let mut invalid = key(&life(), Position::new(9, 9), 0);
    invalid.endpoint = Position::new(8, 8);
    memory.refuge_outcomes.push(RefugeOutcome {
        key: invalid,
        last_observed_tick: 1,
        completed: 1,
        safe_arrivals: 1,
        threat_damage: 0,
        movement_energy: 0,
    });
    assert!(memory.validate().is_err());
}

#[test]
fn offspring_inherits_only_the_joint_mode_and_never_the_parent_history() {
    let mut parents = vec![
        EntitySlot::new(
            1,
            Position::new(0, 0),
            EntityPhase::Active,
            EntityController::Player(0),
            Box::new(HerbivoreEntity::default()),
        ),
        EntitySlot::new(
            2,
            Position::new(1, 0),
            EntityPhase::Active,
            EntityController::Player(1),
            Box::new(HerbivoreEntity::default()),
        ),
    ];
    for parent in &mut parents {
        parent.base.life_mut().age = 30;
        parent.memory.refuge_outcomes_enabled = true;
    }
    parents[0].memory.refuge_outcomes.push(RefugeOutcome {
        key: key(&life(), Position::new(2, 0), 0),
        last_observed_tick: 3,
        completed: 1,
        safe_arrivals: 1,
        threat_damage: 0,
        movement_energy: 2,
    });
    parents[0].memory.refuge_episode = Some(episode(key(&life(), Position::new(2, 0), 0)));
    let mut world = World::with_seed(-2, 2, -2, 2, parents, 52);
    world.logger = curious::logger::Logger::disabled();
    world.push_command(1, Intent::Mate { target_id: 2 });
    world.push_command(2, Intent::Mate { target_id: 1 });
    world.tick();
    let child = &world.entities[2];
    assert!(child.memory.refuge_outcomes_enabled);
    assert!(child.memory.refuge_outcomes.is_empty());
    assert!(child.memory.refuge_episode.is_none());
    assert!(child.memory.pending_refuge_closures.is_empty());
}

#[test]
fn disabled_worlds_keep_checkpoint_and_rng_equal() {
    let make = || {
        let slot = EntitySlot::new(
            1,
            Position::new(0, 0),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        );
        World::with_seed(-4, 4, -4, 4, vec![slot], 9981)
    };
    let mut a = make();
    let mut b = make();
    for _ in 0..40 {
        let a_events = a.tick();
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
}

#[test]
fn pending_refuge_closure_survives_checkpoint_and_emits_once_after_restore() {
    let mut subject = EntitySlot::new(
        1,
        Position::new(3, 3),
        EntityPhase::Active,
        EntityController::AI,
        Box::new(HerbivoreEntity::default()),
    );
    subject.memory.refuge_outcomes_enabled = true;
    let stored_episode = episode(key(&life(), Position::new(2, 3), 0));
    subject
        .memory
        .pending_refuge_closures
        .push(PendingRefugeClosure {
            episode: stored_episode.clone(),
            closure: RefugeClosure::TargetChanged,
            ended_tick: 1,
            final_pos: Position::new(3, 3),
        });
    let mut world = World::with_seed(0, 6, 0, 6, vec![subject], 778);
    world.logger = curious::logger::Logger::disabled();
    let captured = serde_json::to_value(State::capture(&world)).unwrap();
    let mut restored = State::capture(&world).restore().unwrap();
    assert_eq!(
        serde_json::to_value(State::capture(&restored)).unwrap(),
        captured
    );
    assert_eq!(
        restored.entities[0].memory.pending_refuge_closures[0].episode,
        stored_episode
    );
    let first = restored.tick();
    assert_eq!(
        first
            .iter()
            .filter(|event| matches!(event, SimulationEvent::RefugeEpisodeEnded { .. }))
            .count(),
        1
    );
    let second = restored.tick();
    assert!(
        !second
            .iter()
            .any(|event| matches!(event, SimulationEvent::RefugeEpisodeEnded { .. }))
    );
}
