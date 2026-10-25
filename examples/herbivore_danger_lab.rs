//! Bounded continuous-life refuge-memory lab. See the preregistration.
use curious::{
    creatures::{carnivore::CarnivoreEntity, herbivore::HerbivoreEntity},
    entity::{controller::EntityController, intent::Intent, phase::EntityPhase, species::Species},
    environment::{Environment, ResourceKind, ResourceSite},
    logger::Logger,
    map::{
        Map,
        cell::Cell,
        movement::{Direction, Position, Steps},
    },
    simulation::{SimulationEvent, checkpoint::State},
    systems::perception::PerceptionSystem,
    world::{EntitySlot, World},
};
use serde::Serialize;
use serde_json::Value;
use std::{
    io::{self, Write},
    process,
};
#[path = "support/refuge_episode_audit.rs"]
mod refuge_episode_audit;
use refuge_episode_audit::{AuditedEpisode, Tracker};

const SUBJECT_ID: usize = 1;
const PREDATOR_ID: usize = 2;
const END_TICK: usize = 100;
const CHECKPOINT_TICK: usize = 50;
const CALIBRATION_SEED: u64 = 45999;
const CALIBRATION_SEEDS: [u64; 1] = [CALIBRATION_SEED];
const SEEDS: [u64; 8] = [45001, 45002, 45003, 45004, 45005, 45006, 45007, 45008];
const PATROL: [(isize, isize); 24] = [
    (3, 0),
    (2, 0),
    (1, 0),
    (1, 1),
    (1, 2),
    (2, 2),
    (3, 2),
    (4, 2),
    (5, 2),
    (6, 2),
    (6, 3),
    (6, 4),
    (5, 4),
    (4, 4),
    (3, 4),
    (2, 4),
    (1, 4),
    (0, 4),
    (0, 3),
    (0, 2),
    (1, 2),
    (1, 1),
    (1, 0),
    (2, 0),
];

fn p(x: isize, y: isize) -> Position {
    Position::new(x, y)
}

fn make_world(seed: u64, refuge_outcomes_enabled: bool) -> World {
    let mut predator = CarnivoreEntity::default();
    predator.life_state.energy = 2_000;
    predator.life_state.max_energy = 2_000;
    predator.life_state.water = 2_000;
    predator.life_state.max_water = 2_000;
    predator.life_state.health = 2_000;
    predator.life_state.max_health = 2_000;
    predator.life_state.speed = 1;
    let mut subject = EntitySlot::new(
        SUBJECT_ID,
        p(3, 3),
        EntityPhase::Active,
        EntityController::AI,
        Box::new(HerbivoreEntity::default()),
    );
    subject.memory.refuge_outcomes_enabled = refuge_outcomes_enabled;
    let predator = EntitySlot::new(
        PREDATOR_ID,
        p(3, 0),
        EntityPhase::Active,
        EntityController::Player(0),
        Box::new(predator),
    );
    let mut world = World::with_seed(0, 6, 0, 6, vec![subject, predator], seed);
    world.logger = Logger::disabled();
    world.map = Map::new(0, 6, 0, 6);
    world.map.set_cell(p(1, 3), Cell::Burrow);
    world.map.set_cell(p(5, 3), Cell::Burrow);
    world.map.set_cell(p(3, 6), Cell::Food { amount: 20 });
    world.map.set_cell(p(6, 6), Cell::Water { amount: 12 });
    world.environment = Environment {
        sites: vec![
            ResourceSite {
                pos: p(3, 6),
                kind: ResourceKind::Food,
                capacity: 20,
                replenish: 1,
                interval: 2,
                schedule: None,
            },
            ResourceSite {
                pos: p(6, 6),
                kind: ResourceKind::Water,
                capacity: 30,
                replenish: 4,
                interval: 2,
                schedule: None,
            },
        ],
        ..Default::default()
    };
    world.observation.full_for = vec![SUBJECT_ID];
    world
}

