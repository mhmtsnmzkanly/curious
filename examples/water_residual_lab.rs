//! Continuous-lifetime V17 source-residual water lab.
//!
//! Root preregistration supplies seed roles. The fixture uses ordinary AI and
//! the normal World/resolver path.
use curious::{
    creatures::herbivore::HerbivoreEntity,
    entity::{
        controller::EntityController,
        decision::DecisionTrace,
        foraging,
        intent::Intent,
        learning::{Learner, WaterEpisode},
        perception::PerceivedWater,
        phase::EntityPhase,
    },
    environment::{Environment, ResourceKind, ResourceSite},
    logger::Logger,
    map::{Map, cell::Cell, movement::Position},
    simulation::checkpoint::State,
    systems::intent_collection::IntentCollectionSystem,
    world::{EntitySlot, World},
};
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    io::{self, Write},
    process,
};

const FOCAL_ID: usize = 1;
const COMPETITOR_ID: usize = 2;
const FORK_TICK: usize = 200;
const END_TICK: usize = 500;

#[derive(Clone, Debug, Serialize)]
struct ChoiceCandidate {
    pos: Position,
    amount: usize,
    steps_len: usize,
    legacy_value: Option<i32>,
    legacy_visits: Option<u32>,
    residual_sum: i64,
    residual_count: u32,
    residual_mean: i64,
    current_baseline: i64,
    final_score: i64,
}

#[derive(Clone, Debug, Serialize)]
struct ChoiceExposure {
    base_reason: String,
    final_reason: String,
    intent: Value,
    fresh: bool,
    active_episode_before: Option<WaterEpisode>,
    candidates: Vec<ChoiceCandidate>,
    selected_target: Option<Position>,
    selection_kind: &'static str,
}

#[derive(Clone, Debug, Serialize)]
struct TickRow {
    tick: usize,
    state_before: Value,
    rng_before: u64,
    events: Value,
    decisions: Value,
    focal_choice: Option<ChoiceExposure>,
    learner_closures: LearnerClosureAudit,
    state_after: Value,
    rng_after: u64,
}

#[derive(Clone, Debug, Serialize)]
struct LearnerClosureAudit {
    predecision_closures: Vec<Value>,
    episode_after_choice: Option<WaterEpisode>,
    postresolver_closures: Vec<Value>,
    replayed_decision_matches_trace: bool,
    replayed_postdecision_rng_matches_world: bool,
    replayed_learner_matches_world: bool,
}

#[derive(Clone, Debug, Serialize)]
struct BranchRecord {
    status: &'static str,
    learner_treatment: &'static str,
    state_before_treatment: Value,
    state_after_treatment_before_reversal: Value,
    state_at_tick_201_before_world_tick: Value,
    ticks: Vec<TickRow>,
}

#[derive(Clone, Debug, Serialize)]
struct SeedRecord {
    schema: &'static str,
    protocol_version: u8,
    role: &'static str,
    seed: u64,
    status: &'static str,
    physical_ticks: usize,
    initial_state: Value,
    prefix: Vec<TickRow>,
    checkpoint_state: Value,
    forks: Forks,
}

#[derive(Clone, Debug, Serialize)]
struct Forks {
    trained_online: Option<BranchRecord>,
    trained_frozen: Option<BranchRecord>,
    naive_online: Option<BranchRecord>,
    rule_based: Option<BranchRecord>,
}

#[derive(Clone, Copy, Debug)]
enum Treatment {
    Online,
    TrainedFrozen,
    NaiveOnline,
    RuleBased,
}

impl Treatment {
    #[cfg(test)]
    fn name(self) -> &'static str {
        match self {
            Self::Online => "trained_online",
            Self::TrainedFrozen => "trained_frozen",
            Self::NaiveOnline => "naive_online",
            Self::RuleBased => "rule_based",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Online => "global_residual=true;enabled=true;update_enabled=true;retain-tables",
            Self::TrainedFrozen => {
                "global_residual=true;enabled=true;update_enabled=false;retain-tables"
            }
            Self::NaiveOnline => {
                "global_residual=true;enabled=true;update_enabled=true;clear-residuals-only"
            }
            Self::RuleBased => {
                "global_residual=true;enabled=false;update_enabled=false;ordinary-policy"
            }
        }
    }
}

fn p(x: isize, y: isize) -> Position {
    Position::new(x, y)
}

