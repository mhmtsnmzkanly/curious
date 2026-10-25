#[path = "../examples/support/ecology_audit.rs"]
mod ecology_audit;

use curious::{
    creatures::herbivore::HerbivoreEntity,
    entity::{
        Entity, controller::EntityController, genetics::Genome, intent::Intent, phase::EntityPhase,
    },
    environment::{ResourceKind, ResourceSite},
    map::{
        cell::Cell,
        movement::{Direction, Position, Steps},
    },
    scenario::Scenario,
    simulation::SimulationEvent,
    world::{EntitySlot, World},
};
use std::collections::BTreeSet;

fn tiny_world() -> curious::world::World {
    let scenario: Scenario = serde_json::from_value(serde_json::json!({
        "name":"ecology-audit-unit-fixture",
        "bounds":[0,2,0,2],
        "founders":[{"species":"Herbivore","pos":{"x":0,"y":0}}],
        "sites":[
            {"pos":{"x":1,"y":1},"kind":"Food","capacity":4,"replenish":1,"interval":1},
            {"pos":{"x":2,"y":2},"kind":"Water","capacity":4,"replenish":1,"interval":1}
        ],
        "resources":[
            [{"x":1,"y":1},{"Food":{"amount":2}}],
            [{"x":2,"y":2},{"Water":{"amount":2}}]
        ]
    }))
    .unwrap();
    scenario.build(123).unwrap()
}

#[test]
fn bounded_world_resource_flow_reconciles_and_kind_replacement_is_visible() {
    let mut world = tiny_world();
    let mut audit = ecology_audit::Audit::default();
    audit.seed(&world);
    let mut stock = ecology_audit::stock(&world);
    let mut cells = ecology_audit::resource_cells(&world);
    for tick in 1..=4 {
        let events = world.tick();
        (stock, cells) = audit.observe_tick(tick, stock, &cells, &world, &events);
    }
    assert_eq!(audit.reconciliation.len(), 4);
    for row in &audit.reconciliation {
        assert_eq!(row["food"]["unexplained_residual"], 0);
        assert_eq!(row["water"]["unexplained_residual"], 0);
    }

    let previous = ecology_audit::stock(&world);
    let previous_cells = ecology_audit::resource_cells(&world);
    world
        .map
        .set_cell(Position::new(1, 1), Cell::Water { amount: 2 });
    let (end_stock, _) = audit.observe_tick(5, previous, &previous_cells, &world, &[]);
    assert_ne!(end_stock.0, previous.0);
    assert_eq!(audit.replacements.len(), 1);
    assert_eq!(
        audit.replacements[0]["status"],
        "unexplained_resource_kind_replacement"
    );
    assert_ne!(
        audit.reconciliation.last().unwrap()["food"]["unexplained_residual"],
        0
    );
}

#[test]
fn eventless_body_energy_is_unknown_and_dead_founder_ledger_survives_removal() {
    let mut world = tiny_world();
    let mut audit = ecology_audit::Audit::default();
    audit.seed(&world);
    let result = audit.result(&world);
    assert_eq!(
        result["energy_attribution"]["basal_metabolism"],
        "unknown_not_emitted_as_an_event"
    );
    let death = [SimulationEvent::EntityDied {
        id: 1,
        pos: Position::new(0, 0),
        species: curious::entity::species::Species::Herbivore,
        cause: Some(curious::simulation::event::DeathCause::Starvation),
    }];
    let stock = ecology_audit::stock(&world);
    let cells = ecology_audit::resource_cells(&world);
    audit.observe_tick(1, stock, &cells, &world, &death);
    world.entities.clear();
    assert_eq!(audit.ledger.len(), 1);
    let result = audit.result(&world);
    assert_eq!(result["individual_ledger"][0]["founder"], true);
    assert_eq!(result["individual_ledger"][0]["died_from"], "Starvation");
}

