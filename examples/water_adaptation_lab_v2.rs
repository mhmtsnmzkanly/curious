//! Continuous-lifetime context-window water adaptation lab, V2 food-rich fixture.
//!
//! Invoke with one explicit seed and required role: `cargo run --example
//! water_adaptation_lab_v2 -- 62199 calibration`. This example performs no seed
//! scan and writes one complete JSON record to stdout.
use curious::{
    creatures::herbivore::HerbivoreEntity,
    entity::{
        controller::EntityController,
        decision::DecisionTrace,
        intent::Intent,
        learning::{Learner, WaterEpisode},
        perception::PerceivedWater,
        phase::EntityPhase,
    },
    environment::{Environment, ResourceKind, ResourceSite},
    logger::Logger,
    map::{
        Map,
        cell::Cell,
        movement::Position,
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
const FORK_TICK: usize = 200;
const END_TICK: usize = 400;

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
    base_reason_inferred: bool,
    final_reason: String,
    intent: Value,
    fresh: bool,
    two_source_opportunity: bool,
    fresh_two_source_choice: bool,
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
    focal_physical: Value,
    state_after: Value,
    rng_after: u64,
}

#[derive(Clone, Debug, Serialize)]
struct BranchRecord {
    status: &'static str,
    learner_treatment: &'static str,
    state_before_treatment: Value,
    state_after_fork_interruption: Value,
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
    water_manipulation: &'static str,
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
    fn description(self) -> &'static str {
        match self {
            Self::Online => "trained_online:enabled=true;update_enabled=true;retain-values",
            Self::TrainedFrozen => "enabled=true;update_enabled=false;retain-values",
            Self::NaiveOnline => "enabled=true;update_enabled=true;clear-values-residuals-context-episodes",
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
    // The life cap exceeds the 1,000-tick experiment and maturity suppresses
    // reproduction throughout; neither changes during an arm.
    body.life_state.max_age = 2000;
    body.life_state.maturity_age = 2000;
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
    let entities = vec![actor(FOCAL_ID, p(0, 0), true)];
    let mut world = World::with_seed(-6, 6, -3, 3, entities, seed);
    world.logger = Logger::disabled();
    world.map = Map::new(-6, 6, -3, 3);
    world.map.set_cell(p(-2, 0), Cell::Water { amount: 12 });
    world.map.set_cell(p(4, 0), Cell::Water { amount: 12 });
    // Artificially food-rich physical environment: every non-water cell has
    // renewable food, so the actor can eat in place or from an adjacent cell
    // without first paying movement energy to reach a unique food patch.
    // Food remains ordinary, finite, renewable World resource; no body or
    // resource is reset between ticks. Food cells are passable.
    let food_sites: Vec<_> = (-3..=3)
        .flat_map(|y| (-6..=6).map(move |x| p(x, y)))
        .filter(|pos| *pos != p(-2, 0) && *pos != p(4, 0))
        .map(|pos| {
            world.map.set_cell(pos, Cell::Food { amount: 30 });
            site(pos, ResourceKind::Food, 30, 2, 2)
        })
        .collect();
    world.environment = Environment {
        sites: [
            vec![
                site(p(-2, 0), ResourceKind::Water, 12, 4, 2),
                site(p(4, 0), ResourceKind::Water, 12, 1, 12),
            ],
            food_sites,
        ]
        .concat(),
        ..Default::default()
    };
    world.observation.full_for = vec![FOCAL_ID];
    world.entities[0].learner.context_ttl_enabled = true;
    world.policy.water_residual_enabled = false;
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
    } else {
        "learner_choice_rng_not_classified"
    };
    ChoiceExposure {
        base_reason,
        base_reason_inferred: final_reason == "LearnedWater",
        final_reason,
        intent: serde_json::to_value(&trace.decision.intent).expect("serialize focal intent"),
        fresh,
        two_source_opportunity: base_is_water && candidates.len() >= 2,
        fresh_two_source_choice: fresh && base_is_water && candidates.len() >= 2,
        active_episode_before,
        candidates,
        selected_target: selected_target(trace),
        selection_kind,
    }
}

fn tick_row(world: &mut World) -> TickRow {
    let before_life = world.entities.iter().find(|s| s.id == FOCAL_ID).unwrap().base.life().clone();
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
    let after_life = world.entities.iter().find(|s| s.id == FOCAL_ID).unwrap().base.life();
    let event_values = serde_json::to_value(&events).expect("serialize events");
    let drank: usize = events.iter().map(|event| match event {
        curious::simulation::SimulationEvent::EntityDrank { id, amount, .. } if *id == FOCAL_ID => *amount,
        _ => 0,
    }).sum();
    let movement_energy_paid: usize = events.iter().map(|event| match event {
        curious::simulation::SimulationEvent::EntityMoved { id, energy_cost, .. } if *id == FOCAL_ID => *energy_cost,
        _ => 0,
    }).sum();
    let alive = after_life.is_alive();
    let thirsty = before_life.water < before_life.max_water / 4;
    let physical = json!({
        "position_before": world.last_decisions.iter().find(|trace| trace.id == FOCAL_ID).map(|t| t.pos),
        "water_before": before_life.water,
        "water_after": after_life.water,
        "water_gained_resolver": drank,
        "energy_before": before_life.energy,
        "energy_after": after_life.energy,
        "movement_energy_paid_resolver": movement_energy_paid,
        "health_before": before_life.health,
        "health_after": after_life.health,
        "thirst_tick": thirsty,
        "alive_after": alive,
        "death_cause": after_life.death_cause,
        "health_loss_attribution": "unknown_without_resolver_attribution"
    });
    let rng_after = world.rng.state();
    TickRow {
        tick: world.tick_counter,
        state_before: before,
        rng_before,
        events: event_values,
        decisions,
        focal_choice,
        focal_physical: physical,
        state_after: after,
        rng_after,
    }
}

fn apply_source_degradation_reversal(world: &mut World) {
    for source in &mut world.environment.sites {
        if source.pos == p(-2, 0) && source.kind == ResourceKind::Water {
            source.capacity = 12;
            source.replenish = 1;
            source.interval = 12;
        } else if source.pos == p(4, 0) && source.kind == ResourceKind::Water {
            source.capacity = 12;
            source.replenish = 4;
            source.interval = 2;
        }
    }
    world.map.set_cell(p(-2, 0), Cell::Water { amount: 1 });
    world.map.set_cell(p(4, 0), Cell::Water { amount: 12 });
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
            focal.learner.values.clear();
            focal.learner.residuals.clear();
            focal.learner.context_entries.clear();
            focal.learner.episode = None;
            focal.learner.last_episode = None;
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
    // Censor any episode already in progress at the artificial branch boundary,
    // identically in every arm, before applying each arm's learner treatment.
    world.entities.iter_mut().find(|actor| actor.id == FOCAL_ID).unwrap()
        .learner.interrupt(curious::entity::learning::WaterEnd::Interrupted);
    let state_after_fork_interruption = value(&State::capture(&world));
    apply_treatment(&mut world, treatment);
    let state_after_treatment_before_reversal = value(&State::capture(&world));
    apply_source_degradation_reversal(&mut world);
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
        state_after_fork_interruption,
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
    {
        prefix.push(tick_row(&mut world));
    }
    let checkpoint = State::capture(&world);
    let checkpoint_state = value(&checkpoint);
    let can_branch = actor_alive(&world, FOCAL_ID);
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
        schema: "curious-water-adaptation-seed-v2",
        protocol_version: 2,
        role,
        seed,
        status,
        water_manipulation: "V2 food-rich fixture; at fork set A(-2,0) to Water{amount:1}, capacity=12, replenish=1 every 12 ticks; set B(4,0) to Water{amount:12}, capacity=12, replenish=4 every 2 ticks; fixed identically in all arms",
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
    let seed = args.next().and_then(|s| s.parse::<u64>().ok());
    let role_arg = args.next();
    if args.next().is_some() || seed.is_none() || role_arg.is_none() {
        eprintln!("usage: lab-probe SEED calibration|formal");
        process::exit(2);
    }
    // No implicit seed scan or unrecognized role fallback.
    let role: &'static str = match role_arg.as_deref().unwrap() {
        "calibration" => "calibration",
        "formal" => "formal",
        _ => {
            eprintln!("role must be calibration or formal");
            process::exit(2);
        }
    };
    emit(&run_seed(seed.unwrap(), role));
}