fn site(
    pos: Position,
    kind: ResourceKind,
    capacity: usize,
    replenish: usize,
    interval: usize,
) -> ResourceSite {
    ResourceSite {
        pos,
        kind,
        capacity,
        replenish,
        interval,
        schedule: None,
    }
}

fn actor(id: usize, pos: Position, focal: bool) -> EntitySlot {
    let mut body = HerbivoreEntity::default();
    // A disclosed initial-lifespan setting allows a 500-tick observation; it is
    // not changed during life and does not alter any age-dependent capability.
    body.life_state.max_age = 512;
    body.life_state.maturity_age = 501;
    let mut slot = EntitySlot::new(
        id,
        pos,
        EntityPhase::Active,
        EntityController::AI,
        Box::new(body),
    );
    slot.learner.enabled = focal;
    slot.learner.update_enabled = focal;
    slot
}

fn make_world(seed: u64) -> World {
    let entities = vec![
        actor(FOCAL_ID, p(2, 2), true),
        actor(COMPETITOR_ID, p(2, 3), false),
    ];
    let mut world = World::with_seed(0, 4, 0, 4, entities, seed);
    world.logger = Logger::disabled();
    world.map = Map::new(0, 4, 0, 4);
    for pos in [p(0, 0), p(4, 0), p(0, 4), p(4, 4)] {
        world.map.set_cell(pos, Cell::Food { amount: 20 });
    }
    world.map.set_cell(p(0, 2), Cell::Water { amount: 12 });
    world.map.set_cell(p(4, 2), Cell::Water { amount: 12 });
    world.environment = Environment {
        sites: vec![
            site(p(0, 2), ResourceKind::Water, 12, 4, 2),
            site(p(4, 2), ResourceKind::Water, 12, 1, 4),
            site(p(0, 0), ResourceKind::Food, 20, 2, 2),
            site(p(4, 0), ResourceKind::Food, 20, 2, 2),
            site(p(0, 4), ResourceKind::Food, 20, 2, 2),
            site(p(4, 4), ResourceKind::Food, 20, 2, 2),
        ],
        ..Default::default()
    };
    world.observation.full_for = vec![FOCAL_ID, COMPETITOR_ID];
    world.policy.water_residual_enabled = true;
    world
}

fn actor_alive(world: &World, id: usize) -> bool {
    world
        .entities
        .iter()
        .find(|s| s.id == id)
        .is_some_and(|s| s.base.life().is_alive() && !s.phase.is_corpse())
}

fn value(state: &State) -> Value {
    serde_json::to_value(state).expect("serialize state")
}

fn learner_from_state(state: &Value, id: usize) -> Option<Learner> {
    state["entities"]
        .as_array()?
        .iter()
        .find(|actor| actor["id"] == json!(id))
        .and_then(|actor| serde_json::from_value(actor["learner"].clone()).ok())
}

fn trace_reason(trace: &DecisionTrace) -> (String, String) {
    let final_reason = format!("{:?}", trace.decision.reason);
    let base_reason = if final_reason == "LearnedWater" {
        "SeekWater".to_owned()
    } else {
        final_reason.clone()
    };
    (base_reason, final_reason)
}

fn target_for(water: &PerceivedWater, origin: Position) -> Position {
    water
        .steps
        .0
        .iter()
        .fold(origin, |pos, direction| pos + *direction)
}

fn selected_target(trace: &DecisionTrace) -> Option<Position> {
    match &trace.decision.intent {
        Intent::Drink { at } => Some(
            at.0.iter()
                .fold(trace.pos, |pos, direction| pos + *direction),
        ),
        _ => None,
    }
}