#[test]
fn corpse_food_and_plant_food_are_distinct_event_pools() {
    let world = tiny_world();
    let mut audit = ecology_audit::Audit::default();
    audit.seed(&world);
    let events = [
        SimulationEvent::EntityAte {
            corpse_id: None,
            id: 1,
            pos: Position::new(1, 1),
            amount: 3,
        },
        SimulationEvent::EntityAte {
            corpse_id: Some(901),
            id: 1,
            pos: Position::new(1, 1),
            amount: 5,
        },
        SimulationEvent::EntityDrank {
            id: 1,
            pos: Position::new(2, 2),
            amount: 2,
        },
        SimulationEvent::EntityMoved {
            id: 1,
            energy_cost: 7,
            from: Position::new(0, 0),
            to: Position::new(1, 0),
        },
        SimulationEvent::EntityMetabolized {
            id: 1,
            vision_cost: 2,
            stamina_cost: 1,
            vision_paid: 1,
            stamina_paid: 1,
        },
    ];
    let before = ecology_audit::stock(&world);
    let cells = ecology_audit::resource_cells(&world);
    audit.observe_tick(1, before, &cells, &world, &events);
    assert_eq!(audit.food_meals, 3);
    assert_eq!(audit.corpse_meals, 5);
    assert_eq!(audit.reconciliation[0]["food"]["plant_meals"], 3);
    assert_eq!(audit.eaten_plant_by_species["Herbivore"], 3);
    assert_eq!(audit.eaten_corpse_by_species["Herbivore"], 5);
    assert_eq!(audit.drank_by_species["Herbivore"], 2);
    assert_eq!(audit.movement_by_species["Herbivore"]["energy_paid"], 7);
    assert_eq!(audit.traits_by_species["Herbivore"]["vision_requested"], 2);
    assert_eq!(audit.traits_by_species["Herbivore"]["vision_paid"], 1);
}

#[test]
fn observer_preserves_deterministic_world_and_rng_continuation() {
    let mut observed = tiny_world();
    let mut control = tiny_world();
    let mut audit = ecology_audit::Audit::default();
    audit.seed(&observed);
    let mut stock = ecology_audit::stock(&observed);
    let mut cells = ecology_audit::resource_cells(&observed);
    for tick in 1..=4 {
        let observed_events = observed.tick();
        let control_events = control.tick();
        (stock, cells) = audit.observe_tick(tick, stock, &cells, &observed, &observed_events);
        assert_eq!(
            serde_json::to_value(curious::simulation::checkpoint::State::capture(&observed))
                .unwrap(),
            serde_json::to_value(curious::simulation::checkpoint::State::capture(&control))
                .unwrap(),
        );
        assert_eq!(observed_events.len(), control_events.len());
    }
}

#[test]
fn parent_events_distinguish_f1_breeders_f2_and_living_bred_descendants() {
    let scenario: Scenario = serde_json::from_value(serde_json::json!({
        "name":"ecology-lineage-event-fixture",
        "bounds":[0,4,0,4],
        "founders":[
            {"species":"Herbivore","pos":{"x":0,"y":0}},
            {"species":"Herbivore","pos":{"x":2,"y":2}},
            {"species":"Herbivore","pos":{"x":4,"y":4}},
            {"species":"Herbivore","pos":{"x":0,"y":4}}
        ],
        "sites":[],"resources":[]
    }))
    .unwrap();
    let mut world = scenario.build(123).unwrap();
    let mut grandchild = world.entities.pop().unwrap();
    let mut child = world.entities.pop().unwrap();
    child.generation = 1;
    child.parents = vec![1, 2];
    grandchild.generation = 2;
    grandchild.parents = vec![1, 3];

    let mut audit = ecology_audit::Audit::default();
    audit.seed(&world);
    child.generation = 1;
    world.entities.push(child);
    let first = [
        SimulationEvent::EntityMated {
            parent1: 1,
            parent2: 2,
            child: 3,
            pos: Position::new(1, 1),
        },
        SimulationEvent::EntityBorn {
            id: 3,
            pos: Position::new(1, 1),
            species: curious::entity::species::Species::Herbivore,
        },
    ];
    audit.observe_tick(
        1,
        ecology_audit::stock(&world),
        &ecology_audit::resource_cells(&world),
        &world,
        &first,
    );
    world.entities.push(grandchild);
    let second = [
        SimulationEvent::EntityMated {
            parent1: 1,
            parent2: 3,
            child: 4,
            pos: Position::new(3, 3),
        },
        SimulationEvent::EntityBorn {
            id: 4,
            pos: Position::new(3, 3),
            species: curious::entity::species::Species::Herbivore,
        },
    ];
    audit.observe_tick(
        2,
        ecology_audit::stock(&world),
        &ecology_audit::resource_cells(&world),
        &world,
        &second,
    );
    let result = audit.result(&world);
    assert_eq!(result["births_by_generation"]["1"], 1);
    assert_eq!(result["f2_births"], 1);
    assert_eq!(result["f1_breeders_ever"], 1);
    assert_eq!(result["living_bred_descendants_at_end"], 1);
    assert_eq!(result["parent_event_issues"].as_array().unwrap().len(), 0);
    assert_eq!(result["individual_ledger"].as_array().unwrap().len(), 4);
    assert!(result["individual_ledger"][2]["snapshot"]["genotype"].is_object());
    assert!(result["individual_ledger"][3]["snapshot"]["phenotype"].is_object());
}

