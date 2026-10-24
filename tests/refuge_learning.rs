use curious::{
    creatures::{carnivore::CarnivoreEntity, herbivore::HerbivoreEntity},
    entity::{
        Entity,
        controller::EntityController,
        decision::{Decision, Reason},
        intent::Intent,
        memory::{
            GeneralizedRefugeKey, GeneralizedRefugeOutcome, MAX_REFUGE_EPISODES_PER_KEY, Memory,
            PendingRefugeClosure, RefugeClosure, RefugeEpisode, RefugeKey, RefugeLearningMode,
            RefugeOutcome,
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
        learning_mode: RefugeLearningMode::LegacyExact,
        generalized_key: None,
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

fn generalized_view(
    threat_id: usize,
) -> (Position, curious::entity::lifestate::LifeState, Perception) {
    let origin = Position::new(2, 1);
    let life = life();
    let mut view = Perception::empty();
    view.shelters = vec![Steps(vec![Direction::Left]), Steps(vec![Direction::Right])];
    view.cells = vec![
        (origin, Cell::Empty),
        (Position::new(1, 1), Cell::Burrow),
        (Position::new(3, 1), Cell::Burrow),
        (Position::new(2, 0), Cell::Empty),
    ];
    view.add_entity(
        threat_id,
        Species::Carnivore,
        140,
        Steps(vec![Direction::Up]),
        false,
    );
    view.entities[0].health = Some(80);
    (origin, life, view)
}

fn generalized_key(
    origin: Position,
    life: &curious::entity::lifestate::LifeState,
    view: &Perception,
    route: &Steps,
) -> GeneralizedRefugeKey {
    let threat = &view.entities[0];
    curious::entity::refuge_generalization::candidate_key(
        origin,
        Species::Herbivore,
        life,
        view,
        threat.species.clone(),
        threat.health.unwrap(),
        threat.power,
        &threat.steps,
        route,
    )
    .unwrap()
}

fn legacy_key_with_options(
    life: &curious::entity::lifestate::LifeState,
    endpoint: Position,
) -> RefugeKey {
    let mut value = key(life, endpoint, 0);
    value.refuge_options = vec![Position::new(1, 1), Position::new(3, 1)];
    value
}

#[test]
fn disabled_and_unobserved_choices_preserve_original_path() {
    let origin = Position::new(3, 3);
    let life = life();
    let view = view();
    let original = baseline_decision();
    let mut disabled = Memory::default();
    let default_memory = serde_json::to_value(&disabled).unwrap();
    assert!(
        default_memory
            .get("refuge_generalization_enabled")
            .is_none()
    );
    assert!(
        default_memory
            .get("refuge_generalization_update_enabled")
            .is_none()
    );
    assert!(default_memory.get("refuge_generalized_outcomes").is_none());
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
fn resolver_finish_fixture_reuses_context_for_a_different_threat_id() {
    let (origin, life, first_view) = generalized_view(2);
    let left = Steps(vec![Direction::Left]);
    let right = Steps(vec![Direction::Right]);
    let left_context = generalized_key(origin, &life, &first_view, &left);
    let right_context = generalized_key(origin, &life, &first_view, &right);
    assert_eq!(
        left_context.estimated_movement_energy,
        life.move_energy_cost as u64
    );
    assert_eq!(left_context.move_energy_cost, life.move_energy_cost);

    let mut outcomes = Vec::new();
    for (route_key, endpoint, damage) in [
        (left_context, Position::new(1, 1), 5),
        (right_context, Position::new(3, 1), 0),
    ] {
        let mut subject = EntitySlot::new(
            1,
            endpoint,
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        );
        subject.memory.refuge_generalization_enabled = true;
        let mut legacy_key = key(&life, endpoint, 0);
        legacy_key.refuge_options = vec![Position::new(1, 1), Position::new(3, 1)];
        let mut active = episode(legacy_key);
        active.learning_mode = RefugeLearningMode::Generalized;
        active.generalized_key = Some(route_key);
        active.target = endpoint;
        subject.memory.refuge_episode = Some(active);

        let mut map = Map::new(0, 4, 0, 2);
        map.set_cell(endpoint, Cell::Burrow);
        let attack = (damage > 0).then_some(SimulationEvent::EntityAttacked {
            energy_cost: 1,
            killed: false,
            attacker: 2,
            target: 1,
            damage,
        });
        let mut entities = vec![subject];
        let events = refuge_outcomes::finish(
            &mut entities,
            &map,
            &attack.into_iter().collect::<Vec<_>>(),
            3,
        );
        assert!(matches!(
            events.as_slice(),
            [SimulationEvent::RefugeEpisodeEnded {
                completed: true,
                generalization_mode: Some(RefugeLearningMode::Generalized),
                ..
            }]
        ));
        outcomes.extend(entities[0].memory.refuge_generalized_outcomes.clone());
    }
    assert_eq!(outcomes.len(), 2);

    let (_, _, later_view) = generalized_view(99);
    assert_eq!(
        generalized_key(origin, &life, &first_view, &left),
        generalized_key(origin, &life, &later_view, &left)
    );
    let mut memory = Memory::default();
    memory.refuge_generalization_enabled = true;
    memory.refuge_generalized_outcomes = outcomes;
    let decision = memory.choose_refuge(origin, &life, &later_view, baseline_decision(), 4);
    assert!(matches!(
        decision.intent,
        Intent::Move { steps: Steps(ref steps) } if steps == &[Direction::Right]
    ));
}

#[test]
fn v17_default_off_golden_matches_v18_before_feature_use() {
    fn fnv1a(bytes: &[u8]) -> u64 {
        let mut hash = 0xcbf29ce484222325u64;
        for byte in bytes {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }

    let entities = vec![
        EntitySlot::new(
            1,
            Position::new(0, 0),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        ),
        EntitySlot::new(
            2,
            Position::new(0, 1),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(CarnivoreEntity::default()),
        ),
    ];
    let mut world = World::with_seed(-3, 3, -3, 3, entities, 123);
    world.logger = curious::logger::Logger::disabled();
    world.map.set_cell(Position::new(1, 0), Cell::Burrow);
    world.map.set_cell(Position::new(-1, 0), Cell::Burrow);
    let mut event_history = Vec::new();
    for _ in 0..80 {
        event_history.extend_from_slice(&serde_json::to_vec(&world.tick()).unwrap());
        event_history.push(0xff);
    }
    let mut state = State::capture(&world);
    state.version = 18;
    let state_bytes = serde_json::to_vec(&state).unwrap();
    assert_eq!(world.rng.state(), 18_145_481_497_135_795_661);
    assert_eq!(fnv1a(&event_history), 0x3c45bf677bedae7b);
    assert_eq!(fnv1a(&state_bytes), 0x44a35bdf53c5adc9);
    assert_eq!(state_bytes.len(), 2907);
}

#[test]
fn world_tick_resolves_generalized_arrival_and_resumes_checkpoint() {
    let mut herbivore = EntitySlot::new(
        1,
        Position::new(3, 2),
        EntityPhase::Active,
        EntityController::AI,
        Box::new(HerbivoreEntity::default()),
    );
    herbivore.base.life_mut().speed = 1;
    herbivore.memory.refuge_generalization_enabled = true;
    let predator = EntitySlot::new(
        2,
        Position::new(3, 1),
        EntityPhase::Active,
        EntityController::AI,
        Box::new(CarnivoreEntity::default()),
    );
    let mut world = World::with_seed(0, 5, 0, 5, vec![herbivore, predator], 123);
    world.logger = curious::logger::Logger::disabled();
    world.map.set_cell(Position::new(1, 2), Cell::Burrow);

    let first_events = world.tick();
    assert_eq!(world.entities[0].pos, Position::new(2, 2));
    assert!(matches!(
        world
            .last_decisions
            .iter()
            .find(|trace| trace.id == 1)
            .unwrap()
            .decision
            .reason,
        Reason::SeekShelter
    ));
    assert!(first_events.iter().any(|event| matches!(
        event,
        SimulationEvent::EntityMoved { id: 1, energy_cost, .. } if *energy_cost > 0
    )));
    assert!(
        world.entities[0]
            .memory
            .refuge_episode
            .as_ref()
            .is_some_and(|episode| {
                episode.learning_mode == RefugeLearningMode::Generalized
                    && episode
                        .generalized_key
                        .as_ref()
                        .is_some_and(|key| key.route.len() == 2)
            })
    );

    let checkpoint = serde_json::to_value(State::capture(&world)).unwrap();
    let mut restored = serde_json::from_value::<State>(checkpoint)
        .unwrap()
        .restore()
        .unwrap();
    let events = world.tick();
    let resumed_events = restored.tick();
    assert_eq!(
        serde_json::to_value(&events).unwrap(),
        serde_json::to_value(&resumed_events).unwrap()
    );
    assert!(events.iter().any(|event| matches!(
        event,
        SimulationEvent::RefugeEpisodeEnded {
            id: 1,
            closure: RefugeClosure::Arrived,
            completed: true,
            generalization_mode: Some(RefugeLearningMode::Generalized),
            generalized_context: Some(_),
            movement_energy,
            ..
        } if *movement_energy > 0
    )));
    assert_eq!(
        world.entities[0].memory.refuge_generalized_outcomes.len(),
        1
    );
    assert_eq!(
        world.entities[0].memory.refuge_generalized_outcomes,
        restored.entities[0].memory.refuge_generalized_outcomes
    );
    assert_eq!(
        serde_json::to_value(State::capture(&world)).unwrap(),
        serde_json::to_value(State::capture(&restored)).unwrap()
    );
}

#[test]
fn unknown_candidate_context_falls_back_for_the_whole_choice() {
    let (origin, life, mut view) = generalized_view(2);
    let right_key = generalized_key(origin, &life, &view, &Steps(vec![Direction::Right]));
    let mut memory = Memory::default();
    memory.refuge_generalization_enabled = true;
    memory
        .refuge_generalized_outcomes
        .push(GeneralizedRefugeOutcome {
            key: right_key,
            last_completed_tick: 2,
            completed: 1,
            safe_arrivals: 1,
            threat_damage: 0,
            movement_energy: 1,
        });
    view.cells.retain(|(pos, _)| *pos != Position::new(3, 1));
    let decision = memory.choose_refuge(origin, &life, &view, baseline_decision(), 3);
    assert!(matches!(
        decision.intent,
        Intent::Move { steps: Steps(ref steps) } if steps == &[Direction::Left]
    ));
    assert!(memory.refuge_episode.is_none());
}

#[test]
fn incompatible_raw_body_context_does_not_reuse_a_row() {
    let (origin, life, view) = generalized_view(2);
    let right_key = generalized_key(origin, &life, &view, &Steps(vec![Direction::Right]));
    let mut changed_life = life.clone();
    changed_life.energy -= 1;
    let mut memory = Memory::default();
    memory.refuge_generalization_enabled = true;
    memory
        .refuge_generalized_outcomes
        .push(GeneralizedRefugeOutcome {
            key: right_key,
            last_completed_tick: 2,
            completed: 1,
            safe_arrivals: 1,
            threat_damage: 0,
            movement_energy: 1,
        });
    let decision = memory.choose_refuge(origin, &changed_life, &view, baseline_decision(), 3);
    assert!(matches!(
        decision.intent,
        Intent::Move { steps: Steps(ref steps) } if steps == &[Direction::Left]
    ));
}

#[test]
fn frozen_generalized_updates_preserve_rows_and_expiry_only_filters_use() {
    let (origin, life, view) = generalized_view(2);
    let candidate = generalized_key(origin, &life, &view, &Steps(vec![Direction::Left]));
    let mut frozen = GeneralizedRefugeOutcome {
        key: candidate.clone(),
        last_completed_tick: 1,
        completed: 1,
        safe_arrivals: 1,
        threat_damage: 0,
        movement_energy: 1,
    };
    let before = frozen.clone();
    assert!(!frozen.record_completed(2, u64::MAX, 1));
    assert_eq!(frozen, before);

    let mut memory = Memory::default();
    memory.refuge_generalization_enabled = true;
    memory.refuge_generalization_update_enabled = false;
    memory.refuge_generalized_outcomes.push(frozen);
    memory.observe(origin, &view, 102, Species::Herbivore);
    assert_eq!(memory.refuge_generalized_outcomes[0], before);
    assert_eq!(
        curious::entity::refuge_generalization::rank_candidates(
            &[candidate],
            &memory.refuge_generalized_outcomes,
            102,
        ),
        None
    );

    let mut future = before.clone();
    future.last_completed_tick = 103;
    assert_eq!(
        curious::entity::refuge_generalization::rank_candidates(
            &[future.key.clone()],
            &[future.clone()],
            102,
        ),
        None
    );
    let future_before = future.clone();
    assert!(!future.record_completed(102, 0, 1));
    assert_eq!(future, future_before);
}

#[test]
fn generalized_support_rejects_row_reuse_and_sample_overflow_atomically() {
    let (origin, life, view) = generalized_view(2);
    let candidate = generalized_key(origin, &life, &view, &Steps(vec![Direction::Left]));
    let mut row = GeneralizedRefugeOutcome {
        key: candidate.clone(),
        last_completed_tick: 2,
        completed: 1,
        safe_arrivals: 1,
        threat_damage: 0,
        movement_energy: 1,
    };
    assert_eq!(
        curious::entity::refuge_generalization::rank_candidates(
            &[candidate.clone(), candidate.clone()],
            &[row.clone()],
            2,
        ),
        None
    );
    row.completed = curious::entity::refuge_generalization::MAX_GENERALIZED_REFUGE_SAMPLES;
    row.safe_arrivals = row.completed;
    let before = row.clone();
    assert!(!row.record_completed(3, 0, 1));
    assert_eq!(row, before);

    let mut bounded = vec![before.clone(); 1];
    let before_table = bounded.clone();
    assert!(
        !curious::entity::refuge_generalization::record_completed_outcome(
            &mut bounded,
            candidate.clone(),
            4,
            u64::MAX,
            0,
        )
    );
    assert_eq!(bounded, before_table);

    let mut retained = Vec::new();
    for delta in 0..=curious::entity::refuge_generalization::MAX_GENERALIZED_REFUGE_OUTCOMES {
        let mut key = candidate.clone();
        key.subject_energy += delta;
        assert!(
            curious::entity::refuge_generalization::record_completed_outcome(
                &mut retained,
                key,
                10,
                0,
                1,
            )
        );
    }
    assert_eq!(
        retained.len(),
        curious::entity::refuge_generalization::MAX_GENERALIZED_REFUGE_OUTCOMES
    );
    assert!(
        !retained
            .iter()
            .any(|row| row.key.subject_energy == candidate.subject_energy)
    );
}

#[test]
fn disabling_generalization_censors_before_resolver_and_emits_once() {
    let (origin, life, view) = generalized_view(2);
    let endpoint = Position::new(3, 1);
    let context = generalized_key(origin, &life, &view, &Steps(vec![Direction::Right]));
    let mut memory = Memory::default();
    memory.refuge_generalization_enabled = true;
    let mut active = episode(legacy_key_with_options(&life, endpoint));
    active.learning_mode = RefugeLearningMode::Generalized;
    active.generalized_key = Some(context);
    active.target = endpoint;
    memory.refuge_episode = Some(active);
    memory.refuge_generalization_enabled = false;
    let decision = memory.choose_refuge(origin, &life, &view, baseline_decision(), 5);
    assert_eq!(
        serde_json::to_value(decision).unwrap(),
        serde_json::to_value(baseline_decision()).unwrap()
    );
    assert!(memory.refuge_episode.is_none());
    assert_eq!(memory.pending_refuge_closures.len(), 1);
    let mut entities = vec![EntitySlot::new(
        1,
        origin,
        EntityPhase::Active,
        EntityController::AI,
        Box::new(HerbivoreEntity::default()),
    )];
    entities[0].memory = memory;
    let map = Map::new(0, 4, 0, 2);
    let events = refuge_outcomes::finish(&mut entities, &map, &[], 5);
    assert!(matches!(
        events.as_slice(),
        [SimulationEvent::RefugeEpisodeEnded {
            closure: RefugeClosure::FeatureDisabled,
            completed: false,
            ..
        }]
    ));
    assert!(entities[0].memory.refuge_generalized_outcomes.is_empty());
    assert_eq!(
        refuge_outcomes::finish(&mut entities, &map, &[], 6).len(),
        0
    );
}

#[test]
fn pending_and_current_refuge_closures_are_each_audited_once() {
    let (mut entities, map) = shelter_world();
    entities[0].memory.refuge_outcomes_enabled = true;
    entities[0]
        .memory
        .pending_refuge_closures
        .push(PendingRefugeClosure {
            episode: episode(key(&life(), Position::new(1, 0), 0)),
            closure: RefugeClosure::TargetChanged,
            ended_tick: 1,
            final_pos: Position::new(1, 0),
        });
    entities[0].memory.refuge_episode = Some(episode(key(&life(), Position::new(1, 0), 0)));
    let events = refuge_outcomes::finish(&mut entities, &map, &[], 2);
    assert_eq!(events.len(), 2);
    assert!(
        events
            .iter()
            .all(|event| matches!(event, SimulationEvent::RefugeEpisodeEnded { id: 1, .. }))
    );
    assert!(entities[0].memory.pending_refuge_closures.is_empty());
    assert!(entities[0].memory.refuge_episode.is_none());
    assert_eq!(
        refuge_outcomes::finish(&mut entities, &map, &[], 3).len(),
        0
    );
}

#[test]
fn v18_checkpoint_resumes_generalized_table_episode_state_and_rng() {
    let (origin, life, view) = generalized_view(2);
    let endpoint = Position::new(3, 1);
    let general_key = generalized_key(origin, &life, &view, &Steps(vec![Direction::Right]));
    let slot = EntitySlot::new(
        1,
        origin,
        EntityPhase::Active,
        EntityController::AI,
        Box::new(HerbivoreEntity::default()),
    );
    let mut world = World::with_seed(0, 4, 0, 2, vec![slot], 77123);
    world.logger = curious::logger::Logger::disabled();
    world.map.set_cell(endpoint, Cell::Burrow);
    world.tick_counter = 7;
    let memory = &mut world.entities[0].memory;
    memory.refuge_generalization_enabled = true;
    memory
        .refuge_generalized_outcomes
        .push(GeneralizedRefugeOutcome {
            key: general_key.clone(),
            last_completed_tick: 7,
            completed: 2,
            safe_arrivals: 1,
            threat_damage: 4,
            movement_energy: 3,
        });
    let mut active = episode(legacy_key_with_options(&life, endpoint));
    active.learning_mode = RefugeLearningMode::Generalized;
    active.generalized_key = Some(general_key);
    active.target = endpoint;
    memory.refuge_episode = Some(active);

    let state = State::capture(&world);
    assert_eq!(state.version, 18);
    let mut restored = serde_json::from_value::<State>(serde_json::to_value(&state).unwrap())
        .unwrap()
        .restore()
        .unwrap();
    assert_eq!(
        serde_json::to_value(State::capture(&restored)).unwrap(),
        serde_json::to_value(state).unwrap()
    );
    let first_events = world.tick();
    let restored_events = restored.tick();
    assert_eq!(
        serde_json::to_value(first_events).unwrap(),
        serde_json::to_value(restored_events).unwrap()
    );
    assert_eq!(
        serde_json::to_value(State::capture(&world)).unwrap(),
        serde_json::to_value(State::capture(&restored)).unwrap()
    );
}

#[test]
fn v17_migration_preserves_legacy_and_water_while_clearing_new_state() {
    let (origin, life, view) = generalized_view(2);
    let endpoint = Position::new(3, 1);
    let general_key = generalized_key(origin, &life, &view, &Steps(vec![Direction::Right]));
    let slot = EntitySlot::new(
        1,
        origin,
        EntityPhase::Active,
        EntityController::AI,
        Box::new(HerbivoreEntity::default()),
    );
    let mut world = World::with_seed(0, 4, 0, 2, vec![slot], 77124);
    world.policy.water_residual_enabled = true;
    world.entities[0]
        .learner
        .residuals
        .push(curious::entity::learning::ResidualValue {
            pos: Position::new(0, 0),
            sum: 17,
            visits: 3,
        });
    let memory = &mut world.entities[0].memory;
    memory.refuge_outcomes_enabled = true;
    let old_key = legacy_key_with_options(&life, endpoint);
    memory.refuge_outcomes.push(RefugeOutcome {
        key: old_key.clone(),
        last_observed_tick: 7,
        completed: 1,
        safe_arrivals: 1,
        threat_damage: 0,
        movement_energy: 2,
    });
    let mut active = episode(old_key);
    active.learning_mode = RefugeLearningMode::Generalized;
    active.generalized_key = Some(general_key.clone());
    memory.refuge_episode = Some(active);
    memory.refuge_generalization_enabled = true;
    memory.refuge_generalization_update_enabled = false;
    memory
        .refuge_generalized_outcomes
        .push(GeneralizedRefugeOutcome {
            key: general_key,
            last_completed_tick: 7,
            completed: 1,
            safe_arrivals: 1,
            threat_damage: 0,
            movement_energy: 2,
        });

    let mut saved = serde_json::to_value(State::capture(&world)).unwrap();
    saved["version"] = 17.into();
    let restored = serde_json::from_value::<State>(saved)
        .unwrap()
        .restore()
        .unwrap();
    let migrated = &restored.entities[0].memory;
    assert!(restored.policy.water_residual_enabled);
    assert_eq!(restored.entities[0].learner.residuals[0].sum, 17);
    assert_eq!(restored.entities[0].learner.residuals[0].visits, 3);
    assert!(migrated.refuge_outcomes_enabled);
    assert_eq!(migrated.refuge_outcomes.len(), 1);
    assert!(!migrated.refuge_generalization_enabled);
    assert!(migrated.refuge_generalization_update_enabled);
    assert!(migrated.refuge_generalized_outcomes.is_empty());
    let active = migrated.refuge_episode.as_ref().unwrap();
    assert_eq!(active.learning_mode, RefugeLearningMode::LegacyExact);
    assert!(active.generalized_key.is_none());
    assert_eq!(State::capture(&restored).version, 18);
}

#[test]
fn future_checkpoint_versions_remain_rejected() {
    let (entities, _) = shelter_world();
    let world = World::with_seed(0, 4, 0, 4, entities, 4);
    let mut state = State::capture(&world);
    state.version = 19;
    assert!(state.restore().is_err());
}

#[test]
fn malformed_generalized_keys_and_duplicate_rows_are_rejected() {
    let (origin, life, view) = generalized_view(2);
    let mut bad_key = generalized_key(origin, &life, &view, &Steps(vec![Direction::Left]));
    bad_key.estimated_movement_energy += 1;
    let slot = EntitySlot::new(
        1,
        origin,
        EntityPhase::Active,
        EntityController::AI,
        Box::new(HerbivoreEntity::default()),
    );
    let mut world = World::with_seed(0, 4, 0, 2, vec![slot], 77125);
    world.entities[0].memory.refuge_generalized_outcomes = vec![GeneralizedRefugeOutcome {
        key: bad_key,
        last_completed_tick: 1,
        completed: 1,
        safe_arrivals: 1,
        threat_damage: 0,
        movement_energy: 1,
    }];
    assert!(State::capture(&world).restore().is_err());

    world.entities[0].memory.refuge_generalized_outcomes.clear();
    let mut keyless_episode = episode(legacy_key_with_options(&life, Position::new(3, 1)));
    keyless_episode.learning_mode = RefugeLearningMode::Generalized;
    world.entities[0].memory.refuge_episode = Some(keyless_episode);
    assert!(State::capture(&world).restore().is_err());
    world.entities[0].memory.refuge_episode = None;

    let mut wrong_route = generalized_key(origin, &life, &view, &Steps(vec![Direction::Left]));
    wrong_route.turns = 1;
    world.entities[0].memory.refuge_generalized_outcomes = vec![GeneralizedRefugeOutcome {
        key: wrong_route,
        last_completed_tick: 0,
        completed: 1,
        safe_arrivals: 1,
        threat_damage: 0,
        movement_energy: 1,
    }];
    assert!(State::capture(&world).restore().is_err());
    world.entities[0].memory.refuge_generalized_outcomes.clear();

    let valid_key = generalized_key(origin, &life, &view, &Steps(vec![Direction::Left]));
    world.entities[0]
        .memory
        .refuge_generalized_outcomes
        .push(GeneralizedRefugeOutcome {
            key: valid_key.clone(),
            last_completed_tick: 0,
            completed: 1,
            safe_arrivals: 1,
            threat_damage: 0,
            movement_energy: 1,
        });
    let mut wrong_subject = State::capture(&world);
    wrong_subject.entities[0].species = Species::Carnivore;
    assert!(wrong_subject.restore().is_err());
    world.entities[0].memory.refuge_generalized_outcomes.clear();

    world.entities[0].memory.refuge_generalized_outcomes = vec![
        GeneralizedRefugeOutcome {
            key: valid_key.clone(),
            last_completed_tick: 1,
            completed: 1,
            safe_arrivals: 1,
            threat_damage: 0,
            movement_energy: 1,
        },
        GeneralizedRefugeOutcome {
            key: valid_key,
            last_completed_tick: 2,
            completed: 1,
            safe_arrivals: 1,
            threat_damage: 0,
            movement_energy: 1,
        },
    ];
    assert!(State::capture(&world).restore().is_err());
}

#[test]
fn future_generalized_outcome_timestamps_are_rejected_on_restore() {
    let (origin, life, view) = generalized_view(2);
    let key = generalized_key(origin, &life, &view, &Steps(vec![Direction::Left]));
    let slot = EntitySlot::new(
        1,
        origin,
        EntityPhase::Active,
        EntityController::AI,
        Box::new(HerbivoreEntity::default()),
    );
    let mut world = World::with_seed(0, 4, 0, 2, vec![slot], 77126);
    world.tick_counter = 2;
    world.entities[0]
        .memory
        .refuge_generalized_outcomes
        .push(GeneralizedRefugeOutcome {
            key,
            last_completed_tick: 3,
            completed: 1,
            safe_arrivals: 1,
            threat_damage: 0,
            movement_energy: 1,
        });
    assert!(State::capture(&world).restore().is_err());
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
    let serialized_legacy_event = serde_json::to_value(&events[0]).unwrap();
    assert!(serialized_legacy_event.get("generalization_mode").is_none());
    assert!(serialized_legacy_event.get("generalized_context").is_none());
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
        parent.memory.refuge_generalization_enabled = true;
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
    let (origin, subject_life, context_view) = generalized_view(2);
    parents[0]
        .memory
        .refuge_generalized_outcomes
        .push(GeneralizedRefugeOutcome {
            key: generalized_key(
                origin,
                &subject_life,
                &context_view,
                &Steps(vec![Direction::Left]),
            ),
            last_completed_tick: 3,
            completed: 1,
            safe_arrivals: 1,
            threat_damage: 0,
            movement_energy: 2,
        });
    let mut world = World::with_seed(-2, 2, -2, 2, parents, 52);
    world.logger = curious::logger::Logger::disabled();
    world.push_command(1, Intent::Mate { target_id: 2 });
    world.push_command(2, Intent::Mate { target_id: 1 });
    world.tick();
    let child = &world.entities[2];
    assert!(child.memory.refuge_outcomes_enabled);
    assert!(child.memory.refuge_generalization_enabled);
    assert!(child.memory.refuge_outcomes.is_empty());
    assert!(child.memory.refuge_generalized_outcomes.is_empty());
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
