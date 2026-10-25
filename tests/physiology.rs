use curious::{
    creatures::{carnivore::CarnivoreEntity, herbivore::HerbivoreEntity},
    entity::{
        Entity, behavior, controller::EntityController, decision::Reason, intent::Intent,
        perception::Perception, phase::EntityPhase, species::Species,
    },
    environment::{Environment, ResourceKind, ResourceSite},
    map::{
        Map,
        cell::Cell,
        movement::{Direction, Position, Steps},
    },
    persistence::State,
    random::Random,
    rules::biology,
    scenario::Scenario,
    simulation::{SimulationEvent, event::RejectionReason},
    world::{EntitySlot, World},
};
fn pair() -> World {
    let slots = vec![
        EntitySlot::new(
            1,
            Position::new(0, 0),
            EntityPhase::Active,
            EntityController::Player(0),
            Box::new(CarnivoreEntity::default()),
        ),
        EntitySlot::new(
            2,
            Position::new(1, 0),
            EntityPhase::Active,
            EntityController::Player(0),
            Box::new(HerbivoreEntity::default()),
        ),
    ];
    let mut w = World::with_seed(0, 8, 0, 0, slots, 1);
    w.map = Map::new(0, 8, 0, 0);
    w.environment = Default::default();
    w.logger = curious::logger::Logger::disabled();
    w
}
#[test]
fn attacks_use_capabilities_and_cannot_bypass_fatigue() {
    let mut w = pair();
    w.entities[0].base.life_mut().stamina = 0;
    w.entities[0].base.life_mut().exertion = 1;
    w.push_command(1, Intent::Attack { target_id: 2 });
    let events = w.tick();
    assert!(events.iter().any(|e| matches!(
        e,
        SimulationEvent::ActionRejected {
            reason: RejectionReason::Fatigued,
            ..
        }
    )));
    assert_eq!(w.entities[1].base.life().health, 120);
    w.push_command(1, Intent::Attack { target_id: 2 });
    let energy = w.entities[0].base.life().energy;
    let events = w.tick();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, SimulationEvent::EntityAttacked { damage: 20, .. }))
    );
    assert_eq!(w.entities[0].base.life().energy, energy - 4);
    assert_eq!(w.entities[0].base.life().exertion, 2);
}
#[test]
fn resting_recovers_stamina_without_creating_food_energy() {
    let mut w = pair();
    w.entities[0].base.life_mut().stamina = 0;
    w.entities[0].base.life_mut().exertion = 0;
    let energy = w.entities[0].base.life().energy;
    w.tick();
    assert_eq!(w.entities[0].base.life().stamina, 4);
    assert_eq!(w.entities[0].base.life().energy, energy - 1);
    w.entities[0].base.life_mut().exertion = 2;
    w.tick();
    assert_eq!(w.entities[0].base.life().stamina, 5);
}
#[test]
fn hunting_can_be_abandoned_for_body_limits() {
    let mut carn = CarnivoreEntity::default();
    let mut view = Perception::empty();
    view.add_entity(
        2,
        Species::Herbivore,
        200,
        Steps(vec![Direction::Right]),
        false,
    );
    carn.life_mut().energy = 30;
    carn.life_mut().stamina = 0;
    let mut rng = Random::new(1);
    assert_eq!(
        behavior::decide(Species::Carnivore, carn.life(), view.clone(), &mut rng).reason,
        Reason::RecoverStamina
    );
    carn.life_mut().energy = 2;
    assert_eq!(
        behavior::decide(Species::Carnivore, carn.life(), view, &mut rng).reason,
        Reason::HuntLowEnergy
    );
}
#[test]
fn constrained_prey_can_defend_and_hunter_can_feed_after_kill() {
    let mut w = serde_json::from_str::<Scenario>(include_str!("../scenarios/hunt-corridor.json"))
        .unwrap()
        .build(1)
        .unwrap();
    let mut killed = false;
    let mut fed = false;
    let mut defended = false;
    for _ in 0..50 {
        let events = w.tick();
        killed |= events.iter().any(|e| {
            matches!(
                e,
                SimulationEvent::EntityDied {
                    cause: Some(curious::simulation::event::DeathCause::Combat),
                    ..
                }
            )
        });
        fed |= killed
            && events
                .iter()
                .any(|e| matches!(e, SimulationEvent::EntityAte { id: 1, .. }));
        defended |= w
            .last_decisions
            .iter()
            .any(|d| d.id == 2 && d.decision.reason == Reason::CorneredDefense);
    }
    assert!(killed && fed && defended);
}
#[test]
fn version_two_bodies_receive_explicit_stamina_defaults() {
    let mut state = serde_json::to_value(State::capture(&pair())).unwrap();
    state["version"] = 2.into();
    for e in state["entities"].as_array_mut().unwrap() {
        for key in ["stamina", "max_stamina", "exertion"] {
            e["life"].as_object_mut().unwrap().remove(key);
        }
    }
    let w = serde_json::from_value::<State>(state)
        .unwrap()
        .restore()
        .unwrap();
    assert_eq!(w.entities[0].base.life().stamina, 16);
    assert_eq!(w.entities[0].base.life().max_stamina, 16);
    assert_eq!(w.entities[0].base.life().exertion, 0);
    assert_eq!(State::capture(&w).version, 18);
}

