//! Paired, seen-seed genotype observer engineering check. No fitness inference.
#[path = "support/ecology_audit.rs"]
#[allow(dead_code)]
mod ecology_audit;
#[path = "support/genotype_audit.rs"]
#[allow(dead_code)]
mod genotype_audit;

use curious::{
    creatures::{carnivore::CarnivoreEntity, herbivore::HerbivoreEntity},
    entity::{decision::DecisionTrace, genetics::Genome, phase::EntityPhase, species::Species},
    map::movement::Position,
    scenario::{Founder, Scenario},
    simulation::{
        SimulationEvent,
        checkpoint::{Individual, State},
    },
    world::World,
};
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    path::Path,
};

const SEED: u64 = 123;
const TICKS: usize = 200;
const MAX_RAW_BYTES: u64 = 112 * 1024 * 1024;
const MAX_IDENTITIES: usize = 50_000;
const MAX_EVENTS_PER_TICK: usize = 20_000;
const MAX_LINEAGE_EVENTS: usize = 50_000;
const MAX_DETAIL_ROWS: usize = 2_000;
const FRAME_TICKS: [usize; 3] = [1, 100, 200];
const PRIOR_SINGLE_ARM_RAW_BYTES: u64 = 24_728_247;
const PRIOR_IDENTITIES_AT_TICK_200: u64 = 1_844;

fn fixture_projection_bytes() -> u64 {
    let paired_prior_capture = PRIOR_SINGLE_ARM_RAW_BYTES * 2;
    let prior_typed_event_counts = 2 * (177_179 * 72 + 31_725 * 75 + 23_113 * 60);
    let expressed_trait_events = 57_600 * 90;
    let compact_trace_rows = PRIOR_IDENTITIES_AT_TICK_200 * TICKS as u64 * 2 * 24;
    paired_prior_capture
        + prior_typed_event_counts
        + expressed_trait_events
        + compact_trace_rows
        + 1_000_000
}

#[derive(Clone, Debug, Serialize)]
struct FounderRow {
    id: usize,
    species: Species,
    x: isize,
    y: isize,
    tile_row: usize,
    tile_column: usize,
    genome: Genome,
    base_vision_range: usize,
}

#[derive(Clone, Debug, Serialize)]
struct TileCount {
    row: usize,
    column: usize,
    vision_0: usize,
    vision_1: usize,
}

#[derive(Clone, Debug, Serialize)]
struct AssignmentReport {
    founders: Vec<FounderRow>,
    tile_counts: Vec<TileCount>,
    herbivores: usize,
    vision_0: usize,
    vision_1: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    Expressed,
    Neutral,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Self::Expressed => "expressed",
            Self::Neutral => "neutral",
        }
    }
    fn extra_traits(self) -> bool {
        self == Self::Expressed
    }
}

fn tile_coordinate(value: isize, min: isize, max: isize) -> Result<usize, String> {
    if min > max || value < min || value > max {
        return Err("founder position is outside scenario bounds".into());
    }
    let span = (max - min + 1) as i128;
    let offset = (value - min) as i128;
    Ok(((offset * 4) / span).min(3) as usize)
}

fn base_life(founder: &Founder) -> curious::entity::lifestate::LifeState {
    founder
        .life
        .clone()
        .unwrap_or_else(|| match &founder.species {
            Species::Herbivore => HerbivoreEntity::default().life_state,
            Species::Carnivore => CarnivoreEntity::default().life_state,
            Species::Omnivore => curious::creatures::omnivore::OmnivoreEntity::default().life_state,
        })
}

