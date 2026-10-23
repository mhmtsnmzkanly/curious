use curious::{
    creatures::herbivore::HerbivoreEntity,
    entity::{controller::EntityController, phase::EntityPhase, Entity},
    environment::{Environment, ResourceKind, ResourceSchedule, ResourceSite},
    map::{cell::Cell, movement::Position, Map},
    simulation::{checkpoint::State, SimulationEvent},
    world::{EntitySlot, World},
};

fn site(pos: Position, schedule: Option<ResourceSchedule>) -> ResourceSite {
    ResourceSite {
        pos,
        kind: ResourceKind::Food,
        capacity: 100,
        replenish: 1,
        interval: 1,
        schedule,
    }
}

#[test]
fn resource_schedule_staggers_sites_without_using_or_changing_rng() {
    let schedule = ResourceSchedule {
        period: 12,
        active_ticks: 4,
        geographic_phase_step: 1,
        phase_offset: 0,
    };
    schedule.validate().unwrap();
    let a = Position::new(0, 0);
    let b = Position::new(1, 0);
    let mut world = World::with_seed(-2, 2, -2, 2, vec![], 17);
    world.map = Map::new(-2, 2, -2, 2);
    world.environment = Environment {
        sites: vec![site(a, Some(schedule)), site(b, Some(schedule))],
    };
    world.map.set_cell(a, Cell::Food { amount: 0 });
    world.map.set_cell(b, Cell::Food { amount: 0 });
    let initial_rng = world.rng.state();
    let mut events_by_tick = Vec::new();
    let (mut amount_a, mut amount_b) = (0, 0);
    for tick in 1..=48 {
        let events = world.tick();
        events_by_tick.push(
            events
                .iter()
                .filter(|event| matches!(event, SimulationEvent::ResourceReplenished { .. }))
                .count(),
        );
        assert_eq!(world.rng.state(), initial_rng);
        let expected_a = ((tick as isize).rem_euclid(12)) < 4;
        let expected_b = ((tick as isize + 1).rem_euclid(12)) < 4;
        amount_a += usize::from(expected_a);
        amount_b += usize::from(expected_b);
        assert_eq!(
            events_by_tick.last().copied().unwrap(),
            usize::from(expected_a) + usize::from(expected_b)
        );
        assert_eq!(world.map.cell(a), Some(&Cell::Food { amount: amount_a }));
        assert_eq!(world.map.cell(b), Some(&Cell::Food { amount: amount_b }));
    }
    assert_eq!(events_by_tick.iter().sum::<usize>(), 32);
}

#[test]
fn resource_schedule_survives_checkpoint_and_invalid_parameters_are_rejected() {
    let mut world = World::with_seed(-2, 2, -2, 2, vec![], 33);
    world.map = Map::new(-2, 2, -2, 2);
    let p = Position::new(1, -1);
    let schedule = ResourceSchedule {
        period: 12,
        active_ticks: 4,
        geographic_phase_step: 1,
        phase_offset: 2,
    };
    world.environment = Environment {
        sites: vec![site(p, Some(schedule))],
    };
    world.map.set_cell(p, Cell::Food { amount: 3 });
    for _ in 0..17 {
        world.tick();
    }
    let mut resumed = State::capture(&world).restore().unwrap();
    for _ in 0..80 {
        world.tick();
        resumed.tick();
        assert_eq!(
            serde_json::to_value(State::capture(&world)).unwrap(),
            serde_json::to_value(State::capture(&resumed)).unwrap()
        );
    }
    assert_eq!(world.rng.state(), resumed.rng.state());

    let bad = ResourceSchedule {
        period: 0,
        active_ticks: 0,
        geographic_phase_step: 1,
        phase_offset: 0,
    };
    assert!(bad.validate().is_err());
    let bad_offset = ResourceSchedule {
        phase_offset: isize::MAX,
        ..schedule
    };
    assert!(bad_offset.validate().is_err());

    let mut malformed = serde_json::to_value(State::capture(&world)).unwrap();
    malformed["environment"]["sites"][0]["schedule"]["period"] = 0.into();
    assert!(serde_json::from_value::<State>(malformed)
        .unwrap()
        .restore()
        .is_err());
}

#[test]
fn closed_v13_migration_discards_new_opt_in_fields_without_changing_future_state() {
    let slot = EntitySlot::new(
        1,
        Position::new(0, 0),
        EntityPhase::Active,
        EntityController::Player(0),
        Box::new(HerbivoreEntity::default()),
    );
    let mut baseline = World::with_seed(0, 2, 0, 2, vec![slot], 71);
    baseline.map = Map::new(0, 2, 0, 2);
    baseline.environment = Environment {
        sites: vec![site(Position::new(2, 2), None)],
    };
    let mut serialized = serde_json::to_value(State::capture(&baseline)).unwrap();
    serialized["version"] = 13.into();
    serialized["genetics"]["extra_traits_enabled"] = true.into();
    serialized["entities"][0]["genome"]["vision"] = 2.into();
    serialized["entities"][0]["genome"]["stamina"] = 1.into();
    serialized["entities"][0]["life"]["vision_metabolic_cost"] = 2.into();
    serialized["entities"][0]["life"]["stamina_metabolic_cost"] = 1.into();
    serialized["environment"]["sites"][0]["schedule"] = serde_json::json!({
        "period":4,"active_ticks":2,"geographic_phase_step":1,"phase_offset":0
    });
    let mut migrated = serde_json::from_value::<State>(serialized)
        .unwrap()
        .restore()
        .unwrap();
    let mut expected = baseline;
    for _ in 0..50 {
        expected.tick();
        migrated.tick();
        let mut a = serde_json::to_value(State::capture(&expected)).unwrap();
        let mut b = serde_json::to_value(State::capture(&migrated)).unwrap();
        a["version"] = 13.into();
        b["version"] = 13.into();
        assert_eq!(a, b);
    }
    assert_eq!(expected.rng.state(), migrated.rng.state());
}