#[test]
fn local_mating_opportunity_obeys_body_safety_limits() {
    use curious::entity::instinct::{Instinct, InstinctEvaluator};
    let mut herb = HerbivoreEntity::default();
    herb.life_mut().age = 30;
    herb.life_mut().energy = herb.life().max_energy / 2;
    herb.life_mut().water = herb.life().max_water / 2;
    let mut view = Perception::empty();
    view.add_entity(
        2,
        Species::Herbivore,
        200,
        Steps(vec![Direction::Right]),
        true,
    );
    assert_eq!(
        InstinctEvaluator::evaluate(herb.life(), &view, Species::Herbivore).instinct,
        Instinct::Mating
    );
    herb.life_mut().water = 0;
    assert_eq!(
        InstinctEvaluator::evaluate(herb.life(), &view, Species::Herbivore).instinct,
        Instinct::Thirst
    );
    herb.life_mut().water = 60;
    view.add_entity(
        3,
        Species::Carnivore,
        200,
        Steps(vec![Direction::Left]),
        false,
    );
    assert_eq!(
        InstinctEvaluator::evaluate(herb.life(), &view, Species::Herbivore).instinct,
        Instinct::Threat
    );
}

#[test]
fn healing_does_not_spend_a_hungry_juveniles_food_reserve() {
    let mut w = pair();
    let life = w.entities[0].base.life_mut();
    life.health = 70;
    life.energy = 44;
    w.tick();
    assert_eq!(w.entities[0].base.life().energy, 43);
    assert_eq!(w.entities[0].base.life().health, 70);
    w.entities[0].base.life_mut().energy = 90;
    w.tick();
    assert_eq!(w.entities[0].base.life().health, 71);
    assert_eq!(w.entities[0].base.life().energy, 87);
}

#[test]
fn hunting_prefers_an_affordable_wounded_prey() {
    let carn = CarnivoreEntity::default();
    let mut view = Perception::empty();
    view.add_entity(
        2,
        Species::Herbivore,
        200,
        Steps(vec![Direction::Right]),
        false,
    );
    view.entities[0].health = Some(120);
    view.add_entity(
        3,
        Species::Herbivore,
        120,
        Steps(vec![Direction::Left, Direction::Left]),
        false,
    );
    view.entities[1].health = Some(20);
    let d = curious::entity::hunting::decide(Species::Carnivore, carn.life(), &view).unwrap();
    assert_eq!(
        d.intent,
        Intent::Move {
            steps: Steps(vec![Direction::Left, Direction::Left])
        }
    );
    let mut low = CarnivoreEntity::default();
    low.life_mut().energy = 10;
    view.entities.remove(1);
    view.entities[0].steps = Steps(vec![Direction::Right; 3]);
    assert_eq!(
        curious::entity::hunting::decide(Species::Carnivore, low.life(), &view)
            .unwrap()
            .reason,
        Reason::HuntTooCostly
    );
}

fn total_map_food(map: &Map) -> usize {
    let mut total = 0;
    for y in map.min_y..=map.max_y {
        for x in map.min_x..=map.max_x {
            if let Some(Cell::Food { amount }) = map.cell(Position::new(x, y)) {
                total += *amount;
            }
        }
    }
    total
}