/// Deterministic, 4x4 spatially stratified, globally alternating assignment.
/// This helper is pure and never constructs a World.
fn assign_founders(scenario: &mut Scenario) -> Result<AssignmentReport, String> {
    let [min_x, max_x, min_y, max_y] = scenario.bounds;
    if scenario.bounds.iter().any(|v| v.unsigned_abs() > 1_000_000)
        || min_x > max_x
        || min_y > max_y
    {
        return Err("invalid scenario bounds".into());
    }
    let mut tile_members: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
    let mut tiles = vec![[0usize; 2]; 16];
    let mut rows = Vec::with_capacity(scenario.founders.len());
    for (index, founder) in scenario.founders.iter().enumerate() {
        let row = tile_coordinate(founder.pos.y, min_y, max_y)?;
        let column = tile_coordinate(founder.pos.x, min_x, max_x)?;
        rows.push(FounderRow {
            id: index + 1,
            species: founder.species.clone(),
            x: founder.pos.x,
            y: founder.pos.y,
            tile_row: row,
            tile_column: column,
            genome: Genome::default(),
            base_vision_range: base_life(founder).vision_range,
        });
        if founder.species == Species::Herbivore {
            tile_members.entry((row, column)).or_default().push(index);
        }
    }
    let herbivore_count: usize = tile_members.values().map(Vec::len).sum();
    if herbivore_count == 0 || herbivore_count % 2 != 0 {
        return Err("herbivore founder count must be positive and even".into());
    }
    let mut alternating = 0usize;
    for ((row, column), indices) in &mut tile_members {
        indices.sort_by_key(|&i| {
            let f = &scenario.founders[i];
            (f.pos.y, f.pos.x, i)
        });
        for &index in indices.iter() {
            let allele = alternating % 2;
            alternating += 1;
            tiles[row * 4 + column][allele] += 1;
            rows[index].genome = Genome {
                mobility: 0,
                vision: allele as u8,
                stamina: 0,
            };
        }
    }
    for (index, founder) in scenario.founders.iter_mut().enumerate() {
        if founder.species != Species::Herbivore {
            rows[index].genome = Genome::default();
        }
        founder.genome = rows[index].genome.clone();
    }
    let vision_0 = rows
        .iter()
        .filter(|r| r.species == Species::Herbivore && r.genome.vision == 0)
        .count();
    let vision_1 = rows
        .iter()
        .filter(|r| r.species == Species::Herbivore && r.genome.vision == 1)
        .count();
    if vision_0 != herbivore_count / 2 || vision_1 != herbivore_count / 2 {
        return Err("spatial assignment did not yield exact global 1:1 split".into());
    }
    let tile_counts = tiles
        .into_iter()
        .enumerate()
        .map(|(i, values)| TileCount {
            row: i / 4,
            column: i % 4,
            vision_0: values[0],
            vision_1: values[1],
        })
        .collect();
    Ok(AssignmentReport {
        founders: rows,
        tile_counts,
        herbivores: herbivore_count,
        vision_0,
        vision_1,
    })
}

fn explicit_profile(scenario: &mut Scenario, arm: Arm) {
    scenario.genetics.enabled = true;
    scenario.genetics.mutation_per_mille = 0;
    scenario.genetics.extra_traits_enabled = arm.extra_traits();
    scenario.policy.mate_rendezvous = true;
    scenario.policy.reserve_opportunities = true;
    scenario.policy.water_residual_enabled = false;
}

fn configure_agents(world: &mut World) {
    world.observation.full_all = true;
    world.observation.full_for.clear();
    for entity in &mut world.entities {
        entity.learner.enabled = true;
        entity.learner.update_enabled = true;
        entity.memory.enabled = true;
        entity.memory.companions_enabled = true;
        entity.hunt_learner.enabled = false;
        entity.memory.refuge_outcomes_enabled = false;
        entity.memory.refuge_generalization_enabled = false;
        entity.memory.refuge_generalization_update_enabled = false;
    }
}