fn subject_alive(world: &World) -> bool {
    world
        .entities
        .iter()
        .find(|s| s.id == SUBJECT_ID)
        .is_some_and(|s| s.base.life().is_alive() && !s.phase.is_corpse())
}

fn direction(from: Position, to: Position) -> Direction {
    match (to.x - from.x, to.y - from.y) {
        (1, 0) => Direction::Right,
        (-1, 0) => Direction::Left,
        (0, 1) => Direction::Down,
        (0, -1) => Direction::Up,
        other => panic!("patrol contains nonadjacent step {other:?}"),
    }
}

fn predator_intent(world: &World, patrol_index: usize) -> (Intent, Option<usize>) {
    let predator = world.entities.iter().find(|s| s.id == PREDATOR_ID).unwrap();
    assert_eq!(
        predator.pos,
        p(PATROL[patrol_index].0, PATROL[patrol_index].1),
        "patrol cursor drift"
    );
    let perception = PerceptionSystem::build(world, predator);
    if perception
        .entities
        .iter()
        .any(|e| e.id == SUBJECT_ID && e.species == Species::Herbivore && e.steps.len() <= 1)
    {
        return (
            Intent::Attack {
                target_id: SUBJECT_ID,
            },
            None,
        );
    }
    let next_index = (patrol_index + 1) % PATROL.len();
    let next = PATROL[next_index];
    (
        Intent::Move {
            steps: Steps(vec![direction(predator.pos, p(next.0, next.1))]),
        },
        Some(next_index),
    )
}

fn advance_patrol(index: usize, planned: Option<usize>, events: &[SimulationEvent]) -> usize {
    let Some(next_index) = planned else {
        return index;
    };
    if events.iter().any(|event| {
        matches!(event,
        SimulationEvent::EntityMoved { id: PREDATOR_ID, to, .. }
            if *to == p(PATROL[next_index].0, PATROL[next_index].1))
    }) {
        next_index
    } else {
        index
    }
}

#[derive(Serialize)]
struct TickRow {
    tick: usize,
    events: Value,
    decisions: Value,
    state: Value,
    memory_before: Value,
    memory_after: Value,
    patrol_index: usize,
    episode_audit: Vec<AuditedEpisode>,
}

#[derive(Serialize)]
struct RoundTrip {
    tick: usize,
    serialized_state: Value,
    restored_state: Value,
    state_equal: bool,
    rng_before: u64,
    rng_after: u64,
    patrol_index_before: usize,
    patrol_index_after: usize,
}

#[derive(Serialize)]
struct Branch {
    status: &'static str,
    initial_state: Value,
    timeline: Vec<TickRow>,
    checkpoint_roundtrip: Option<RoundTrip>,
}