fn total_map_water(map: &Map) -> usize {
    let mut total = 0;
    for y in map.min_y..=map.max_y {
        for x in map.min_x..=map.max_x {
            if let Some(Cell::Water { amount }) = map.cell(Position::new(x, y)) {
                total += *amount;
            }
        }
    }
    total
}

#[derive(Clone, Debug)]
struct EntityPreTickSnapshot {
    id: usize,
    energy: usize,
    water: usize,
    health: usize,
    max_energy: usize,
    max_water: usize,
    max_health: usize,
    is_alive: bool,
    is_corpse: bool,
}

#[test]
fn energy_and_resource_stock_flow_accounting_closes_exactly() {
    let mut map = Map::new(0, 10, 0, 10);
    map.set_cell(Position::new(1, 0), Cell::Food { amount: 20 });
    map.set_cell(Position::new(5, 0), Cell::Water { amount: 30 });

    let environment = Environment {
        sites: vec![
            ResourceSite {
                pos: Position::new(1, 0),
                kind: ResourceKind::Food,
                capacity: 50,
                replenish: 5,
                interval: 1,
                schedule: None,
            },
            ResourceSite {
                pos: Position::new(5, 0),
                kind: ResourceKind::Water,
                capacity: 50,
                replenish: 5,
                interval: 1,
                schedule: None,
            },
        ],
        ..Default::default()
    };

    // Entity 1: Carnivore (hunter & scavenger)
    let mut c1 = CarnivoreEntity::default();
    c1.life_mut().energy = 70;
    c1.life_mut().max_energy = 100;
    c1.life_mut().stamina = 16;
    c1.life_mut().max_stamina = 16;
    c1.life_mut().health = 100;
    c1.life_mut().max_health = 100;

    // Entity 2: Herbivore prey (to be killed in combat, creating a corpse)
    let mut h2 = HerbivoreEntity::default();
    h2.life_mut().energy = 40;
    h2.life_mut().max_energy = 100;
    h2.life_mut().stamina = 16;
    h2.life_mut().max_stamina = 16;
    h2.life_mut().health = 20; // 1 Carnivore attack (20 dmg) will kill it
    h2.life_mut().max_health = 100;

    // Entity 3: Herbivore Parent 1 (will heal passively, eat plant food, move, mate)
    let mut h3 = HerbivoreEntity::default();
    h3.life_mut().energy = 85; // >= 75 for passive heal
    h3.life_mut().max_energy = 100;
    h3.life_mut().stamina = 16;
    h3.life_mut().max_stamina = 16;
    h3.life_mut().water = 80;
    h3.life_mut().max_water = 100;
    h3.life_mut().health = 90; // < 100, so passive heal triggers
    h3.life_mut().max_health = 100;
    h3.life_mut().age = 30;
    h3.life_mut().maturity_age = 20;
    h3.life_mut().reproduction_cooldown = 0;

    // Entity 4: Herbivore Parent 2 (will move, drink water, mate)
    let mut h4 = HerbivoreEntity::default();
    h4.life_mut().energy = 80;
    h4.life_mut().max_energy = 100;
    h4.life_mut().stamina = 16;
    h4.life_mut().max_stamina = 16;
    h4.life_mut().water = 60; // < 75 so it can drink up to 10
    h4.life_mut().max_water = 100;
    h4.life_mut().health = 100;
    h4.life_mut().max_health = 100;
    h4.life_mut().age = 30;
    h4.life_mut().maturity_age = 20;
    h4.life_mut().reproduction_cooldown = 0;

    // Entity 5: Carnivore with extra trait metabolic maintenance
    let mut c5 = CarnivoreEntity::default();
    c5.life_mut().energy = 80;
    c5.life_mut().max_energy = 100;
    c5.life_mut().stamina = 16;
    c5.life_mut().max_stamina = 16;
    c5.life_mut().health = 100;
    c5.life_mut().max_health = 100;
    c5.life_mut().vision_metabolic_cost = 1;
    c5.life_mut().stamina_metabolic_cost = 1;

    let slots = vec![
        EntitySlot::new(
            1,
            Position::new(8, 8),
            EntityPhase::Active,
            EntityController::Player(0),
            Box::new(c1),
        ),
        EntitySlot::new(
            2,
            Position::new(8, 7),
            EntityPhase::Active,
            EntityController::Player(0),
            Box::new(h2),
        ),
        EntitySlot::new(
            3,
            Position::new(1, 1),
            EntityPhase::Active,
            EntityController::Player(0),
            Box::new(h3),
        ),
        EntitySlot::new(
            4,
            Position::new(2, 1),
            EntityPhase::Active,
            EntityController::Player(0),
            Box::new(h4),
        ),
        EntitySlot::new(
            5,
            Position::new(5, 5),
            EntityPhase::Active,
            EntityController::Player(0),
            Box::new(c5),
        ),
    ];

    let mut world = World::with_seed(0, 10, 0, 10, slots, 999);
    world.map = map;
    world.environment = environment;
    world.logger = curious::logger::Logger::disabled();

    let initial_plant_food = total_map_food(&world.map);
    let initial_water = total_map_water(&world.map);
    assert_eq!(initial_plant_food, 20);
    assert_eq!(initial_water, 30);

    let mut cumulative_replenished_food = 0;
    let mut cumulative_eaten_plant_food = 0;
    let mut cumulative_replenished_water = 0;
    let mut cumulative_drank_water = 0;

    let mut count_moves = 0;
    let mut count_eats_plant = 0;
    let mut count_eats_corpse = 0;
    let mut count_drank = 0;
    let mut count_passive_heal = 0;
    let mut count_metabolized_traits = 0;
    let mut count_attacks = 0;
    let mut count_mates = 0;

    for tick in 1..=6 {
        // Snapshot all entities before tick
        let pre_snapshots: Vec<EntityPreTickSnapshot> = world
            .entities
            .iter()
            .map(|s| EntityPreTickSnapshot {
                id: s.id,
                energy: s.base.life().energy,
                water: s.base.life().water,
                health: s.base.life().health,
                max_energy: s.base.life().max_energy,
                max_water: s.base.life().max_water,
                max_health: s.base.life().max_health,
                is_alive: s.base.life().is_alive(),
                is_corpse: matches!(s.phase, EntityPhase::Corpse { .. }),
            })
            .collect();

        // Push deterministic scripted commands per tick
        match tick {
            1 => {
                // Carnivore 1 attacks Herbivore 2 (kills it, creating corpse)
                world.push_command(1, Intent::Attack { target_id: 2 });
                // Herbivore 3 eats plant food at (1, 0)
                world.push_command(
                    3,
                    Intent::Eat {
                        at: Steps(vec![Direction::Up]),
                        corpse_id: None,
                    },
                );
                // Herbivore 4 moves Right towards water: (2, 1) -> (3, 1)
                world.push_command(
                    4,
                    Intent::Move {
                        steps: Steps(vec![Direction::Right]),
                    },
                );
                // Entity 5 moves Down: (5, 5) -> (5, 6)
                world.push_command(
                    5,
                    Intent::Move {
                        steps: Steps(vec![Direction::Down]),
                    },
                );
            }
            2 => {
                // Carnivore 1 eats corpse of Herbivore 2 at (8, 7)
                world.push_command(
                    1,
                    Intent::Eat {
                        at: Steps(vec![Direction::Up]),
                        corpse_id: Some(2),
                    },
                );
                // Herbivore 3 moves Right: (1, 1) -> (2, 1)
                world.push_command(
                    3,
                    Intent::Move {
                        steps: Steps(vec![Direction::Right]),
                    },
                );
                // Herbivore 4 moves Right: (3, 1) -> (4, 1)
                world.push_command(
                    4,
                    Intent::Move {
                        steps: Steps(vec![Direction::Right]),
                    },
                );
                // Entity 5 moves Right: (5, 6) -> (6, 6)
                world.push_command(
                    5,
                    Intent::Move {
                        steps: Steps(vec![Direction::Right]),
                    },
                );
            }
            3 => {
                // Carnivore 1 moves Left: (8, 8) -> (7, 8)
                world.push_command(
                    1,
                    Intent::Move {
                        steps: Steps(vec![Direction::Left]),
                    },
                );
                // Herbivore 3 moves Right: (2, 1) -> (3, 1)
                world.push_command(
                    3,
                    Intent::Move {
                        steps: Steps(vec![Direction::Right]),
                    },
                );
                // Herbivore 4 moves Right: (4, 1) -> (5, 1) (adjacent to water at (5, 0))
                world.push_command(
                    4,
                    Intent::Move {
                        steps: Steps(vec![Direction::Right]),
                    },
                );
            }
            4 => {
                // Herbivore 4 drinks water from (5, 0)
                world.push_command(
                    4,
                    Intent::Drink {
                        at: Steps(vec![Direction::Up]),
                    },
                );
                // Herbivore 3 moves Right: (3, 1) -> (4, 1) (adjacent to Herbivore 4 at (5, 1))
                world.push_command(
                    3,
                    Intent::Move {
                        steps: Steps(vec![Direction::Right]),
                    },
                );
            }
            5 => {
                // Herbivore 3 and Herbivore 4 reproduce
                world.push_command(3, Intent::Mate { target_id: 4 });
                world.push_command(4, Intent::Mate { target_id: 3 });
                // Carnivore 1 moves: (7, 8) -> (6, 8)
                world.push_command(
                    1,
                    Intent::Move {
                        steps: Steps(vec![Direction::Left]),
                    },
                );
            }
            6 => {
                // Child (id 6) moves Down: (5, 2) -> (5, 3)
                world.push_command(
                    6,
                    Intent::Move {
                        steps: Steps(vec![Direction::Down]),
                    },
                );
                // Carnivore 1 moves: (6, 8) -> (5, 8)
                world.push_command(
                    1,
                    Intent::Move {
                        steps: Steps(vec![Direction::Left]),
                    },
                );
            }
            _ => {}
        }

        let tick_events = world.tick();

        // 1. Track resource stock flows from events
        for e in &tick_events {
            match e {
                SimulationEvent::ResourceReplenished { kind, amount, .. } => match kind {
                    ResourceKind::Food => cumulative_replenished_food += amount,
                    ResourceKind::Water => cumulative_replenished_water += amount,
                },
                SimulationEvent::EntityAte { corpse_id, amount, .. } => {
                    if corpse_id.is_none() {
                        cumulative_eaten_plant_food += amount;
                        count_eats_plant += 1;
                    } else {
                        count_eats_corpse += 1;
                    }
                }
                SimulationEvent::EntityDrank { amount, .. } => {
                    cumulative_drank_water += amount;
                    count_drank += 1;
                }
                SimulationEvent::EntityMoved { energy_cost, .. } => {
                    if *energy_cost > 0 {
                        count_moves += 1;
                    }
                }
                SimulationEvent::EntityAttacked { energy_cost, .. } => {
                    if *energy_cost > 0 {
                        count_attacks += 1;
                    }
                }
                SimulationEvent::EntityMated { .. } => {
                    count_mates += 1;
                }
                SimulationEvent::EntityMetabolized { vision_paid, stamina_paid, .. } => {
                    if *vision_paid > 0 || *stamina_paid > 0 {
                        count_metabolized_traits += 1;
                    }
                }
                _ => {}
            }
        }

        // 2. Per-tick entity energy balance verification
        // For every entity active before this tick:
        for snap in &pre_snapshots {
            if snap.is_corpse {
                continue;
            }

            // Basal metabolism paid (if alive)
            let basal_paid = if snap.is_alive {
                snap.energy.min(biology::BMR_ENERGY_COST)
            } else {
                0
            };

            // Extra traits metabolized
            let trait_paid: usize = tick_events
                .iter()
                .filter_map(|e| match e {
                    SimulationEvent::EntityMetabolized {
                        id,
                        vision_paid,
                        stamina_paid,
                        ..
                    } if *id == snap.id => Some(*vision_paid + *stamina_paid),
                    _ => None,
                })
                .sum();

            // Passive healing paid
            let energy_after_basal_and_traits =
                snap.energy.saturating_sub(basal_paid + trait_paid);
            let water_after_bmr = snap.water.saturating_sub(biology::BMR_WATER_COST);
            let is_water_low = water_after_bmr <= snap.max_water / 4;
            let can_heal = snap.is_alive
                && energy_after_basal_and_traits >= snap.max_energy.saturating_mul(3) / 4
                && !is_water_low
                && snap.health < snap.max_health;
            let passive_heal_paid = if can_heal {
                count_passive_heal += 1;
                biology::PASSIVE_HEAL_ENERGY_COST
            } else {
                0
            };

            // Movement energy paid
            let move_energy: usize = tick_events
                .iter()
                .filter_map(|e| match e {
                    SimulationEvent::EntityMoved {
                        id, energy_cost, ..
                    } if *id == snap.id => Some(*energy_cost),
                    _ => None,
                })
                .sum();

            // Combat attack energy paid
            let attack_energy: usize = tick_events
                .iter()
                .filter_map(|e| match e {
                    SimulationEvent::EntityAttacked {
                        attacker,
                        energy_cost,
                        ..
                    } if *attacker == snap.id => Some(*energy_cost),
                    _ => None,
                })
                .sum();

            // Eating energy restored
            let eaten_energy: usize = tick_events
                .iter()
                .filter_map(|e| match e {
                    SimulationEvent::EntityAte { id, amount, .. } if *id == snap.id => {
                        Some(*amount)
                    }
                    _ => None,
                })
                .sum();

            // Mating energy transferred
            let mate_energy: usize = tick_events
                .iter()
                .filter_map(|e| match e {
                    SimulationEvent::EntityMated {
                        parent1, parent2, ..
                    } if *parent1 == snap.id || *parent2 == snap.id => {
                        Some(snap.max_energy / 4)
                    }
                    _ => None,
                })
                .sum();

            let expected_energy = snap.energy
                .saturating_sub(basal_paid + trait_paid + passive_heal_paid + move_energy + attack_energy + mate_energy)
                .saturating_add(eaten_energy)
                .min(snap.max_energy);

            let end_slot = world
                .entities
                .iter()
                .find(|s| s.id == snap.id)
                .expect("Entity slot should exist");

            assert_eq!(
                end_slot.base.life().energy,
                expected_energy,
                "Tick {tick} entity {}: energy balance does not close exactly. Expected {expected_energy}, got {}",
                snap.id,
                end_slot.base.life().energy
            );
        }

        // Newborn child energy check in the tick of birth
        for e in &tick_events {
            if let SimulationEvent::EntityMated {
                parent1,
                parent2,
                child,
                ..
            } = e
            {
                let p1 = pre_snapshots.iter().find(|s| s.id == *parent1).unwrap();
                let p2 = pre_snapshots.iter().find(|s| s.id == *parent2).unwrap();
                let child_slot = world.entities.iter().find(|s| s.id == *child).unwrap();
                let expected_child_energy = (p1.max_energy / 4 + p2.max_energy / 4)
                    .min(child_slot.base.life().max_energy);
                assert_eq!(
                    child_slot.base.life().energy,
                    expected_child_energy,
                    "Newborn child {child} energy must match parental transfer exactly"
                );
            }
        }
    }

    // --- Final verification of resource stock-flow conservation ---
    let final_plant_food = total_map_food(&world.map);
    assert_eq!(
        initial_plant_food + cumulative_replenished_food - cumulative_eaten_plant_food,
        final_plant_food,
        "Food stock-flow conservation failed: initial({initial_plant_food}) + replenished({cumulative_replenished_food}) - eaten({cumulative_eaten_plant_food}) != final({final_plant_food})"
    );

    let final_water = total_map_water(&world.map);
    assert_eq!(
        initial_water + cumulative_replenished_water - cumulative_drank_water,
        final_water,
        "Water stock-flow conservation failed: initial({initial_water}) + replenished({cumulative_replenished_water}) - drank({cumulative_drank_water}) != final({final_water})"
    );

    // Verify all physical action types occurred during the scenario
    assert!(count_moves > 0, "Scenario must include movement");
    assert!(count_eats_plant > 0, "Scenario must include eating plant food");
    assert!(count_eats_corpse > 0, "Scenario must include eating corpse");
    assert!(count_drank > 0, "Scenario must include drinking water");
    assert!(count_passive_heal > 0, "Scenario must include passive healing");
    assert!(
        count_metabolized_traits > 0,
        "Scenario must include extra-trait metabolism"
    );
    assert!(count_attacks > 0, "Scenario must include combat attack");
    assert!(count_mates > 0, "Scenario must include reproduction");
    assert!(
        cumulative_replenished_food > 0,
        "Food must have replenished"
    );
    assert!(
        cumulative_replenished_water > 0,
        "Water must have replenished"
    );
}