#[test]
fn v15_reader_preserves_v14_risk_evidence() {
    let slot = EntitySlot::new(
        1,
        Position::new(0, 0),
        EntityPhase::Active,
        EntityController::Player(0),
        Box::new(HerbivoreEntity::default()),
    );
    let world = World::with_seed(0, 1, 0, 1, vec![slot], 19);
    let mut state = serde_json::to_value(State::capture(&world)).unwrap();
    state["version"] = 14.into();
    state["entities"][0]["hunt_learner"]["risk_enabled"] = true.into();
    state["entities"][0]["hunt_learner"]["risk_values"] = serde_json::json!([{
        "context":{"energy":1,"stamina":2,"distance":0,"health":3},
        "exposures":2,"injuries":1,"damage":7
    }]);
    let restored = serde_json::from_value::<State>(state)
        .unwrap()
        .restore()
        .unwrap();
    assert!(restored.entities[0].hunt_learner.risk_enabled);
    assert_eq!(restored.entities[0].hunt_learner.risk_values.len(), 1);
    assert_eq!(restored.entities[0].hunt_learner.risk_values[0].damage, 7);
}

#[test]
fn metabolism_audits_requested_and_actually_paid_trait_costs() {
    for (id, energy, vision_paid, stamina_paid) in [(1, 2, 1, 0), (3, 1, 0, 0)] {
        let mut body = HerbivoreEntity::default();
        body.life_mut().vision_metabolic_cost = 1;
        body.life_mut().stamina_metabolic_cost = 1;
        body.life_mut().energy = energy;
        let slot = EntitySlot::new(
            id,
            Position::new(0, 0),
            EntityPhase::Active,
            EntityController::Player(0),
            Box::new(body),
        );
        let mut world = World::with_seed(0, 0, 0, 0, vec![slot], 13);
        world.map = Map::new(0, 0, 0, 0);
        world.environment = Environment::default();
        let events = world.tick();
        assert!(events.iter().any(|event| matches!(
            event,
            SimulationEvent::EntityMetabolized {
                id: event_id,
                vision_cost: 1,
                stamina_cost: 1,
                vision_paid: actual_vision,
                stamina_paid: actual_stamina,
            } if *event_id == id && *actual_vision == vision_paid && *actual_stamina == stamina_paid
        )));
    }

    let mut legacy = HerbivoreEntity::default();
    legacy.life_mut().energy = 20;
    let slot = EntitySlot::new(
        2,
        Position::new(0, 0),
        EntityPhase::Active,
        EntityController::Player(0),
        Box::new(legacy),
    );
    let mut world = World::with_seed(0, 0, 0, 0, vec![slot], 13);
    world.map = Map::new(0, 0, 0, 0);
    world.environment = Environment::default();
    assert!(!world
        .tick()
        .iter()
        .any(|event| matches!(event, SimulationEvent::EntityMetabolized { .. })));
}

#[test]
fn scenario_alleles_express_only_when_extra_traits_are_enabled() {
    let mut scenario: curious::scenario::Scenario =
        serde_json::from_str(include_str!("../scenarios/ecology-traits-lab.json")).unwrap();
    for founder in &mut scenario.founders {
        founder.genome.vision = 1;
        founder.genome.stamina = 1;
    }
    let enabled = scenario.build(5).unwrap();
    assert!(enabled.genetics.extra_traits_enabled);
    assert_eq!(enabled.entities[0].base.life().vision_range, 7);
    assert_eq!(enabled.entities[0].base.life().max_stamina, 20);
    assert_eq!(enabled.entities[0].base.life().vision_metabolic_cost, 1);
    assert_eq!(enabled.entities[0].base.life().stamina_metabolic_cost, 1);

    let mut closed: curious::scenario::Scenario =
        serde_json::from_str(include_str!("../scenarios/ecology-traits-lab.json")).unwrap();
    closed.genetics.extra_traits_enabled = false;
    for founder in &mut closed.founders {
        founder.genome.vision = 1;
        founder.genome.stamina = 1;
    }
    let closed = closed.build(5).unwrap();
    assert_eq!(closed.entities[0].base.life().vision_range, 6);
    assert_eq!(closed.entities[0].base.life().max_stamina, 16);
    assert_eq!(closed.entities[0].base.life().vision_metabolic_cost, 0);
    assert_eq!(closed.entities[0].base.life().stamina_metabolic_cost, 0);
}
