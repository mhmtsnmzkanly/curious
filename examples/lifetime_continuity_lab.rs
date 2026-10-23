//! Bounded same-individual resource/predator reversal and checkpoint study.
//! See docs/LIFETIME_CONTINUITY_PROTOCOL.md before running.
use curious::{
    creatures::{carnivore::CarnivoreEntity, herbivore::HerbivoreEntity},
    entity::{controller::EntityController, intent::Intent, phase::EntityPhase},
    environment::{Environment, ResourceKind, ResourceSite},
    logger::Logger,
    map::{
        Map,
        cell::Cell,
        movement::{Direction, Position, Steps},
    },
    simulation::checkpoint::State,
    world::{EntitySlot, World},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    io::{self, Write},
    process,
};

const SUBJECT_ID: usize = 1;
const PREDATOR_ID: usize = 2;
const SPLIT_TICK: usize = 52;
const SECOND_REVERSAL_TICK: usize = 78;
const END_TICK: usize = 104;
const SEEDS: [u64; 8] = [44001, 44002, 44003, 44004, 44005, 44006, 44007, 44008];

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ControllerCursor {
    direction: i8,
}

#[derive(Serialize)]
struct TickRow {
    tick: usize,
    events: Value,
    decisions: Value,
    state: Value,
    controller: ControllerCursor,
    interventions_after: Vec<&'static str>,
}

fn point(x: isize, y: isize) -> Position {
    Position::new(x, y)
}

fn make_world(seed: u64) -> World {
    let mut carnivore = CarnivoreEntity::default();
    // Keep the scripted predator alive through the fixed observation window;
    // the tested individual's ordinary species body is left untouched.
    carnivore.life_state.energy = 2_000;
    carnivore.life_state.max_energy = 2_000;
    carnivore.life_state.water = 2_000;
    carnivore.life_state.max_water = 2_000;
    let entities = vec![
        EntitySlot::new(
            SUBJECT_ID,
            point(3, 3),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        ),
        EntitySlot::new(
            PREDATOR_ID,
            point(0, 1),
            EntityPhase::Active,
            EntityController::Player(0),
            Box::new(carnivore),
        ),
    ];
    let mut world = World::with_seed(0, 6, 0, 6, entities, seed);
    world.logger = Logger::disabled();
    world.map = Map::new(0, 6, 0, 6);
    world.map.set_cell(point(1, 3), Cell::Water { amount: 12 });
    world.map.set_cell(point(5, 3), Cell::Water { amount: 12 });
    world.map.set_cell(point(3, 0), Cell::Food { amount: 20 });
    world.map.set_cell(point(3, 6), Cell::Food { amount: 20 });
    world.environment = Environment {
        sites: vec![
            ResourceSite {
                pos: point(1, 3),
                kind: ResourceKind::Water,
                capacity: 30,
                replenish: 4,
                interval: 2,
                schedule: None,
            },
            ResourceSite {
                pos: point(5, 3),
                kind: ResourceKind::Water,
                capacity: 30,
                replenish: 1,
                interval: 4,
                schedule: None,
            },
            ResourceSite {
                pos: point(3, 0),
                kind: ResourceKind::Food,
                capacity: 20,
                replenish: 1,
                interval: 2,
                schedule: None,
            },
            ResourceSite {
                pos: point(3, 6),
                kind: ResourceKind::Food,
                capacity: 20,
                replenish: 1,
                interval: 2,
                schedule: None,
            },
        ],
    };
    world.observation.full_for = vec![SUBJECT_ID];
    world
}

fn subject_alive(world: &World) -> bool {
    world
        .entities
        .iter()
        .find(|slot| slot.id == SUBJECT_ID)
        .is_some_and(|slot| slot.base.life().is_alive() && !slot.phase.is_corpse())
}