#[test]
fn actual_world_resolver_mating_records_offspring_genotype_and_phenotype() {
    let make_slot = |id, x| {
        let mut body = HerbivoreEntity::default();
        body.life_mut().age = 30;
        let mut slot = EntitySlot::new(
            id,
            Position::new(x, 0),
            EntityPhase::Active,
            EntityController::Player(0),
            Box::new(body),
        );
        slot.genome = Genome {
            mobility: (id - 1) as u8,
            vision: id as u8,
            stamina: 1,
        };
        slot.genome
            .express_extra_from(&Genome::default(), slot.base.life_mut());
        slot
    };
    let mut world = World::with_seed(0, 4, 0, 4, vec![make_slot(1, 0), make_slot(2, 1)], 123);
    world.map = curious::map::Map::new(0, 4, 0, 4);
    world.environment = Default::default();
    world.logger = curious::logger::Logger::disabled();
    world.genetics.extra_traits_enabled = true;
    world.genetics.mutation_per_mille = 0;
    world.push_command(1, Intent::Mate { target_id: 2 });
    world.push_command(2, Intent::Mate { target_id: 1 });

    let mut audit = ecology_audit::Audit::default();
    audit.seed(&world);
    let opening = ecology_audit::stock(&world);
    let cells = ecology_audit::resource_cells(&world);
    let events = world.tick();
    assert!(events.iter().any(|e| matches!(
        e,
        SimulationEvent::EntityMated {
            parent1,
            parent2,
            child: 3,
            ..
        } if (*parent1 == 1 && *parent2 == 2) || (*parent1 == 2 && *parent2 == 1)
    )));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, SimulationEvent::EntityBorn { id: 3, .. }))
    );
    audit.observe_tick(1, opening, &cells, &world, &events);
    let result = audit.result(&world);
    assert_eq!(result["births_by_generation"]["1"], 1);
    assert_eq!(result["parent_event_issues"].as_array().unwrap().len(), 0);
    assert!(result["individual_ledger"][2]["snapshot"]["genotype"].is_object());
    assert!(result["individual_ledger"][2]["snapshot"]["phenotype"]["vision_range"].is_number());
    assert!(
        result["trait_maintenance_by_species"]["Herbivore"]["vision_requested"]
            .as_u64()
            .unwrap()
            > 0
    );
}

