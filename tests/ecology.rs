use curious::{
    creatures::{carnivore::CarnivoreEntity, herbivore::HerbivoreEntity, omnivore::OmnivoreEntity},
    entity::{
        Entity,
        controller::EntityController,
        instinct::{Instinct, InstinctEvaluator},
        intent::Intent,
        perception::Perception,
        phase::EntityPhase,
        species::Species,
    },
    map::{Map, movement::Position},
    persistence::State,
    simulation::SimulationEvent,
    simulation::event::{DeathCause, RejectionReason},
    systems::perception::PerceptionSystem,
    world::{EntitySlot, World},
};
fn slot(id: usize, x: isize, base: Box<dyn Entity>) -> EntitySlot {
    EntitySlot::new(
        id,
        Position::new(x, 0),
        EntityPhase::Active,
        EntityController::Player(0),
        base,
    )
}
fn world(entities: Vec<EntitySlot>) -> World {
    let mut w = World::with_seed(-8, 8, -8, 8, entities, 42);
    w.map = Map::new(-8, 8, -8, 8);
    w.environment = Default::default();
    w.logger = curious::logger::Logger::disabled();
    w
}
#[test]
fn offspring_does_not_inherit_parent_cooldown_or_injury_cause() {
    let parents: Vec<Box<dyn Entity>> = vec![
        Box::new(HerbivoreEntity::default()),
        Box::new(CarnivoreEntity::default()),
        Box::new(OmnivoreEntity::default()),
    ];
    for mut parent in parents {
        parent.life_mut().on_reproduce();
        parent.life_mut().moves_used = 2;
        parent.life_mut().death_cause = Some(DeathCause::Combat);
        let child = parent.reproduce();
        assert_eq!(child.life().age, 0);
        assert_eq!(child.life().reproduction_cooldown, 0);
        assert_eq!(child.life().moves_used, 0);
        assert_eq!(child.life().stamina, child.life().max_stamina);
        assert_eq!(child.life().exertion, 0);
        assert_eq!(child.life().death_cause, None);
    }
}
#[test]
fn injury_does_not_prevent_thirst_decision() {
    let mut e = HerbivoreEntity::default();
    e.life_mut().health = 1;
    e.life_mut().water = 0;
    assert_eq!(
        InstinctEvaluator::evaluate(e.life(), &Perception::empty(), Species::Herbivore).instinct,
        Instinct::Thirst
    );
}
#[test]
fn sleeping_cannot_stop_metabolism_or_hide_a_death() {
    let mut s = slot(1, 0, Box::new(HerbivoreEntity::default()));
    s.phase = EntityPhase::Sleeping { remaining: 10 };
    s.base.life_mut().water = 0;
    s.base.life_mut().health = 1;
    let mut w = world(vec![s]);
    let events = w.tick();
    assert!(matches!(w.entities[0].phase, EntityPhase::Corpse { .. }));
    assert_eq!(w.entities[0].base.life().age, 1);
    assert!(events.iter().any(|e| matches!(
        e,
        SimulationEvent::EntityDied {
            cause: Some(DeathCause::Dehydration),
            ..
        }
    )));
    assert!(w.last_intents.is_empty());
    assert!(
        !w.tick()
            .iter()
            .any(|e| matches!(e, SimulationEvent::EntityDied { .. }))
    );
}
#[test]
fn dead_entities_are_not_perceived_as_threats_or_mates() {
    let mut dead = slot(2, 1, Box::new(CarnivoreEntity::default()));
    dead.phase = EntityPhase::Corpse { remaining: 5 };
    dead.base.life_mut().health = 0;
    let w = world(vec![slot(1, 0, Box::new(HerbivoreEntity::default())), dead]);
    let perception = PerceptionSystem::build(&w, &w.entities[0]);
    assert!(perception.entities.is_empty());
}
#[test]
fn mating_validates_species_and_reports_rejection() {
    let mut w = world(vec![
        slot(1, 0, Box::new(HerbivoreEntity::default())),
        slot(2, 1, Box::new(CarnivoreEntity::default())),
    ]);
    w.push_command(1, Intent::Mate { target_id: 2 });
    let events = w.tick();
    assert!(events.iter().any(|e| matches!(
        e,
        SimulationEvent::ActionRejected {
            reason: RejectionReason::DifferentSpecies,
            ..
        }
    )));
    assert_eq!(w.entities.len(), 2);
}
#[test]
fn simultaneous_mating_creates_one_young_child() {
    let mut w = world(vec![
        slot(1, 0, Box::new(HerbivoreEntity::default())),
        slot(2, 1, Box::new(HerbivoreEntity::default())),
    ]);
    for s in &mut w.entities {
        s.base.life_mut().age = 30;
    }
    w.push_command(1, Intent::Mate { target_id: 2 });
    w.push_command(2, Intent::Mate { target_id: 1 });
    let events = w.tick();
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, SimulationEvent::EntityBorn { .. }))
            .count(),
        1
    );
    let child = &w.entities[2];
    assert_eq!(child.base.life().age, 0);
    assert_eq!(child.base.life().reproduction_cooldown, 0);
    assert!(events.iter().any(|e| matches!(
        e,
        SimulationEvent::ActionRejected {
            reason: RejectionReason::Cooldown,
            ..
        }
    )));
}
#[test]
fn independent_worlds_and_checkpoints_have_independent_random_streams() {
    let make = || {
        let mut w = World::with_seed(
            -8,
            8,
            -8,
            8,
            vec![EntitySlot::new(
                1,
                Position::new(0, 0),
                EntityPhase::Active,
                EntityController::AI,
                Box::new(HerbivoreEntity::default()),
            )],
            27,
        );
        w.logger = curious::logger::Logger::disabled();
        w
    };
    let mut a = make();
    let mut b = make();
    let mut other = world(vec![slot(7, 0, Box::new(OmnivoreEntity::default()))]);
    for _ in 0..30 {
        let a_events = a.tick();
        other.tick();
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
    let mut resumed = State::capture(&a).restore().unwrap();
    for _ in 0..30 {
        a.tick();
        other.tick();
        resumed.tick();
        assert_eq!(
            serde_json::to_value(State::capture(&a)).unwrap(),
            serde_json::to_value(State::capture(&resumed)).unwrap()
        );
    }
}

#[test]
fn depleted_sites_regrow_on_schedule_and_respect_capacity() {
    use curious::environment::{Environment, ResourceKind, ResourceSite};
    let pos = Position::new(0, 0);
    let mut map = Map::new(-1, 1, -1, 1);
    let env = Environment {
        sites: vec![ResourceSite {
            pos,
            kind: ResourceKind::Food,
            capacity: 3,
            replenish: 2,
            interval: 2,
        }],
    };
    let mut events = Vec::new();
    env.update(&mut map, 1, &mut events);
    assert!(map.cell(pos).is_none());
    env.update(&mut map, 2, &mut events);
    assert_eq!(
        map.cell(pos),
        Some(&curious::map::cell::Cell::Food { amount: 2 })
    );
    env.update(&mut map, 4, &mut events);
    assert_eq!(
        map.cell(pos),
        Some(&curious::map::cell::Cell::Food { amount: 3 })
    );
    map.reduce_cell_amount(pos, 3);
    env.update(&mut map, 6, &mut events);
    assert_eq!(
        map.cell(pos),
        Some(&curious::map::cell::Cell::Food { amount: 2 })
    );
}

#[test]
fn adjacent_animals_can_share_food_without_sharing_a_cell() {
    use curious::map::{
        cell::Cell,
        movement::{Direction, Steps},
    };
    let mut w = world(vec![
        slot(1, -1, Box::new(HerbivoreEntity::default())),
        slot(2, 1, Box::new(HerbivoreEntity::default())),
    ]);
    w.map
        .set_cell(Position::new(0, 0), Cell::Food { amount: 20 });
    for s in &mut w.entities {
        s.base.life_mut().energy = 10;
    }
    w.push_command(
        1,
        Intent::Eat {
            at: Steps(vec![Direction::Right]),
            corpse_id: None,
        },
    );
    w.push_command(
        2,
        Intent::Eat {
            at: Steps(vec![Direction::Left]),
            corpse_id: None,
        },
    );
    let events = w.tick();
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, SimulationEvent::EntityAte { amount: 10, .. }))
            .count(),
        2
    );
    assert_eq!(w.entities[0].base.life().energy, 19);
    assert_eq!(w.entities[1].base.life().energy, 19);
    assert_eq!(w.entities[0].pos, Position::new(-1, 0));
    assert_eq!(w.entities[1].pos, Position::new(1, 0));
}