fn run_branch(seed: u64, enabled: bool) -> Branch {
    let mut world = make_world(seed, enabled);
    let initial_state = serde_json::to_value(State::capture(&world)).unwrap();
    let mut timeline = Vec::new();
    let mut checkpoint_roundtrip = None;
    let mut patrol_index = 0usize;
    let mut tracker = Tracker::default();
    while world.tick_counter < END_TICK && subject_alive(&world) {
        let memory_before = serde_json::to_value(
            &world
                .entities
                .iter()
                .find(|s| s.id == SUBJECT_ID)
                .unwrap()
                .memory,
        )
        .unwrap();
        let (intent, move_index) = predator_intent(&world, patrol_index);
        world.push_command(PREDATOR_ID, intent);
        let mut events = world.tick();
        let trace = world
            .last_decisions
            .iter()
            .find(|trace| trace.id == SUBJECT_ID);
        tracker.before_resolver(trace, world.tick_counter);
        if let Some(subject) = world.entities.iter().find(|s| s.id == SUBJECT_ID) {
            let burrows = (0..=6)
                .flat_map(|y| (0..=6).map(move |x| p(x, y)))
                .filter(|position| world.map.cell(*position) == Some(&Cell::Burrow))
                .collect::<Vec<_>>();
            tracker.after_resolver(
                &events,
                subject.pos,
                subject.base.life(),
                subject_alive(&world),
                &burrows,
                world.tick_counter,
            );
        }
        if world.tick_counter == END_TICK && subject_alive(&world) {
            if let Some(subject) = world.entities.iter().find(|s| s.id == SUBJECT_ID) {
                tracker.censor_at_boundary(
                    subject.pos,
                    subject.base.life().health,
                    world.tick_counter,
                );
            }
            if let Some(subject) = world.entities.iter_mut().find(|s| s.id == SUBJECT_ID) {
                subject.memory.censor_refuge_at_boundary(
                    subject.pos,
                    subject.base.life().health,
                    world.tick_counter,
                );
            }
            events.extend(curious::systems::refuge_outcomes::finish(
                &mut world.entities,
                &world.map,
                &[],
                world.tick_counter,
            ));
        }
        patrol_index = advance_patrol(patrol_index, move_index, &events);
        timeline.push(TickRow {
            tick: world.tick_counter,
            events: serde_json::to_value(events).unwrap(),
            decisions: serde_json::to_value(&world.last_decisions).unwrap(),
            state: serde_json::to_value(State::capture(&world)).unwrap(),
            memory_before,
            memory_after: serde_json::to_value(
                &world
                    .entities
                    .iter()
                    .find(|s| s.id == SUBJECT_ID)
                    .unwrap()
                    .memory,
            )
            .unwrap(),
            patrol_index,
            episode_audit: tracker.take_closures(),
        });
        if world.tick_counter == CHECKPOINT_TICK {
            let serialized = serde_json::to_value(State::capture(&world)).unwrap();
            let rng_before = world.rng.state();
            let patrol_index_before = patrol_index;
            world = State::capture(&world).restore().unwrap();
            let restored = serde_json::to_value(State::capture(&world)).unwrap();
            let rng_after = world.rng.state();
            assert_eq!(serialized, restored, "tick-50 State roundtrip mismatch");
            assert_eq!(rng_before, rng_after, "tick-50 RNG roundtrip mismatch");
            assert_eq!(
                patrol_index_before, patrol_index,
                "external patrol cursor changed"
            );
            checkpoint_roundtrip = Some(RoundTrip {
                tick: CHECKPOINT_TICK,
                state_equal: true,
                serialized_state: serialized,
                restored_state: restored,
                rng_before,
                rng_after,
                patrol_index_before,
                patrol_index_after: patrol_index,
            });
        }
    }
    let died = !subject_alive(&world);
    Branch {
        status: if died {
            "natural_death"
        } else {
            "completed_100_ticks"
        },
        initial_state,
        timeline,
        checkpoint_roundtrip,
    }
}

#[derive(Serialize)]
struct SeedResult {
    schema: &'static str,
    seed: u64,
    baseline: Branch,
    refuge_memory: Branch,
    physical_ticks: usize,
    paired_initial_state_equal_except_feature: bool,
}

fn run_seed(seed: u64) -> SeedResult {
    let baseline = run_branch(seed, false);
    let refuge_memory = run_branch(seed, true);
    let mut normalized = refuge_memory.initial_state.clone();
    normalized["entities"][0]["memory"]["refuge_outcomes_enabled"] = Value::Bool(false);
    let paired_initial_state_equal_except_feature = normalized == baseline.initial_state;
    let physical_ticks = baseline.timeline.len() + refuge_memory.timeline.len();
    assert!(
        paired_initial_state_equal_except_feature,
        "paired initial world mismatch for seed {seed}"
    );
    assert!(physical_ticks <= END_TICK * 2, "seed tick budget exceeded");
    SeedResult {
        schema: "curious-herbivore-danger-lab-v1",
        seed,
        baseline,
        refuge_memory,
        physical_ticks,
        paired_initial_state_equal_except_feature,
    }
}