fn verify_initial(world: &World, arm: Arm, assignment: &AssignmentReport) -> Result<(), String> {
    if world.entities.len() != assignment.founders.len() {
        return Err("world founder count differs from scenario assignment".into());
    }
    if !world.genetics.enabled
        || world.genetics.mutation_per_mille != 0
        || world.genetics.extra_traits_enabled != arm.extra_traits()
        || !world.policy.mate_rendezvous
        || !world.policy.reserve_opportunities
        || world.policy.water_residual_enabled
    {
        return Err("actual genetics or accepted policy profile mismatch".into());
    }
    for row in &assignment.founders {
        let Some(slot) = world.entities.iter().find(|slot| slot.id == row.id) else {
            return Err(format!("initial State missing founder {}", row.id));
        };
        let life = slot.base.life();
        let expected_vision = if arm.extra_traits() && row.species == Species::Herbivore {
            (row.base_vision_range as isize + row.genome.vision as isize).clamp(1, 32) as usize
        } else {
            row.base_vision_range
        };
        let expected_cost = if arm.extra_traits() && row.species == Species::Herbivore {
            row.genome.vision as usize
        } else {
            0
        };
        if slot.genome != row.genome
            || life.vision_range != expected_vision
            || life.vision_metabolic_cost != expected_cost
            || life.stamina_metabolic_cost != 0
        {
            return Err(format!(
                "founder {} genotype/phenotype expression mismatch",
                row.id
            ));
        }
    }
    Ok(())
}

fn audit_relevant(event: &SimulationEvent) -> bool {
    matches!(
        event,
        SimulationEvent::ResourceReplenished { .. }
            | SimulationEvent::EntityAte { .. }
            | SimulationEvent::EntityDrank { .. }
            | SimulationEvent::EntityMetabolized { .. }
            | SimulationEvent::EntityMated { .. }
            | SimulationEvent::EntityBorn { .. }
            | SimulationEvent::EntityDied { .. }
    )
}

fn relevant_events(
    events: &[SimulationEvent],
    after: &State,
) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let after_by: BTreeMap<_, _> = after.entities.iter().map(|i| (i.id, i)).collect();
    let mut rows = Vec::new();
    for event in events.iter().filter(|e| audit_relevant(e)) {
        let value = serde_json::to_value(event)?;
        let child = match event {
            SimulationEvent::EntityBorn { id, .. } => after_by.get(id).map(|i| json!({
                "id":i.id,"species":i.species,"genome":i.genome,"generation":i.generation,"parents":i.parents,
                "vision_range":i.life.vision_range,"vision_cost_requested":i.life.vision_metabolic_cost,
                "stamina":i.life.stamina,"max_stamina":i.life.max_stamina,"stamina_cost_requested":i.life.stamina_metabolic_cost
            })),
            _ => None,
        };
        rows.push(match child {
            Some(snapshot) => json!([value, snapshot]),
            None => value,
        });
    }
    Ok(rows)
}

fn compact_traces(
    traces: &[DecisionTrace],
    before: &State,
) -> (Vec<Value>, Vec<usize>, Vec<usize>) {
    let seen: BTreeSet<_> = traces.iter().map(|t| t.id).collect();
    let missing: Vec<_> = before
        .entities
        .iter()
        .filter(|i| !seen.contains(&i.id))
        .map(|i| i.id)
        .collect();
    let mut perception_missing = Vec::new();
    let rows = traces
        .iter()
        .map(|trace| {
            let (plant, corpse) = match &trace.perception {
                Some(view) => view.foods.iter().fold((0usize, 0usize), |(p, c), food| {
                    if food.is_corpse {
                        (p, c + 1)
                    } else {
                        (p + 1, c)
                    }
                }),
                None => {
                    perception_missing.push(trace.id);
                    (0, 0)
                }
            };
            let full = trace.perception.is_some();
            json!([
                trace.id,
                full,
                trace.before.vision_range,
                if full { Some(plant) } else { None },
                if full { Some(corpse) } else { None }
            ])
        })
        .collect();
    (rows, missing, perception_missing)
}

fn trait_state_mismatches(state: &State) -> Vec<Value> {
    state.entities.iter().filter_map(|i| {
        let expected_vision=if state.genetics.extra_traits_enabled {i.genome.vision as usize}else{0};
        let expected_stamina=if state.genetics.extra_traits_enabled {i.genome.stamina as usize}else{0};
        (i.life.vision_metabolic_cost!=expected_vision || i.life.stamina_metabolic_cost!=expected_stamina).then(||json!({
            "id":i.id,"species":i.species,"genome":i.genome,"life_vision_cost":i.life.vision_metabolic_cost,"expected_vision_cost":expected_vision,
            "life_stamina_cost":i.life.stamina_metabolic_cost,"expected_stamina_cost":expected_stamina
        }))
    }).collect()
}