#[test]
fn meat_is_finite_and_cannot_be_eaten_by_a_herbivore() {
    use curious::map::movement::{Direction, Steps};
    let mut dead = slot(2, 1, Box::new(HerbivoreEntity::default()));
    dead.phase = EntityPhase::Corpse { remaining: 5 };
    dead.base.life_mut().health = 0;
    dead.corpse_energy = 3;
    let mut w = world(vec![slot(1, 0, Box::new(HerbivoreEntity::default())), dead]);
    w.push_command(
        1,
        Intent::Eat {
            at: Steps(vec![Direction::Right]),
            corpse_id: Some(2),
        },
    );
    assert!(w.tick().iter().any(|e| matches!(
        e,
        SimulationEvent::ActionRejected {
            reason: RejectionReason::InvalidDiet,
            ..
        }
    )));
    w.entities[0].base = Box::new(CarnivoreEntity::default());
    w.entities[0].base.life_mut().energy = 10;
    w.push_command(
        1,
        Intent::Eat {
            at: Steps(vec![Direction::Right]),
            corpse_id: Some(2),
        },
    );
    let events = w.tick();
    assert_eq!(w.entities[0].base.life().energy, 12);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, SimulationEvent::CorpseConsumed { .. }))
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, SimulationEvent::EntityDied { .. }))
    );
    assert_eq!(w.entities[1].corpse_energy, 0);
}