fn set_water_regime(world: &mut World, reversed: bool) {
    let west_index = world
        .environment
        .sites
        .iter()
        .position(|site| site.pos == point(1, 3))
        .unwrap();
    let east_index = world
        .environment
        .sites
        .iter()
        .position(|site| site.pos == point(5, 3))
        .unwrap();
    if reversed {
        world.environment.sites[west_index].replenish = 1;
        world.environment.sites[west_index].interval = 4;
        world.environment.sites[east_index].replenish = 4;
        world.environment.sites[east_index].interval = 2;
    } else {
        world.environment.sites[west_index].replenish = 4;
        world.environment.sites[west_index].interval = 2;
        world.environment.sites[east_index].replenish = 1;
        world.environment.sites[east_index].interval = 4;
    }
}

fn perimeter() -> Vec<Position> {
    let mut route = Vec::with_capacity(20);
    for x in 0..=6 {
        route.push(point(x, 1));
    }
    for y in 2..=5 {
        route.push(point(6, y));
    }
    for x in (0..6).rev() {
        route.push(point(x, 5));
    }
    for y in (2..5).rev() {
        route.push(point(0, y));
    }
    route
}

fn direction_between(from: Position, to: Position) -> Direction {
    match (to.x - from.x, to.y - from.y) {
        (1, 0) => Direction::Right,
        (-1, 0) => Direction::Left,
        (0, 1) => Direction::Down,
        (0, -1) => Direction::Up,
        other => panic!("perimeter route is not adjacent: {other:?}"),
    }
}

fn predator_command(world: &World, cursor: ControllerCursor) -> Intent {
    let route = perimeter();
    let slot = world
        .entities
        .iter()
        .find(|slot| slot.id == PREDATOR_ID)
        .unwrap();
    let index = route
        .iter()
        .position(|p| *p == slot.pos)
        .expect("scripted predator left its fixed perimeter route");
    let next =
        (index as isize + cursor.direction as isize).rem_euclid(route.len() as isize) as usize;
    Intent::Move {
        steps: Steps(vec![direction_between(slot.pos, route[next])]),
    }
}

fn tick(world: &mut World, cursor: ControllerCursor) -> TickRow {
    if let Some(predator) = world.entities.iter().find(|slot| slot.id == PREDATOR_ID) {
        if predator.phase.is_active() && predator.base.life().is_alive() {
            world.push_command(PREDATOR_ID, predator_command(world, cursor));
        }
    }
    let events = serde_json::to_value(world.tick()).unwrap();
    TickRow {
        tick: world.tick_counter,
        events,
        decisions: serde_json::to_value(&world.last_decisions).unwrap(),
        state: serde_json::to_value(State::capture(world)).unwrap(),
        controller: cursor,
        interventions_after: Vec::new(),
    }
}

#[derive(Serialize)]
struct SeedResult {
    schema: &'static str,
    seed: u64,
    status: &'static str,
    initial_state: Value,
    uninterrupted: Vec<TickRow>,
    checkpoint_state: Option<Value>,
    checkpoint_controller: Option<ControllerCursor>,
    resumed_initial_state: Option<Value>,
    frozen_initial_state: Option<Value>,
    resumed: Vec<TickRow>,
    frozen: Vec<TickRow>,
    physical_ticks: usize,
}