fn trait_event_mismatches(events: &[SimulationEvent], before: &State, after: &State) -> Vec<Value> {
    let before_by: BTreeMap<_, _> = before.entities.iter().map(|i| (i.id, i)).collect();
    let after_by: BTreeMap<_, _> = after.entities.iter().map(|i| (i.id, i)).collect();
    events.iter().filter_map(|event| if let SimulationEvent::EntityMetabolized{id,vision_cost,stamina_cost,vision_paid,stamina_paid} = event {
        let actor=before_by.get(id).or_else(||after_by.get(id));
        let expected=actor.map(|i|(i.life.vision_metabolic_cost,i.life.stamina_metabolic_cost));
        let mismatch=expected.is_none_or(|(v,s)|v!=*vision_cost||s!=*stamina_cost)||vision_paid>vision_cost||stamina_paid>stamina_cost;
        mismatch.then(||json!({"id":id,"vision_requested":vision_cost,"stamina_requested":stamina_cost,"vision_paid":vision_paid,"stamina_paid":stamina_paid,"expected_life_cost":expected}))
    } else {None}).collect()
}

fn live_counts(world: &World) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for slot in &world.entities {
        if slot.base.life().is_alive()
            && matches!(
                slot.phase,
                EntityPhase::Active | EntityPhase::Sleeping { .. }
            )
        {
            *counts
                .entry(format!("{:?}", slot.base.species()))
                .or_default() += 1;
        }
    }
    counts
}

fn stock_reconciliation(audit: &ecology_audit::Audit) -> Option<Value> {
    audit.reconciliation.last().cloned()
}

fn projected_ecology_issues(
    audit: &ecology_audit::Audit,
    world: &World,
    events: &[SimulationEvent],
) -> usize {
    let mut parented = audit.parented_births.clone();
    let mut count = 0usize;
    for event in events {
        match event {
            SimulationEvent::EntityMated {
                parent1,
                parent2,
                child,
                ..
            } => {
                if !audit.ledger.contains_key(parent1) || !audit.ledger.contains_key(parent2) {
                    count += 1;
                }
                let expected = audit
                    .ledger
                    .get(parent1)
                    .and_then(|x| x.generation)
                    .zip(audit.ledger.get(parent2).and_then(|x| x.generation))
                    .map(|(a, b)| a.max(b) + 1);
                if let Some(c) = world.entities.iter().find(|x| x.id == *child) {
                    if expected.is_some_and(|g| g != c.generation)
                        || c.parents.len() != 2
                        || !c.parents.contains(parent1)
                        || !c.parents.contains(parent2)
                    {
                        count += 1;
                    }
                } else {
                    count += 1;
                }
                parented.insert(*child);
            }
            SimulationEvent::EntityBorn { id, species, .. } => {
                if audit.ledger.get(id).is_some_and(|x| x.species != *species)
                    || !parented.contains(id)
                {
                    count += 1;
                }
            }
            SimulationEvent::EntityAte { id, .. }
            | SimulationEvent::EntityDrank { id, .. }
            | SimulationEvent::EntityMoved { id, .. } => {
                if !audit.ledger.contains_key(id) {
                    count += 1;
                }
            }
            SimulationEvent::EntityMetabolized {
                id,
                vision_cost,
                stamina_cost,
                vision_paid,
                stamina_paid,
            } => {
                if vision_paid > vision_cost || stamina_paid > stamina_cost {
                    count += 1;
                }
                if !audit.ledger.contains_key(id) {
                    count += 1;
                }
            }
            _ => {}
        }
    }
    count
}

fn write_line(
    writer: &mut BufWriter<std::fs::File>,
    bytes: &mut u64,
    value: &Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let row = serde_json::to_vec(value)?;
    let next = bytes.saturating_add(row.len() as u64).saturating_add(1);
    if next > MAX_RAW_BYTES {
        return Err("combined two-arm raw JSONL cap exceeded".into());
    }
    writer.write_all(&row)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    *bytes = next;
    Ok(())
}