#[test]
fn unknown_generation_and_unknown_parent_are_not_rewritten_as_generation_zero() {
    let world = tiny_world();
    let mut audit = ecology_audit::Audit::default();
    audit.seed(&world);
    let events = [
        SimulationEvent::EntityMated {
            parent1: 1,
            parent2: 999,
            child: 998,
            pos: Position::new(1, 1),
        },
        SimulationEvent::EntityBorn {
            id: 998,
            pos: Position::new(1, 1),
            species: curious::entity::species::Species::Herbivore,
        },
    ];
    let stock = ecology_audit::stock(&world);
    let cells = ecology_audit::resource_cells(&world);
    audit.observe_tick(1, stock, &cells, &world, &events);
    assert_eq!(audit.unknown_generation_births, 1);
    assert!(!audit.births_by_generation.contains_key(&0));
    assert!(
        audit
            .parent_event_issues
            .iter()
            .any(|row| row["kind"] == "unknown_parent")
    );
}

#[test]
fn parent_event_generation_mismatch_is_reported_against_actual_child_slot() {
    let mut world = tiny_world();
    let mut child = EntitySlot::new(
        2,
        Position::new(1, 0),
        EntityPhase::Active,
        EntityController::Player(0),
        Box::new(HerbivoreEntity::default()),
    );
    child.generation = 2;
    child.parents = vec![1, 1];
    let mut audit = ecology_audit::Audit::default();
    audit.seed(&world);
    world.entities.push(child);
    let events = [
        SimulationEvent::EntityMated {
            parent1: 1,
            parent2: 1,
            child: 2,
            pos: Position::new(1, 0),
        },
        SimulationEvent::EntityBorn {
            id: 2,
            pos: Position::new(1, 0),
            species: curious::entity::species::Species::Herbivore,
        },
    ];
    let stock = ecology_audit::stock(&world);
    let cells = ecology_audit::resource_cells(&world);
    audit.observe_tick(1, stock, &cells, &world, &events);
    assert!(
        audit
            .parent_event_issues
            .iter()
            .any(|row| row["kind"] == "parent_ledger_mismatch")
    );
    assert_eq!(audit.births_by_generation.get(&2), Some(&1));
}

#[test]
fn reconciliation_detail_storage_is_bounded_but_summary_counts_every_row() {
    let world = tiny_world();
    let mut audit = ecology_audit::Audit::default();
    let stock = ecology_audit::stock(&world);
    let cells = ecology_audit::resource_cells(&world);
    for tick in 1..=2_001 {
        audit.observe_tick(tick, stock, &cells, &world, &[]);
    }
    let result = audit.result(&world);
    assert_eq!(result["resource_reconciliation"]["ticks_checked"], 2_001);
    assert_eq!(
        result["resource_reconciliation"]["detail_rows_retained"],
        2_000
    );
}