fn seeds_for_args(args: &[String]) -> Option<&'static [u64]> {
    match args {
        [] => Some(&SEEDS),
        [flag] if flag == "--calibration" => Some(&CALIBRATION_SEEDS),
        _ => None,
    }
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let Some(seeds) = seeds_for_args(&args) else {
        eprintln!("usage: herbivore_danger_lab [--calibration]");
        process::exit(2);
    };
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut total_ticks = 0;
    for seed in seeds.iter().copied() {
        let row = run_seed(seed);
        total_ticks += row.physical_ticks;
        let study_tick_ceiling = if args.is_empty() {
            SEEDS.len() * END_TICK * 2
        } else {
            END_TICK * 2
        };
        assert!(
            total_ticks <= study_tick_ceiling,
            "study tick budget exceeded"
        );
        serde_json::to_writer(&mut out, &row).unwrap();
        out.write_all(b"\n").unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patrol_cursor_completes_two_physical_cycles_through_resolver() {
        let mut predator = CarnivoreEntity::default();
        predator.life_state.energy = 2_000;
        predator.life_state.max_energy = 2_000;
        predator.life_state.water = 2_000;
        predator.life_state.max_water = 2_000;
        let actor = EntitySlot::new(
            PREDATOR_ID,
            p(PATROL[0].0, PATROL[0].1),
            EntityPhase::Active,
            EntityController::Player(0),
            Box::new(predator),
        );
        let mut world = World::with_seed(0, 6, 0, 6, vec![actor], 45001);
        world.logger = Logger::disabled();
        world.map = Map::new(0, 6, 0, 6);
        world.map.set_cell(p(1, 3), Cell::Burrow);
        world.map.set_cell(p(5, 3), Cell::Burrow);
        let mut cursor = 0;
        for _ in 0..(PATROL.len() * 2) {
            let (intent, planned) = predator_intent(&world, cursor);
            world.push_command(PREDATOR_ID, intent);
            let events = world.tick();
            let next_cursor = advance_patrol(cursor, planned, &events);
            assert_ne!(cursor, next_cursor, "pinned patrol move was rejected");
            cursor = next_cursor;
            let expected = PATROL[cursor];
            assert_eq!(world.entities[0].pos, p(expected.0, expected.1));
        }
        assert_eq!(cursor, 0);
    }

    #[test]
    fn external_cursor_does_not_advance_for_attack_or_rejected_move() {
        let target = PATROL[1];
        let moved = SimulationEvent::EntityMoved {
            id: PREDATOR_ID,
            energy_cost: 1,
            from: p(PATROL[0].0, PATROL[0].1),
            to: p(target.0, target.1),
        };
        assert_eq!(advance_patrol(0, None, &[moved.clone()]), 0);
        assert_eq!(advance_patrol(0, Some(1), &[]), 0);
        assert_eq!(advance_patrol(0, Some(1), &[moved]), 1);
    }

    #[test]
    fn calibration_mode_is_one_fixed_seed_and_formal_mode_stays_held_out() {
        assert_eq!(seeds_for_args(&[]).unwrap(), &SEEDS);
        assert_eq!(seeds_for_args(&["--calibration".into()]).unwrap(), &[45999]);
        assert!(seeds_for_args(&["--other".into()]).is_none());
        assert!(!SEEDS.contains(&CALIBRATION_SEED));
    }

    #[test]
    fn predator_attacks_only_a_currently_perceived_adjacent_subject() {
        let predator = EntitySlot::new(
            PREDATOR_ID,
            p(3, 0),
            EntityPhase::Active,
            EntityController::Player(0),
            Box::new(CarnivoreEntity::default()),
        );
        let subject = EntitySlot::new(
            SUBJECT_ID,
            p(3, 1),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        );
        let mut world = World::with_seed(0, 6, 0, 6, vec![subject, predator], 45001);
        world.logger = Logger::disabled();
        world.map = Map::new(0, 6, 0, 6);
        let (intent, planned) = predator_intent(&world, 0);
        assert!(matches!(
            intent,
            Intent::Attack {
                target_id: SUBJECT_ID
            }
        ));
        assert_eq!(planned, None);
    }
}