fn choice_exposure(trace: &DecisionTrace, before: &Value) -> ChoiceExposure {
    let (base_reason, final_reason) = trace_reason(trace);
    let learner =
        learner_from_state(before, FOCAL_ID).expect("focal pre-choice learner is recorded");
    let active_episode_before = learner.episode.clone();
    let waters = trace
        .perception
        .as_ref()
        .map(|perception| &perception.waters);
    let candidates: Vec<_> = waters
        .into_iter()
        .flatten()
        .map(|water| {
            let pos = target_for(water, trace.pos);
            let legacy = learner.values.iter().find(|entry| entry.pos == pos);
            let residual = learner.residuals.iter().find(|entry| entry.pos == pos);
            let residual_mean = residual
                .filter(|entry| entry.visits > 0)
                .map_or(0, |entry| entry.sum / entry.visits as i64);
            let current_baseline =
                foraging::value(water.amount, water.steps.len(), &trace.before, true) as i64;
            ChoiceCandidate {
                pos,
                amount: water.amount,
                steps_len: water.steps.len(),
                legacy_value: legacy.map(|entry| entry.value),
                legacy_visits: legacy.map(|entry| entry.visits),
                residual_sum: residual.map_or(0, |entry| entry.sum),
                residual_count: residual.map_or(0, |entry| entry.visits),
                residual_mean,
                current_baseline,
                final_score: current_baseline + residual_mean,
            }
        })
        .collect();
    let candidate_has_active = active_episode_before.as_ref().is_some_and(|episode| {
        candidates
            .iter()
            .any(|candidate| candidate.pos == episode.target)
    });
    let base_is_water = base_reason == "SeekWater";
    let has_choice_set = base_is_water && candidates.len() >= 2;
    let ongoing = learner.enabled && final_reason == "LearnedWater" && candidate_has_active;
    let fresh = has_choice_set && !ongoing;
    let selection_kind = if !learner.enabled || !base_is_water {
        "base"
    } else if ongoing {
        "ongoing"
    } else if candidates.len() < 2 {
        "base"
    } else if trace.rng_before % 100 < 10 {
        "explore"
    } else {
        "greedy"
    };
    ChoiceExposure {
        base_reason,
        final_reason,
        intent: serde_json::to_value(&trace.decision.intent).expect("serialize focal intent"),
        fresh,
        active_episode_before,
        candidates,
        selected_target: selected_target(trace),
        selection_kind,
    }
}

fn tick_row(world: &mut World) -> TickRow {
    let before = value(&State::capture(world));
    let rng_before = world.rng.state();
    let events = world.tick();
    let decisions = serde_json::to_value(&world.last_decisions).expect("serialize decision traces");
    let focal_trace = world
        .last_decisions
        .iter()
        .find(|trace| trace.id == FOCAL_ID);
    let focal_choice = focal_trace.map(|trace| choice_exposure(trace, &before));
    let learner_closures = replay_learner_closures(world, &before, &events, focal_choice.as_ref());
    let after = value(&State::capture(world));
    let rng_after = world.rng.state();
    TickRow {
        tick: world.tick_counter,
        state_before: before,
        rng_before,
        events: serde_json::to_value(events).expect("serialize events"),
        decisions,
        focal_choice,
        learner_closures,
        state_after: after,
        rng_after,
    }
}