#[cfg(test)]
mod fixture_tests {
    use super::*;
    use curious::simulation::SimulationEvent;

    #[test]
    fn renewable_food_covers_every_non_water_cell() {
        let world = make_world(1);
        let water_positions = [p(-2, 0), p(4, 0)];
        let food_sites: Vec<_> = world
            .environment
            .sites
            .iter()
            .filter(|resource| resource.kind == ResourceKind::Food)
            .collect();
        assert_eq!(food_sites.len(), 91 - water_positions.len());
        for y in -3..=3 {
            for x in -6..=6 {
                let pos = p(x, y);
                if water_positions.contains(&pos) {
                    assert!(matches!(world.map.cell(pos), Some(Cell::Water { .. })));
                } else {
                    assert!(matches!(world.map.cell(pos), Some(Cell::Food { amount: 30 })));
                    assert!(food_sites.iter().any(|resource| {
                        resource.pos == pos
                            && resource.capacity == 30
                            && resource.replenish == 2
                            && resource.interval == 2
                    }));
                }
            }
        }
    }

    #[test]
    fn zero_energy_actor_can_eat_food_underfoot_or_adjacent() {
        for start in [p(0, 0), p(-2, 0)] {
            let mut world = make_world(1);
            let focal = world.entities.iter_mut().find(|actor| actor.id == FOCAL_ID).unwrap();
            focal.pos = start;
            let life = focal.base.life_mut();
            life.energy = 0;
            life.water = life.max_water;
            life.health = life.max_health;

            let events = world.tick();

            assert!(events.iter().any(|event| matches!(
                event,
                SimulationEvent::EntityAte { id: FOCAL_ID, amount, .. } if *amount > 0
            )), "expected resolver food gain from start {start:?}; events={events:?}");
            assert!(!events.iter().any(|event| matches!(
                event,
                SimulationEvent::EntityMoved { id: FOCAL_ID, .. }
            )), "zero-energy actor moved from start {start:?}; events={events:?}");
            let life = world.entities.iter().find(|actor| actor.id == FOCAL_ID).unwrap().base.life();
            assert!(life.energy > 0, "food did not restore positive energy from start {start:?}");
            assert!(life.is_alive(), "actor died after eating from start {start:?}");
        }
    }
}