#[test]
fn founder_f1_f2_and_reproducing_lineage_metrics_reconcile_across_generations() {
    let make_founder = |id: usize, x: isize, y: isize| {
        let mut body = HerbivoreEntity::default();
        let life = body.life_mut();
        life.age = 10;
        life.maturity_age = 2;
        life.energy = 80;
        life.water = 80;
        life.health = 100;
        life.stamina = life.max_stamina;
        EntitySlot::new(
            id,
            Position::new(x, y),
            EntityPhase::Active,
            EntityController::Player(0),
            Box::new(body),
        )
    };

    let founders = vec![
        make_founder(1, 1, 1),
        make_founder(2, 2, 1),
        make_founder(3, 4, 1),
        make_founder(4, 3, 1),
        make_founder(5, 5, 1),
        make_founder(6, 6, 1),
        make_founder(7, 7, 1),
    ];

    let mut world = World::with_seed(0, 8, 0, 8, founders, 42);
    world.map = curious::map::Map::new(0, 8, 0, 8);
    world.environment = curious::environment::Environment::default();
    world.logger = curious::logger::Logger::disabled();

    world.environment.set_site(ResourceSite {
        pos: Position::new(2, 2),
        kind: ResourceKind::Food,
        capacity: 10,
        replenish: 1,
        interval: 1,
        schedule: None,
    });
    world.environment.set_site(ResourceSite {
        pos: Position::new(5, 2),
        kind: ResourceKind::Food,
        capacity: 10,
        replenish: 1,
        interval: 1,
        schedule: None,
    });
    world.environment.set_site(ResourceSite {
        pos: Position::new(6, 3),
        kind: ResourceKind::Food,
        capacity: 10,
        replenish: 1,
        interval: 1,
        schedule: None,
    });
    world.environment.set_site(ResourceSite {
        pos: Position::new(3, 3),
        kind: ResourceKind::Water,
        capacity: 10,
        replenish: 1,
        interval: 1,
        schedule: None,
    });
    world.environment.set_site(ResourceSite {
        pos: Position::new(4, 3),
        kind: ResourceKind::Water,
        capacity: 10,
        replenish: 1,
        interval: 1,
        schedule: None,
    });
    world.environment.set_site(ResourceSite {
        pos: Position::new(7, 2),
        kind: ResourceKind::Water,
        capacity: 10,
        replenish: 1,
        interval: 1,
        schedule: None,
    });

    world
        .map
        .set_cell(Position::new(2, 2), Cell::Food { amount: 4 });
    world
        .map
        .set_cell(Position::new(5, 2), Cell::Food { amount: 4 });
    world
        .map
        .set_cell(Position::new(6, 3), Cell::Food { amount: 4 });
    world
        .map
        .set_cell(Position::new(3, 3), Cell::Water { amount: 4 });
    world
        .map
        .set_cell(Position::new(4, 3), Cell::Water { amount: 4 });
    world
        .map
        .set_cell(Position::new(7, 2), Cell::Water { amount: 4 });

    let mut audit = ecology_audit::Audit::default();
    audit.seed(&world);
    let mut stock = ecology_audit::stock(&world);
    let mut cells = ecology_audit::resource_cells(&world);

    // Tick 1: Pair 1 (1 and 2), Pair 2 (4 and 3), Pair 3 (5 and 6) mate. Founder 7 idles.
    world.push_command(1, Intent::Mate { target_id: 2 });
    world.push_command(4, Intent::Mate { target_id: 3 });
    world.push_command(5, Intent::Mate { target_id: 6 });

    let events = world.tick();
    (stock, cells) = audit.observe_tick(1, stock, &cells, &world, &events);
    for slot in &mut world.entities {
        slot.controller = EntityController::Player(0);
    }

    // Tick 2: F1 offspring 10 moves right to (3, 2). Offspring 8 and Founder 7 drink water, 9 eats food.
    world.push_command(
        10,
        Intent::Move {
            steps: Steps(vec![Direction::Right]),
        },
    );
    world.push_command(
        8,
        Intent::Drink {
            at: Steps(vec![Direction::Down]),
        },
    );
    world.push_command(
        9,
        Intent::Eat {
            at: Steps(vec![Direction::Down]),
            corpse_id: None,
        },
    );
    world.push_command(
        7,
        Intent::Drink {
            at: Steps(vec![Direction::Down]),
        },
    );

    let events = world.tick();
    (stock, cells) = audit.observe_tick(2, stock, &cells, &world, &events);

    // Tick 3: F1 offspring 10 drinks water, F1 offspring 8 eats food.
    world.push_command(
        10,
        Intent::Drink {
            at: Steps(vec![Direction::Down]),
        },
    );
    world.push_command(
        8,
        Intent::Eat {
            at: Steps(vec![Direction::Right]),
            corpse_id: None,
        },
    );

    let events = world.tick();
    (stock, cells) = audit.observe_tick(3, stock, &cells, &world, &events);

    // Tick 4: F1 offspring 10 eats food to satisfy reproduction energy reserve.
    world.push_command(
        10,
        Intent::Eat {
            at: Steps(vec![Direction::Left]),
            corpse_id: None,
        },
    );

    let events = world.tick();
    (stock, cells) = audit.observe_tick(4, stock, &cells, &world, &events);

    // Tick 5: F1 offspring 10 and 8 mate to produce F2 offspring 11.
    // F1 offspring 9 and Founder 7 do not mate.
    world.push_command(10, Intent::Mate { target_id: 8 });
    world.push_command(8, Intent::Mate { target_id: 10 });

    let events = world.tick();
    (stock, cells) = audit.observe_tick(5, stock, &cells, &world, &events);
    for slot in &mut world.entities {
        slot.controller = EntityController::Player(0);
    }

    // Tick 6: Passive observation tick. All survivors remain stable.
    let events = world.tick();
    (stock, cells) = audit.observe_tick(6, stock, &cells, &world, &events);
    let _ = (stock, cells);

    // --- Cohort Separation Verification ---
    let founders_cohort = audit.generation_cohort(0);
    let f1_cohort = audit.generation_cohort(1);
    let f2_cohort = audit.generation_cohort(2);

    let founder_ids: BTreeSet<usize> = founders_cohort.iter().map(|ind| ind.id).collect();
    let f1_ids: BTreeSet<usize> = f1_cohort.iter().map(|ind| ind.id).collect();
    let f2_ids: BTreeSet<usize> = f2_cohort.iter().map(|ind| ind.id).collect();

    assert_eq!(founder_ids, BTreeSet::from([1, 2, 3, 4, 5, 6, 7]));
    assert_eq!(f1_ids, BTreeSet::from([8, 9, 10]));
    assert_eq!(f2_ids, BTreeSet::from([11]));

    assert!(founder_ids.is_disjoint(&f1_ids));
    assert!(founder_ids.is_disjoint(&f2_ids));
    assert!(f1_ids.is_disjoint(&f2_ids));

    // --- Reproducing Individuals vs Non-reproducing Survivors ---
    let reproducing: BTreeSet<usize> = audit
        .reproducing_individuals()
        .into_iter()
        .map(|ind| ind.id)
        .collect();
    assert_eq!(reproducing, BTreeSet::from([1, 2, 3, 4, 5, 6, 8, 10]));

    let non_reproducing: BTreeSet<usize> = audit
        .non_reproducing_survivors(&world)
        .into_iter()
        .map(|ind| ind.id)
        .collect();
    assert_eq!(non_reproducing, BTreeSet::from([7, 9, 11]));

    // --- Lineage Descendants Exact Matching ---
    assert_eq!(audit.lineage_descendants(1), BTreeSet::from([10, 11]));
    assert_eq!(audit.lineage_descendants(2), BTreeSet::from([10, 11]));
    assert_eq!(audit.lineage_descendants(3), BTreeSet::from([8, 11]));
    assert_eq!(audit.lineage_descendants(4), BTreeSet::from([8, 11]));
    assert_eq!(audit.lineage_descendants(5), BTreeSet::from([9]));
    assert_eq!(audit.lineage_descendants(6), BTreeSet::from([9]));
    assert_eq!(audit.lineage_descendants(7), BTreeSet::new());

    // --- Stock-Flow Conservation Verification ---
    assert_eq!(audit.reconciliation_ticks, 6);
    assert_eq!(audit.food_residual_nonzero_ticks, 0);
    assert_eq!(audit.water_residual_nonzero_ticks, 0);
    assert_eq!(audit.food_residual_total, 0);
    assert_eq!(audit.water_residual_total, 0);
    assert!(audit.first_reconciliation_anomaly.is_none());
    assert!(audit.last_reconciliation_anomaly.is_none());
    assert!(audit.replacements.is_empty());
    assert!(audit.parent_event_issues.is_empty());
    assert!(audit.unattributed_actor_events.is_empty());

    for row in &audit.reconciliation {
        assert_eq!(row["food"]["unexplained_residual"], 0);
        assert_eq!(row["water"]["unexplained_residual"], 0);
    }

    // --- Audit Summary Metric Verification ---
    let result = audit.result(&world);
    assert_eq!(result["births_by_generation"]["1"], 3);
    assert_eq!(result["births_by_generation"]["2"], 1);
    assert_eq!(result["f1_breeders_ever"], 2);
    assert_eq!(result["f2_births"], 1);
    assert_eq!(result["unknown_generation_births"], 0);
}