/// Replay only the focal learner's decision/outcome boundary on a cloned learner
/// and RNG. The real World tick and its RNG are never invoked by this audit.
fn replay_learner_closures(
    world_after: &World,
    state_before: &Value,
    events: &[curious::simulation::SimulationEvent],
    focal_choice: Option<&ChoiceExposure>,
) -> LearnerClosureAudit {
    let Some(actual) = world_after.entities.iter().find(|slot| slot.id == FOCAL_ID) else {
        return LearnerClosureAudit {
            predecision_closures: Vec::new(),
            episode_after_choice: None,
            postresolver_closures: Vec::new(),
            replayed_decision_matches_trace: false,
            replayed_postdecision_rng_matches_world: false,
            replayed_learner_matches_world: false,
        };
    };
    let mut replay_world = serde_json::from_value::<State>(state_before.clone())
        .expect("deserialize pre-tick V17 checkpoint")
        .restore()
        .expect("restore cloned pre-tick world");
    replay_world.tick_counter += 1;
    replay_world
        .entities
        .retain(|slot| !matches!(slot.phase, EntityPhase::Removed));
    let mut prelude_events = Vec::new();
    replay_world.environment.update(
        &mut replay_world.map,
        replay_world.tick_counter,
        &mut prelude_events,
    );
    curious::systems::life_cycle::LifeCycleSystem::begin_tick(
        &mut replay_world.entities,
        &mut replay_world.map,
        &mut prelude_events,
    );
    let learner_before =
        learner_from_state(state_before, FOCAL_ID).expect("focal learner exists before tick");
    let mut log = Vec::new();
    let intents = IntentCollectionSystem::collect(&mut replay_world, &mut log);
    let replay_trace = replay_world
        .last_decisions
        .iter()
        .find(|trace| trace.id == FOCAL_ID);
    let actual_trace = world_after
        .last_decisions
        .iter()
        .find(|trace| trace.id == FOCAL_ID);
    let replayed_decision_matches_trace = match (replay_trace, actual_trace) {
        (Some(replay), Some(actual)) => {
            replay.tick == actual.tick
                && serde_json::to_value(&replay.decision).ok()
                    == serde_json::to_value(&actual.decision).ok()
                && replay.pos == actual.pos
                && serde_json::to_value(&replay.before).ok()
                    == serde_json::to_value(&actual.before).ok()
                && serde_json::to_value(&replay.perception).ok()
                    == serde_json::to_value(&actual.perception).ok()
                && replay.rng_before == actual.rng_before
        }
        (None, None) => true,
        _ => false,
    };
    let replayed_postdecision_rng_matches_world =
        replay_world.rng.state() == world_after.rng.state();
    let predecision_closure_expected = replay_trace.is_some_and(|trace| {
        let Some(attempt) = &learner_before.episode else {
            return false;
        };
        let choice = focal_choice.expect("focal trace has captured choice evidence");
        if !learner_before.enabled || choice.base_reason != "SeekWater" {
            return true;
        }
        trace.perception.as_ref().is_none_or(|view| {
            !view
                .waters
                .iter()
                .any(|water| target_for(water, trace.pos) == attempt.target)
        })
    });
    let replay_decisions = replay_world.last_decisions.clone();
    let mut replay_slot = replay_world
        .entities
        .iter_mut()
        .find(|slot| slot.id == FOCAL_ID)
        .expect("focal clone exists for resolver replay");
    let mut predecision_closures = Vec::new();
    if predecision_closure_expected {
        predecision_closures.push(
            serde_json::to_value(&replay_slot.learner.last_episode)
                .expect("serialize predecision closure"),
        );
    }
    let episode_after_choice = replay_slot.learner.episode.clone();
    *replay_slot.base.life_mut() = actual.base.life().clone();
    replay_slot.pos = actual.pos;
    replay_slot.phase = actual.phase.clone();
    let had_resolver_episode = replay_slot.learner.episode.is_some();
    curious::systems::water_outcomes::finish(
        std::slice::from_mut(&mut replay_slot),
        &replay_decisions,
        &intents,
        events,
    );
    let mut postresolver_closures = Vec::new();
    if had_resolver_episode && replay_slot.learner.episode.is_none() {
        postresolver_closures.push(
            serde_json::to_value(&replay_slot.learner.last_episode)
                .expect("serialize postresolver closure"),
        );
    }
    let replayed_learner_matches_world = serde_json::to_value(&replay_slot.learner).ok()
        == serde_json::to_value(&actual.learner).ok();
    LearnerClosureAudit {
        predecision_closures,
        episode_after_choice,
        postresolver_closures,
        replayed_decision_matches_trace,
        replayed_postdecision_rng_matches_world,
        replayed_learner_matches_world,
    }
}

fn reverse_water_rates(world: &mut World) {
    for source in &mut world.environment.sites {
        if source.pos == p(0, 2) && source.kind == ResourceKind::Water {
            source.replenish = 1;
            source.interval = 4;
        } else if source.pos == p(4, 2) && source.kind == ResourceKind::Water {
            source.replenish = 4;
            source.interval = 2;
        }
    }
}

fn apply_treatment(world: &mut World, treatment: Treatment) {
    let focal = world
        .entities
        .iter_mut()
        .find(|actor| actor.id == FOCAL_ID)
        .expect("focal actor");
    match treatment {
        Treatment::Online => {
            focal.learner.enabled = true;
            focal.learner.update_enabled = true;
        }
        Treatment::TrainedFrozen => {
            focal.learner.enabled = true;
            focal.learner.update_enabled = false;
        }
        Treatment::NaiveOnline => {
            focal.learner.enabled = true;
            focal.learner.update_enabled = true;
            focal.learner.residuals.clear();
        }
        Treatment::RuleBased => {
            focal.learner.enabled = false;
            focal.learner.update_enabled = false;
        }
    }
}