fn run_arm(
    scenario_bytes: &[u8],
    arm: Arm,
    raw: &mut BufWriter<std::fs::File>,
    raw_bytes: &mut u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut scenario: Scenario = serde_json::from_slice(scenario_bytes)?;
    let assignment = assign_founders(&mut scenario).map_err(std::io::Error::other)?;
    if assignment.founders.len() != 608
        || assignment.herbivores != 576
        || assignment.vision_0 != 288
        || assignment.vision_1 != 288
    {
        return Err("accepted fixture founder count or balanced assignment mismatch".into());
    }
    explicit_profile(&mut scenario, arm);
    let scenario_name = scenario.name.clone();
    let mut world = scenario.build(SEED)?;
    configure_agents(&mut world);
    verify_initial(&world, arm, &assignment).map_err(std::io::Error::other)?;
    let initial_state = State::capture(&world);
    let initial_state_value = serde_json::to_value(&initial_state)?;
    let start_stock = ecology_audit::stock(&world);
    let mut previous_stock = start_stock;
    let mut previous_cells = ecology_audit::resource_cells(&world);
    let mut genotype = genotype_audit::Audit::default();
    genotype
        .seed(&initial_state)
        .map_err(|e| std::io::Error::other(format!("genotype audit seed: {e:?}")))?;
    let mut ecology = ecology_audit::Audit::default();
    ecology.seed(&world);
    write_line(
        raw,
        raw_bytes,
        &json!({"arm_start":{"arm":arm.name(),"seed":SEED,"ticks":TICKS,"scenario_name":scenario_name,
        "founder_assignment":assignment,"accepted_profile":{"mate_rendezvous":true,"reserve_opportunities":true,"water_residual_enabled":false,
        "genetics_enabled":true,"mutation_per_mille":0,"extra_traits_enabled":arm.extra_traits(),"full_perception_capture":"memory only; compact counts serialized"},
        "tick0_state":initial_state_value,"start_food_stock":start_stock.0,"start_water_stock":start_stock.1}}),
    )?;

    let mut lineage_events = 0usize;
    for tick in 1..=TICKS {
        let before = State::capture(&world);
        let frame_pre = if FRAME_TICKS.contains(&tick) {
            Some(serde_json::to_value(&before)?)
        } else {
            None
        };
        let events = world.tick();
        let after = State::capture(&world);
        let traces = &world.last_decisions;
        if events.len() > MAX_EVENTS_PER_TICK || world.entities.len() > MAX_IDENTITIES {
            return Err(
                format!("event or entity cap exceeded at {} tick {tick}", arm.name()).into(),
            );
        }
        let lineage_here = events
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    SimulationEvent::EntityMated { .. }
                        | SimulationEvent::EntityBorn { .. }
                        | SimulationEvent::EntityDied { .. }
                )
            })
            .count();
        if lineage_events.saturating_add(lineage_here) > MAX_LINEAGE_EVENTS
            || genotype.individual_count().saturating_add(
                events
                    .iter()
                    .filter(|e| matches!(e, SimulationEvent::EntityBorn { .. }))
                    .count(),
            ) > MAX_IDENTITIES
        {
            return Err(format!(
                "lineage or retained identity cap exceeded at {} tick {tick}",
                arm.name()
            )
            .into());
        }
        let projected_rows = projected_ecology_issues(&ecology, &world, &events);
        let retained = ecology.reconciliation.len()
            + ecology.replacements.len()
            + ecology.parent_event_issues.len()
            + ecology.unattributed_actor_events.len()
            + ecology.trait_payment_issues.len();
        let cells_now = ecology_audit::resource_cells(&world);
        let replacements = previous_cells
            .keys()
            .chain(cells_now.keys())
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .filter(|pos| {
                let a = previous_cells.get(pos).copied().unwrap_or("empty");
                let b = cells_now.get(pos).copied().unwrap_or("empty");
                a != b && a != "empty" && b != "empty"
            })
            .count();
        if retained
            .saturating_add(projected_rows)
            .saturating_add(replacements)
            .saturating_add(1)
            > MAX_DETAIL_ROWS
        {
            return Err(format!(
                "detail row cap exceeded before audit mutation at {} tick {tick}",
                arm.name()
            )
            .into());
        }
        let (trace_rows, missing_trace_ids, missing_perception_ids) =
            compact_traces(traces, &before);
        let audited_events = relevant_events(&events, &after)?;
        let trait_state_issues = trait_state_mismatches(&before)
            .into_iter()
            .chain(trait_state_mismatches(&after))
            .collect::<Vec<_>>();
        let trait_event_issues = trait_event_mismatches(&events, &before, &after);
        let full_events = if FRAME_TICKS.contains(&tick) {
            Some(serde_json::to_value(&events)?)
        } else {
            None
        };
        let event_counts =
            events
                .iter()
                .fold(BTreeMap::<String, usize>::new(), |mut map, event| {
                    let val = serde_json::to_value(event).unwrap_or(Value::Null);
                    if let Some(kind) = val.as_object().and_then(|o| o.keys().next()) {
                        *map.entry(kind.clone()).or_default() += 1;
                    }
                    map
                });
        let _ = ecology.observe_tick(tick, previous_stock, &previous_cells, &world, &events);
        let (food_stock, water_stock) = ecology_audit::stock(&world);
        let audit_error = genotype.observe(&before, &after, &events, traces);
        lineage_events += lineage_here;
        let frame=frame_pre.map(|pre|json!({"pre_state":pre,"all_typed_events":full_events,"post_state":serde_json::to_value(&after).unwrap_or(Value::Null)}));
        let row = json!({"arm_tick":{"arm":arm.name(),"tick":tick,"alive_by_species":live_counts(&world),
            "food_stock":food_stock,"water_stock":water_stock,"event_counts_by_type":event_counts,
            "audit_events":audited_events,"trace_food_encounters":trace_rows,
            "missing_trace_ids":missing_trace_ids,"missing_full_perception_ids":missing_perception_ids,
            "trait_state_issues":trait_state_issues,"trait_event_issues":trait_event_issues,
            "resource_reconciliation":stock_reconciliation(&ecology),"resource_replacements_cumulative":ecology.replacement_count,
            "genotype_audit_error":audit_error.as_ref().err().map(|e|format!("{e:?}")),"genotype_issues":genotype.issues(),
            "checkpoint_frame":frame}});
        write_line(raw, raw_bytes, &row)?;
        previous_stock = (food_stock, water_stock);
        previous_cells = cells_now;
        if tick == 100 {
            let encoded = serde_json::to_vec(&State::capture(&world))?;
            let decoded: State = serde_json::from_slice(&encoded)?;
            let restored = decoded.restore()?;
            let restored_value = serde_json::to_value(State::capture(&restored))?;
            let roundtrip_ok = restored_value == serde_json::to_value(State::capture(&world))?;
            write_line(
                raw,
                raw_bytes,
                &json!({"checkpoint_restore":{"arm":arm.name(),"tick":100,"state_and_rng_roundtrip_equal":roundtrip_ok,"serialized_bytes":encoded.len()}}),
            )?;
            if !roundtrip_ok {
                return Err(
                    format!("checkpoint State/RNG roundtrip mismatch in {}", arm.name()).into(),
                );
            }
            world = restored;
        }
        if audit_error.is_err()
            || !trait_state_issues.is_empty()
            || !trait_event_issues.is_empty()
            || !genotype.issues().is_empty()
            || !ecology.parent_event_issues.is_empty()
            || !ecology.unattributed_actor_events.is_empty()
            || !ecology.trait_payment_issues.is_empty()
            || ecology.replacement_count > 0
            || ecology.food_residual_nonzero_ticks > 0
            || ecology.water_residual_nonzero_ticks > 0
        {
            return Err(format!(
                "observer or resource-flow issue at {} tick {tick}",
                arm.name()
            )
            .into());
        }
    }
    let summary = json!({"arm_complete":{"arm":arm.name(),"ticks_completed":TICKS,"initial_state":initial_state_value,
        "genotype_summary":genotype.summary(),"ecology_audit":ecology.result(&world),"end_state":serde_json::to_value(State::capture(&world))?,
        "food_stock_at_start":start_stock.0,"water_stock_at_start":start_stock.1}});
    write_line(raw, raw_bytes, &summary)?;
    Ok(())
}

fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    const PROTOCOL: &str = include_str!("../docs/GENOTYPE_MEASUREMENT_PROTOCOL.md");
    if !PROTOCOL
        .lines()
        .any(|line| line.contains("Status: FROZEN BY ROOT"))
    {
        return Err("protocol is not marked FROZEN BY ROOT; refusing World execution".into());
    }
    let args: Vec<_> = env::args().collect();
    if args.len() != 3 {
        return Err(
            "usage: genotype_measurement_lab ACCEPTED_SCENARIO.json NEW_OUTPUT.jsonl".into(),
        );
    }
    let scenario_bytes = fs::read(&args[1])?;
    let output_path = Path::new(&args[2]);
    let output_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output_path)?;
    let mut raw = BufWriter::new(output_file);
    let mut bytes = 0u64;
    write_line(
        &mut raw,
        &mut bytes,
        &json!({"measurement_lab_header":{"schema":"curious-genotype-measurement-v1","seed":SEED,"ticks_per_arm":TICKS,"arms":["expressed","neutral"],"claim_scope":"seen-seed measurement engineering only; no fitness or selection claim","raw_cap_bytes":MAX_RAW_BYTES,"hard_limits":{"identities":MAX_IDENTITIES,"events_per_tick":MAX_EVENTS_PER_TICK,"lineage_events_per_arm":MAX_LINEAGE_EVENTS,"detail_rows_per_arm":MAX_DETAIL_ROWS}}}),
    )?;
    run_arm(&scenario_bytes, Arm::Expressed, &mut raw, &mut bytes)?;
    run_arm(&scenario_bytes, Arm::Neutral, &mut raw, &mut bytes)?;
    let raw_bytes = bytes;
    write_line(
        &mut raw,
        &mut bytes,
        &json!({"measurement_lab_complete":{"status":"observer_measurement_complete","raw_bytes_before_footer":raw_bytes,"seed":SEED,"ticks_per_arm":TICKS}}),
    )?;
    raw.get_ref().sync_all()?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("genotype_measurement_lab: {error}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use curious::{
        entity::{
            controller::EntityController,
            decision::{Decision, DecisionTrace, Reason},
            intent::Intent,
            lifestate::LifeState,
            perception::Perception,
            phase::EntityPhase,
        },
        map::movement::Steps,
    };

    fn life() -> LifeState {
        LifeState {
            max_meal: 10,
            move_energy_cost: 1,
            max_stamina: 16,
            stamina: 16,
            exertion: 0,
            vision_metabolic_cost: 0,
            stamina_metabolic_cost: 0,
            death_cause: None,
            max_age: 100,
            max_health: 100,
            max_energy: 80,
            max_water: 60,
            maturity_age: 20,
            vision_range: 6,
            age: 0,
            health: 100,
            energy: 80,
            water: 60,
            reproduction_cooldown: 0,
            reproduction_interval: 25,
            reproduction_reserve_per_mille: 500,
            speed: 3,
            moves_used: 0,
        }
    }
    fn founder(species: Species, pos: Position) -> Founder {
        Founder {
            genome: Genome::default(),
            life: Some(life()),
            species,
            pos,
        }
    }
    fn fixture() -> Scenario {
        Scenario {
            name: "pure assignment".into(),
            bounds: [0, 7, 0, 7],
            founders: vec![
                founder(Species::Herbivore, Position::new(0, 0)),
                founder(Species::Herbivore, Position::new(1, 0)),
                founder(Species::Herbivore, Position::new(0, 1)),
                founder(Species::Herbivore, Position::new(1, 1)),
                founder(Species::Carnivore, Position::new(7, 7)),
            ],
            sites: vec![],
            resources: vec![],
            genetics: Default::default(),
            policy: Default::default(),
        }
    }

    #[test]
    fn assignment_is_repeatable_balanced_and_tile_local() {
        let mut a = fixture();
        let mut b = fixture();
        let ar = assign_founders(&mut a).unwrap();
        let br = assign_founders(&mut b).unwrap();
        assert_eq!(
            ar.founders
                .iter()
                .map(|f| f.genome.clone())
                .collect::<Vec<_>>(),
            br.founders
                .iter()
                .map(|f| f.genome.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!((ar.herbivores, ar.vision_0, ar.vision_1), (4, 2, 2));
        assert!(
            ar.tile_counts
                .iter()
                .filter(|t| t.vision_0 + t.vision_1 > 0)
                .all(|t| t.vision_0.abs_diff(t.vision_1) <= 1)
        );
        assert!(
            ar.founders
                .iter()
                .all(|f| f.genome.mobility == 0 && f.genome.stamina == 0)
        );
    }

    #[test]
    fn assignment_rejects_odd_herbivore_population_and_wrong_bounds() {
        let mut odd = fixture();
        odd.founders.remove(0);
        assert!(assign_founders(&mut odd).unwrap_err().contains("even"));
        let mut out = fixture();
        out.founders[0].pos = Position::new(99, 99);
        assert!(assign_founders(&mut out).unwrap_err().contains("outside"));
    }

    #[test]
    fn trace_summary_keeps_repeats_and_unknown_perception_distinct() {
        let mut state = State {
            policy: Default::default(),
            genetics: Default::default(),
            observation: Default::default(),
            environment: Default::default(),
            version: 18,
            tick: 0,
            rng: 1,
            next_entity_id: 2,
            bounds: [0, 2, 0, 2],
            entities: vec![Individual {
                genome: Genome {
                    mobility: 0,
                    vision: 1,
                    stamina: 0,
                },
                learner: Default::default(),
                hunt_learner: Default::default(),
                memory: Default::default(),
                generation: 0,
                parents: vec![],
                corpse_energy: 0,
                id: 1,
                pos: Position::new(0, 0),
                phase: EntityPhase::Active,
                controller: EntityController::AI,
                species: Species::Herbivore,
                life: life(),
            }],
            resources: vec![],
            commands: vec![],
        };
        let mut view = Perception::empty();
        view.add_food(1, false, Steps::empty());
        view.add_food(1, false, Steps::empty());
        view.add_food(1, true, Steps::empty());
        let full = DecisionTrace::new(
            1,
            1,
            Species::Herbivore,
            Position::new(0, 0),
            life(),
            1,
            view,
            Decision::new(Intent::Idle { duration: 1 }, Reason::Rest),
            true,
        );
        let (rows, missing, unknown) = compact_traces(&[full], &state);
        assert!(missing.is_empty());
        assert!(unknown.is_empty());
        assert_eq!(rows[0][3], 2);
        assert_eq!(rows[0][4], 1);
        let partial = DecisionTrace::new(
            1,
            1,
            Species::Herbivore,
            Position::new(0, 0),
            life(),
            1,
            Perception::empty(),
            Decision::new(Intent::Idle { duration: 1 }, Reason::Rest),
            false,
        );
        let (rows, _, unknown) = compact_traces(&[partial], &state);
        assert_eq!(unknown, vec![1]);
        assert!(rows[0][3].is_null());
        state.entities.clear();
    }

    #[test]
    fn fixed_frame_schedule_is_only_the_registered_capture_points() {
        assert!(!FRAME_TICKS.contains(&0));
        assert!(FRAME_TICKS.contains(&1));
        assert!(FRAME_TICKS.contains(&100));
        assert!(FRAME_TICKS.contains(&200));
    }

    #[test]
    fn raw_projection_from_prior_fixture_counts_fits_below_cap() {
        assert_eq!(fixture_projection_bytes(), 106_388_980);
        assert!(fixture_projection_bytes() < MAX_RAW_BYTES);
        assert!(
            serde_json::to_vec(&json!([18_440, true, 6, 10, 3]))
                .unwrap()
                .len()
                <= 24
        );
    }
}