fn run_seed(seed: u64) -> SeedResult {
    let mut world = make_world(seed);
    let initial_state = serde_json::to_value(State::capture(&world)).unwrap();
    let mut controller = ControllerCursor { direction: 1 };
    let mut uninterrupted = Vec::new();
    let mut checkpoint_state = None;
    let mut checkpoint_controller = None;
    let mut resumed_initial_state = None;
    let mut frozen_initial_state = None;
    let mut checkpoint: Option<State> = None;

    while world.tick_counter < END_TICK && subject_alive(&world) {
        let row = tick(&mut world, controller);
        let t = world.tick_counter;
        uninterrupted.push(row);
        if t == SPLIT_TICK && subject_alive(&world) {
            // The reversal and direction change apply before decision tick 53.
            uninterrupted.last_mut().unwrap().interventions_after = vec![
                "reverse_water_before_53",
                "reverse_predator_route_before_53",
                "capture_checkpoint_before_53",
            ];
            set_water_regime(&mut world, true);
            controller.direction = -1;
            let state = State::capture(&world);
            checkpoint_state = Some(serde_json::to_value(&state).unwrap());
            checkpoint_controller = Some(controller);
            checkpoint = Some(state);
        } else if t == SECOND_REVERSAL_TICK {
            uninterrupted.last_mut().unwrap().interventions_after = vec!["reverse_water_before_79"];
            set_water_regime(&mut world, false);
        }
    }

    let status = if checkpoint.is_none() && world.tick_counter == SPLIT_TICK {
        "died_at_checkpoint"
    } else if checkpoint.is_none() {
        "died_before_checkpoint"
    } else if subject_alive(&world) {
        "completed"
    } else {
        "died_after_checkpoint"
    };
    let mut resumed = Vec::new();
    let mut frozen = Vec::new();
    if let Some(state) = checkpoint {
        let cursor = checkpoint_controller.unwrap();
        let mut replay = state.clone().restore().unwrap();
        resumed_initial_state = Some(serde_json::to_value(State::capture(&replay)).unwrap());
        assert_eq!(resumed_initial_state.as_ref(), checkpoint_state.as_ref());
        let replay_cursor = cursor;
        while replay.tick_counter < END_TICK && subject_alive(&replay) {
            let row = tick(&mut replay, replay_cursor);
            let t = replay.tick_counter;
            resumed.push(row);
            if t == SECOND_REVERSAL_TICK {
                resumed.last_mut().unwrap().interventions_after = vec!["reverse_water_before_79"];
                set_water_regime(&mut replay, false);
            }
        }
        assert_eq!(
            uninterrupted
                .iter()
                .filter(|row| row.tick > SPLIT_TICK)
                .map(|row| (&row.events, &row.decisions, &row.state, row.controller))
                .collect::<Vec<_>>(),
            resumed
                .iter()
                .map(|row| (&row.events, &row.decisions, &row.state, row.controller))
                .collect::<Vec<_>>(),
            "uninterrupted and restored online branches diverged for seed {seed}"
        );

        // Freeze exactly the existing water learner from the tick-52 checkpoint.
        let mut frozen_state = state.clone();
        let subject = frozen_state
            .entities
            .iter_mut()
            .find(|entity| entity.id == SUBJECT_ID)
            .unwrap();
        subject.learner.update_enabled = false;
        let mut frozen_world = frozen_state.restore().unwrap();
        frozen_initial_state = Some(serde_json::to_value(State::capture(&frozen_world)).unwrap());
        let frozen_cursor = cursor;
        while frozen_world.tick_counter < END_TICK && subject_alive(&frozen_world) {
            let row = tick(&mut frozen_world, frozen_cursor);
            let t = frozen_world.tick_counter;
            frozen.push(row);
            if t == SECOND_REVERSAL_TICK {
                frozen.last_mut().unwrap().interventions_after = vec!["reverse_water_before_79"];
                set_water_regime(&mut frozen_world, false);
            }
        }
    }
    let physical_ticks = uninterrupted.len() + resumed.len() + frozen.len();
    SeedResult {
        schema: "curious-lifetime-lab-v1",
        seed,
        status,
        initial_state,
        uninterrupted,
        checkpoint_state,
        checkpoint_controller,
        resumed_initial_state,
        frozen_initial_state,
        resumed,
        frozen,
        physical_ticks,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 1 {
        eprintln!("usage: lifetime_continuity_lab (fixed seeds 44001-44008)");
        process::exit(2);
    }
    let stdout = io::stdout();
    let mut output = stdout.lock();
    for seed in SEEDS {
        let row = run_seed(seed);
        serde_json::to_writer(&mut output, &row).unwrap();
        output.write_all(b"\n").unwrap();
    }
}