fn run_branch(checkpoint: &State, treatment: Treatment) -> BranchRecord {
    let state_before_treatment = value(checkpoint);
    let mut world = checkpoint
        .clone()
        .restore()
        .expect("restore fork checkpoint");
    apply_treatment(&mut world, treatment);
    let state_after_treatment_before_reversal = value(&State::capture(&world));
    reverse_water_rates(&mut world);
    let state_at_tick_201_before_world_tick = value(&State::capture(&world));
    let mut ticks = Vec::new();
    while world.tick_counter < END_TICK && actor_alive(&world, FOCAL_ID) {
        ticks.push(tick_row(&mut world));
    }
    let status = if actor_alive(&world, FOCAL_ID) {
        "ceiling"
    } else {
        "focal_died"
    };
    BranchRecord {
        status,
        learner_treatment: treatment.description(),
        state_before_treatment,
        state_after_treatment_before_reversal,
        state_at_tick_201_before_world_tick,
        ticks,
    }
}

fn run_world(mut world: World, seed: u64, role: &'static str) -> SeedRecord {
    let initial_state = value(&State::capture(&world));
    let mut prefix = Vec::with_capacity(FORK_TICK);
    while world.tick_counter < FORK_TICK
        && actor_alive(&world, FOCAL_ID)
        && actor_alive(&world, COMPETITOR_ID)
    {
        prefix.push(tick_row(&mut world));
    }
    let checkpoint = State::capture(&world);
    let checkpoint_state = value(&checkpoint);
    let can_branch = actor_alive(&world, FOCAL_ID) && actor_alive(&world, COMPETITOR_ID);
    let forks = if can_branch {
        Forks {
            trained_online: Some(run_branch(&checkpoint, Treatment::Online)),
            trained_frozen: Some(run_branch(&checkpoint, Treatment::TrainedFrozen)),
            naive_online: Some(run_branch(&checkpoint, Treatment::NaiveOnline)),
            rule_based: Some(run_branch(&checkpoint, Treatment::RuleBased)),
        }
    } else {
        Forks {
            trained_online: None,
            trained_frozen: None,
            naive_online: None,
            rule_based: None,
        }
    };
    let branch_ticks: usize = [
        forks.trained_online.as_ref(),
        forks.trained_frozen.as_ref(),
        forks.naive_online.as_ref(),
        forks.rule_based.as_ref(),
    ]
    .into_iter()
    .flatten()
    .map(|branch| branch.ticks.len())
    .sum();
    let status = if !can_branch {
        "invalid_checkpoint_actor_dead"
    } else {
        "complete"
    };
    SeedRecord {
        schema: "curious-water-residual-seed-v1",
        protocol_version: 1,
        role,
        seed,
        status,
        physical_ticks: prefix.len() + branch_ticks,
        initial_state,
        prefix,
        checkpoint_state,
        forks,
    }
}

fn run_seed(seed: u64, role: &'static str) -> SeedRecord {
    run_world(make_world(seed), seed, role)
}

fn emit(record: &SeedRecord) {
    serde_json::to_writer(io::stdout().lock(), record).expect("write seed record");
    println!();
    io::stdout().flush().expect("flush seed record");
}

