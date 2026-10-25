//! Continuous-lifetime water-learning lab, protocol v2.
//!
//! Run `lab-probe --calibration` for the single calibration record or
//! `lab-probe --formal` for the eight preregistered records. The fixture has
//! two ordinary AI herbivores and uses only the normal world/resolver path.
use curious::{
    creatures::herbivore::HerbivoreEntity,
    entity::{
        controller::EntityController,
        decision::{Decision, DecisionTrace, Reason},
        intent::Intent,
        learning::{Learner, Value as WaterValue, WaterEpisode},
        perception::{PerceivedWater, Perception},
        phase::EntityPhase,
        species::Species,
    },
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
const CALIBRATION_SEED: u64 = 51099;
const FORMAL_SEEDS: [u64; 8] = [51001, 51002, 51003, 51004, 51005, 51006, 51007, 51008];

#[derive(Clone, Debug, Serialize)]
struct ChoiceCandidate {
    pos: Position,
    amount: usize,
    steps_len: usize,
    saved_value: Option<i32>,
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
    state_after: Value,
    rng_after: u64,
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
    online: Option<BranchRecord>,
    trained_frozen: Option<BranchRecord>,
    ablated_frozen: Option<BranchRecord>,
    rule_based: Option<BranchRecord>,
}

#[derive(Clone, Copy, Debug)]
enum Treatment {
    Online,
    TrainedFrozen,
    AblatedFrozen,
    RuleBased,
}

impl Treatment {
    fn name(self) -> &'static str {
        match self {
            Self::Online => "online",
            Self::TrainedFrozen => "trained_frozen",
            Self::AblatedFrozen => "ablated_frozen",
            Self::RuleBased => "rule_based",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Online => "enabled=true;update_enabled=true;retain-values",
            Self::TrainedFrozen => "enabled=true;update_enabled=false;retain-values",
            Self::AblatedFrozen => "enabled=true;update_enabled=false;clear-values-only",
            Self::RuleBased => "enabled=false;update_enabled=false;retain-unused-values",
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
            ChoiceCandidate {
                pos,
                amount: water.amount,
                steps_len: water.steps.len(),
                saved_value: learner
                    .values
                    .iter()
                    .find(|entry| entry.pos == pos)
                    .map(|entry| entry.value),
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
    let after = value(&State::capture(world));
    let rng_after = world.rng.state();
    TickRow {
        tick: world.tick_counter,
        state_before: before,
        rng_before,
        events: serde_json::to_value(events).expect("serialize events"),
        decisions,
        focal_choice,
        state_after: after,
        rng_after,
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
        Treatment::AblatedFrozen => {
            focal.learner.enabled = true;
            focal.learner.update_enabled = false;
            focal.learner.values.clear();
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
            online: Some(run_branch(&checkpoint, Treatment::Online)),
            trained_frozen: Some(run_branch(&checkpoint, Treatment::TrainedFrozen)),
            ablated_frozen: Some(run_branch(&checkpoint, Treatment::AblatedFrozen)),
            rule_based: Some(run_branch(&checkpoint, Treatment::RuleBased)),
        }
    } else {
        Forks {
            online: None,
            trained_frozen: None,
            ablated_frozen: None,
            rule_based: None,
        }
    };
    let branch_ticks: usize = [
        forks.online.as_ref(),
        forks.trained_frozen.as_ref(),
        forks.ablated_frozen.as_ref(),
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
        schema: "curious-lifetime-seed-v2",
        protocol_version: 2,
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
    let mode = mode_arg.as_deref().unwrap_or("--formal");
    if args.next().is_some() {
        eprintln!("usage: lab-probe [--calibration|--formal]");
        process::exit(2);
    }
    match mode {
        "--calibration" => emit(&run_seed(CALIBRATION_SEED, "calibration")),
        "--formal" => {
            for seed in FORMAL_SEEDS {
                emit(&run_seed(seed, "formal"));
            }
        }
        _ => {
            eprintln!("usage: lab-probe [--calibration|--formal]");
            process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            Treatment::AblatedFrozen,
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
                Treatment::AblatedFrozen => {
                    assert_eq!(treated_learner["enabled"], true);
                    assert_eq!(treated_learner["update_enabled"], false);
                    assert_eq!(treated_learner["values"], json!([]));
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
            ("online", "enabled=true;update_enabled=true;retain-values")
        );
        let (_, trained, _) = treatment_state(123, Treatment::TrainedFrozen);
        let (_, ablated, _) = treatment_state(123, Treatment::AblatedFrozen);
        let (_, rule, _) = treatment_state(123, Treatment::RuleBased);
        assert_eq!(trained["entities"][0]["learner"]["enabled"], true);
        assert_eq!(trained["entities"][0]["learner"]["update_enabled"], false);
        assert_eq!(ablated["entities"][0]["learner"]["enabled"], true);
        assert_eq!(ablated["entities"][0]["learner"]["update_enabled"], false);
        assert_eq!(ablated["entities"][0]["learner"]["values"], json!([]));
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
        assert!(record.forks.online.is_none());
        assert!(record.forks.trained_frozen.is_none());
        assert!(record.forks.ablated_frozen.is_none());
        assert!(record.forks.rule_based.is_none());
        assert_eq!(
            record.checkpoint_state["entities"][0]["id"],
            json!(FOCAL_ID)
        );
    }
}