#[test]
fn offspring_energy_and_water_are_paid_by_the_parents() {
    let mut w = world(vec![
        slot(1, 0, Box::new(HerbivoreEntity::default())),
        slot(2, 1, Box::new(HerbivoreEntity::default())),
    ]);
    for s in &mut w.entities {
        s.base.life_mut().age = 30;
    }
    let energy: usize = w.entities.iter().map(|s| s.base.life().energy).sum();
    let water: usize = w.entities.iter().map(|s| s.base.life().water).sum();
    w.push_command(1, Intent::Mate { target_id: 2 });
    w.tick();
    assert_eq!(w.entities.len(), 3);
    assert_eq!(
        w.entities
            .iter()
            .map(|s| s.base.life().energy)
            .sum::<usize>(),
        energy - 2
    );
    assert_eq!(
        w.entities
            .iter()
            .map(|s| s.base.life().water)
            .sum::<usize>(),
        water - 2
    );
    assert_eq!(w.entities[2].generation, 1);
    assert_eq!(w.entities[2].parents, vec![1, 2]);
}

#[test]
fn occupied_cells_are_approachable_but_not_a_path_through_a_wall() {
    use std::collections::HashSet;
    let map = Map::new(0, 4, 0, 0);
    let blocked = HashSet::from([Position::new(1, 0)]);
    let paths = map.paths_within(Position::new(0, 0), 4, &blocked);
    assert!(paths.contains_key(&Position::new(1, 0)));
    assert!(!paths.contains_key(&Position::new(2, 0)));
}

#[test]
fn scarce_food_priority_does_not_permanently_favour_a_founder() {
    use curious::{
        environment::{ResourceKind, ResourceSite},
        map::movement::{Direction, Steps},
    };
    let mut w = world(vec![
        slot(1, -1, Box::new(HerbivoreEntity::default())),
        slot(2, 1, Box::new(HerbivoreEntity::default())),
    ]);
    w.environment.set_site(ResourceSite {
        pos: Position::new(0, 0),
        kind: ResourceKind::Food,
        capacity: 10,
        replenish: 10,
        interval: 1,
    });
    for s in &mut w.entities {
        s.base.life_mut().energy = 10;
    }
    let mut meals = std::collections::BTreeMap::new();
    for _ in 0..4 {
        w.push_command(
            1,
            Intent::Eat {
                at: Steps(vec![Direction::Right]),
                corpse_id: None,
            },
        );
        w.push_command(
            2,
            Intent::Eat {
                at: Steps(vec![Direction::Left]),
                corpse_id: None,
            },
        );
        for event in w.tick() {
            if let SimulationEvent::EntityAte { id, amount, .. } = event {
                *meals.entry(id).or_insert(0) += amount;
            }
        }
    }
    assert_eq!(meals.get(&1), Some(&20));
    assert_eq!(meals.get(&2), Some(&20));
}

#[test]
fn renewable_meadow_supports_descendants_that_reproduce() {
    let setup = || {
        serde_json::from_str::<curious::scenario::Scenario>(include_str!(
            "../scenarios/meadow.json"
        ))
        .unwrap()
        .build(1)
        .unwrap()
    };
    let mut renewed = setup();
    let mut finite = setup();
    finite.environment.sites.clear();
    let mut births = 0;
    let mut generation = 0;
    for _ in 0..1000 {
        births += renewed
            .tick()
            .iter()
            .filter(|e| matches!(e, SimulationEvent::EntityBorn { .. }))
            .count();
        generation = generation.max(
            renewed
                .entities
                .iter()
                .map(|s| s.generation)
                .max()
                .unwrap_or(0),
        );
        finite.tick();
    }
    assert!(births > 30 && generation > 10);
    assert!(
        renewed
            .entities
            .iter()
            .any(|s| matches!(s.phase, EntityPhase::Active | EntityPhase::Sleeping { .. }))
    );
    assert!(
        !finite
            .entities
            .iter()
            .any(|s| matches!(s.phase, EntityPhase::Active | EntityPhase::Sleeping { .. }))
    );
}

#[test]
fn version_one_checkpoints_migrate_explicitly_to_current_version() {
    let mut w = world(vec![slot(1, 0, Box::new(HerbivoreEntity::default()))]);
    w.map.set_cell(
        Position::new(1, 0),
        curious::map::cell::Cell::Food { amount: 10 },
    );
    let mut legacy = serde_json::to_value(State::capture(&w)).unwrap();
    legacy["version"] = serde_json::json!(1);
    legacy.as_object_mut().unwrap().remove("environment");
    for e in legacy["entities"].as_array_mut().unwrap() {
        for key in ["generation", "parents", "corpse_energy"] {
            e.as_object_mut().unwrap().remove(key);
        }
        e["life"].as_object_mut().unwrap().remove("death_cause");
    }
    let restored = serde_json::from_value::<State>(legacy)
        .unwrap()
        .restore()
        .unwrap();
    assert_eq!(restored.environment.sites.len(), 1);
    assert_eq!(restored.entities[0].generation, 0);
    assert_eq!(State::capture(&restored).version, 10);
}