fn main() {
    let mut args = std::env::args().skip(1);
    let mode_arg = args.next();
    let seeds_arg = args.next();
    if mode_arg.as_deref() == Some("--unit-fixture") {
        if seeds_arg.is_some() || args.next().is_some() {
            eprintln!("usage: lab-probe --unit-fixture");
            process::exit(2);
        }
        emit(&run_seed(123, "unit"));
        return;
    }
    if args.next().is_some() || seeds_arg.is_none() {
        eprintln!("usage: lab-probe --calibration SEED | --formal SEED[,SEED...] | --unit-fixture");
        process::exit(2);
    }
    let seeds = seeds_arg
        .unwrap()
        .split(',')
        .map(|raw| raw.parse::<u64>())
        .collect::<Result<Vec<_>, _>>();
    let Ok(seeds) = seeds else {
        eprintln!("invalid seed list");
        process::exit(2);
    };
    match mode_arg.as_deref() {
        Some("--calibration") if seeds.len() == 1 => emit(&run_seed(seeds[0], "calibration")),
        Some("--formal") if !seeds.is_empty() && seeds.len() <= 8 => {
            for seed in seeds {
                emit(&run_seed(seed, "formal"));
            }
        }
        _ => {
            eprintln!("invalid mode or bounded seed count");
            process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use curious::{
        entity::{
            decision::{Decision, Reason},
            learning::{ResidualValue, Value as WaterValue},
            perception::Perception,
            species::Species,
        },
        map::movement::{Direction, Steps},
    };

    fn normalized_learner_branch(mut state: Value) -> Value {
        let entity = state["entities"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|entity| entity["id"] == json!(FOCAL_ID))
            .unwrap();
        entity["learner"]["enabled"] = json!(true);
        entity["learner"]["update_enabled"] = json!(true);
        entity["learner"]["values"] = json!([]);
        entity["learner"]["residuals"] = json!([]);
        state
    }

    fn normalized_reversal(mut state: Value) -> Value {
        for site in state["environment"]["sites"].as_array_mut().unwrap() {
            if site["pos"] == json!({"x":0,"y":2}) {
                site["replenish"] = json!(4);
                site["interval"] = json!(2);
            } else if site["pos"] == json!({"x":4,"y":2}) {
                site["replenish"] = json!(1);
                site["interval"] = json!(4);
            }
        }
        normalized_learner_branch(state)
    }

    fn treatment_state(seed: u64, treatment: Treatment) -> (Value, Value, Value) {
        let mut world = make_world(seed);
        while world.tick_counter < FORK_TICK {
            world.tick();
        }
        // Give the fork assertion a nonempty, deterministic history table so
        // accidental clearing in any preserved-table arm cannot pass vacuously.
        world.entities[0].learner.values = vec![
            WaterValue {
                pos: p(0, 2),
                value: 125,
                visits: 3,
            },
            WaterValue {
                pos: p(4, 2),
                value: 75,
                visits: 2,
            },
        ];
        world.entities[0].learner.residuals = vec![
            ResidualValue {
                pos: p(0, 2),
                sum: 210,
                visits: 3,
            },
            ResidualValue {
                pos: p(4, 2),
                sum: -80,
                visits: 2,
            },
        ];
        let checkpoint = State::capture(&world);
        let checkpoint_bytes = serde_json::to_vec(&checkpoint).unwrap();
        let serialized_checkpoint: State = serde_json::from_slice(&checkpoint_bytes).unwrap();
        assert_eq!(value(&checkpoint), value(&serialized_checkpoint));
        let mut branch = serialized_checkpoint.restore().unwrap();
        apply_treatment(&mut branch, treatment);
        let treated = value(&State::capture(&branch));
        reverse_water_rates(&mut branch);
        let reversed = value(&State::capture(&branch));
        (value(&checkpoint), treated, reversed)
    }

    #[test]
    fn fixture_pins_two_ai_lifetimes_and_visible_reversal_sources() {
        let world = make_world(123);
        assert!(world.policy.water_residual_enabled);
        assert_eq!(
            [
                world.map.min_x,
                world.map.max_x,
                world.map.min_y,
                world.map.max_y
            ],
            [0, 4, 0, 4]
        );
        assert_eq!(
            world
                .entities
                .iter()
                .map(|actor| actor.id)
                .collect::<Vec<_>>(),
            [FOCAL_ID, COMPETITOR_ID]
        );
        for actor in &world.entities {
            assert_eq!(actor.base.life().max_age, 512);
            assert_eq!(actor.base.life().maturity_age, 501);
            assert!(matches!(actor.controller, EntityController::AI));
        }
        assert!(world.entities[0].learner.enabled && world.entities[0].learner.update_enabled);
        assert!(!world.entities[1].learner.enabled && !world.entities[1].learner.update_enabled);
        assert_eq!(
            world
                .environment
                .sites
                .iter()
                .filter(|site| site.kind == ResourceKind::Water)
                .count(),
            2
        );
    }

    #[test]
    fn checkpoint_serialization_restore_replays_state_rng_events_and_decisions() {
        let mut uninterrupted = make_world(123);
        for _ in 0..17 {
            uninterrupted.tick();
        }
        let checkpoint = State::capture(&uninterrupted);
        let json = serde_json::to_vec(&checkpoint).unwrap();
        let restored: State = serde_json::from_slice(&json).unwrap();
        let mut resumed = restored.restore().unwrap();
        assert_eq!(
            value(&State::capture(&uninterrupted)),
            value(&State::capture(&resumed))
        );
        for _ in 0..40 {
            let left_events = uninterrupted.tick();
            let right_events = resumed.tick();
            assert_eq!(
                serde_json::to_value(&left_events).unwrap(),
                serde_json::to_value(&right_events).unwrap()
            );
            assert_eq!(
                serde_json::to_value(&uninterrupted.last_decisions).unwrap(),
                serde_json::to_value(&resumed.last_decisions).unwrap()
            );
            assert_eq!(
                value(&State::capture(&uninterrupted)),
                value(&State::capture(&resumed))
            );
        }
    }

    #[test]
    fn fork_treatments_change_only_registered_learner_fields_and_rates() {
        for treatment in [
            Treatment::Online,
            Treatment::TrainedFrozen,
            Treatment::NaiveOnline,
            Treatment::RuleBased,
        ] {
            let (checkpoint, treated, reversed) = treatment_state(123, treatment);
            let checkpoint_values = checkpoint["entities"][0]["learner"]["values"].clone();
            let treated_learner = &treated["entities"][0]["learner"];
            match treatment {
                Treatment::Online => {
                    assert_eq!(treated_learner["enabled"], true);
                    assert_eq!(treated_learner["update_enabled"], true);
                    assert_eq!(treated_learner["values"], checkpoint_values);
                }
                Treatment::TrainedFrozen => {
                    assert_eq!(treated_learner["enabled"], true);
                    assert_eq!(treated_learner["update_enabled"], false);
                    assert_eq!(treated_learner["values"], checkpoint_values);
                }
                Treatment::NaiveOnline => {
                    assert_eq!(treated_learner["enabled"], true);
                    assert_eq!(treated_learner["update_enabled"], true);
                    assert_eq!(treated_learner["values"], checkpoint_values);
                    assert!(
                        treated_learner
                            .get("residuals")
                            .and_then(Value::as_array)
                            .is_none_or(Vec::is_empty)
                    );
                }
                Treatment::RuleBased => {
                    assert_eq!(treated_learner["enabled"], false);
                    assert_eq!(treated_learner["update_enabled"], false);
                    assert_eq!(treated_learner["values"], checkpoint_values);
                }
            }
            assert_eq!(
                normalized_learner_branch(treated),
                normalized_learner_branch(checkpoint.clone())
            );
            assert_eq!(
                normalized_reversal(reversed),
                normalized_learner_branch(checkpoint)
            );
        }
        assert_eq!(
            (Treatment::Online.name(), Treatment::Online.description()),
            (
                "trained_online",
                "global_residual=true;enabled=true;update_enabled=true;retain-tables"
            )
        );
        let (_, trained, _) = treatment_state(123, Treatment::TrainedFrozen);
        let (_, naive, _) = treatment_state(123, Treatment::NaiveOnline);
        let (_, rule, _) = treatment_state(123, Treatment::RuleBased);
        assert_eq!(trained["entities"][0]["learner"]["enabled"], true);
        assert_eq!(trained["entities"][0]["learner"]["update_enabled"], false);
        assert_eq!(naive["entities"][0]["learner"]["enabled"], true);
        assert_eq!(naive["entities"][0]["learner"]["update_enabled"], true);
        assert!(
            naive["entities"][0]["learner"]
                .get("residuals")
                .and_then(Value::as_array)
                .is_none_or(Vec::is_empty)
        );
        assert_eq!(rule["entities"][0]["learner"]["enabled"], false);
        assert_eq!(rule["entities"][0]["learner"]["update_enabled"], false);
    }

    #[test]
    fn water_choice_classifier_does_not_count_visible_episode_continuations_as_fresh() {
        let mut world = make_world(123);
        let row = tick_row(&mut world);
        assert_eq!(row.tick, 1);
        // This fixture starts with full water and should not yet choose water.
        assert!(
            row.focal_choice
                .as_ref()
                .is_some_and(|choice| !choice.fresh)
        );
    }

    fn immediate_water_closure_world(with_stale_episode: bool) -> World {
        let mut world = make_world(123);
        world.entities[0].pos = p(0, 2);
        world.entities[0].base.life_mut().water = 10;
        if with_stale_episode {
            world.entities[0].learner.episode = Some(WaterEpisode {
                target: p(4, 4),
                ticks: 3,
                movement_energy: 2,
                incoming_damage: 0,
                choice_baseline: Some(5),
                outcome_overflowed: false,
                candidate_censored: false,
            });
        }
        world
    }

    #[test]
    fn fresh_immediate_drink_has_one_postresolver_closure() {
        let mut world = immediate_water_closure_world(false);
        let row = tick_row(&mut world);
        assert!(row.learner_closures.predecision_closures.is_empty());
        assert_eq!(row.learner_closures.postresolver_closures.len(), 1);
        assert_eq!(
            row.learner_closures.postresolver_closures[0]["end"],
            json!("Reached")
        );
        assert!(row.learner_closures.replayed_decision_matches_trace);
        assert!(row.learner_closures.replayed_postdecision_rng_matches_world);
        assert!(row.learner_closures.replayed_learner_matches_world);
    }

    #[test]
    fn stale_episode_interruption_and_fresh_same_tick_drink_record_two_closures() {
        let mut world = immediate_water_closure_world(true);
        let row = tick_row(&mut world);
        assert_eq!(row.learner_closures.predecision_closures.len(), 1);
        assert_eq!(
            row.learner_closures.predecision_closures[0]["end"],
            json!("Interrupted")
        );
        assert_eq!(row.learner_closures.postresolver_closures.len(), 1);
        assert_eq!(
            row.learner_closures.postresolver_closures[0]["end"],
            json!("Reached")
        );
        assert!(row.learner_closures.replayed_decision_matches_trace);
        assert!(row.learner_closures.replayed_postdecision_rng_matches_world);
        assert!(row.learner_closures.replayed_learner_matches_world);
    }

    #[test]
    fn stale_active_water_target_restarts_as_a_fresh_choice() {
        let world = make_world(123);
        let origin = p(2, 2);
        let mut view = Perception::empty();
        view.add_water(6, Steps(vec![Direction::Left, Direction::Left]));
        view.add_water(4, Steps(vec![Direction::Right, Direction::Right]));
        let trace = DecisionTrace::new(
            FORK_TICK + 1,
            FOCAL_ID,
            Species::Herbivore,
            origin,
            world.entities[0].base.life().clone(),
            123,
            view,
            Decision::new(
                Intent::Drink {
                    at: Steps(vec![Direction::Left, Direction::Left]),
                },
                Reason::LearnedWater,
            ),
            true,
        );
        let mut before = value(&State::capture(&world));
        before["entities"][0]["learner"]["episode"] = json!({
            "target": {"x": 4, "y": 4},
            "ticks": 3,
            "movement_energy": 2,
            "incoming_damage": 0
        });
        let exposure = choice_exposure(&trace, &before);
        assert_eq!(exposure.active_episode_before.unwrap().target, p(4, 4));
        assert!(exposure.fresh);
        assert_eq!(exposure.selection_kind, "greedy");
    }

    #[test]
    fn reversed_rates_do_not_change_world_rng_or_current_resource_amounts() {
        let mut world = make_world(123);
        let before_rng = world.rng.state();
        let before_cells = [
            world.map.cell(p(0, 2)).cloned(),
            world.map.cell(p(4, 2)).cloned(),
        ];
        reverse_water_rates(&mut world);
        assert_eq!(world.rng.state(), before_rng);
        assert_eq!(
            [
                world.map.cell(p(0, 2)).cloned(),
                world.map.cell(p(4, 2)).cloned()
            ],
            before_cells
        );
    }

    #[test]
    fn early_natural_death_ends_prefix_without_replacement_or_forks() {
        let mut world = make_world(123);
        let focal = world
            .entities
            .iter_mut()
            .find(|actor| actor.id == FOCAL_ID)
            .unwrap();
        focal.base.life_mut().health = 1;
        focal.base.life_mut().water = 0;
        let record = run_world(world, 123, "calibration");
        assert_eq!(record.status, "invalid_checkpoint_actor_dead");
        assert_eq!(record.prefix.len(), 1);
        assert_eq!(record.checkpoint_state["tick"], json!(1));
        assert_eq!(record.physical_ticks, 1);
        assert!(record.forks.trained_online.is_none());
        assert!(record.forks.trained_frozen.is_none());
        assert!(record.forks.naive_online.is_none());
        assert!(record.forks.rule_based.is_none());
        assert_eq!(
            record.checkpoint_state["entities"][0]["id"],
            json!(FOCAL_ID)
        );
    }
}
