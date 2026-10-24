//! Bounded, explicitly policy-bound fixture viability measurement.
//!
//! This is not a trait comparison or ecology acceptance runner. The scenario
//! is read once and never modified on disk; accepted policy flags are supplied
//! explicitly because archived scenario JSON predates the policy field.
//!
//! Usage: ecology_validity_lab SCENARIO.json SEED TICKS MATE_RENDEZVOUS
//!        RESERVE_OPPORTUNITIES NEW_OUTPUT.json
#[path = "support/ecology_audit.rs"]
mod ecology_audit;

use curious::{scenario::Scenario, world::World};
use serde_json::json;
use std::{
    collections::BTreeMap,
    env, fs,
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::Path,
};

const MAX_TICKS: usize = 500;
const MAX_IDENTITIES: usize = 50_000;
const MAX_EVENTS_PER_TICK: usize = 20_000;
const MAX_RETAINED_DETAIL_ROWS: usize = 2_000;
const MAX_LINEAGE_EVENTS: usize = 50_000;
const MAX_RAW_BYTES: u64 = 128 * 1024 * 1024;
const AUDIT_FRAME_TICKS: [usize; 5] = [1, 20, 100, 200, 500];

fn write_raw_line(
    raw: &mut BufWriter<std::fs::File>,
    value: &serde_json::Value,
    max_bytes: u64,
) -> std::io::Result<()> {
    let bytes = serde_json::to_vec(value).map_err(std::io::Error::other)?;
    let next_size = raw
        .get_ref()
        .metadata()?
        .len()
        .saturating_add(bytes.len() as u64)
        .saturating_add(1);
    if next_size > max_bytes {
        return Err(std::io::Error::other(
            "aggregate raw output cap exceeded before row capture",
        ));
    }
    raw.write_all(&bytes)?;
    raw.write_all(b"\n")?;
    raw.flush()
}

fn alive(world: &World) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for entity in &world.entities {
        if entity.base.life().is_alive()
            && matches!(
                &entity.phase,
                curious::entity::phase::EntityPhase::Active
                    | curious::entity::phase::EntityPhase::Sleeping { .. }
            )
        {
            *counts
                .entry(format!("{:?}", entity.base.species()))
                .or_default() += 1;
        }
    }
    counts
}

fn eligible_adjacent_pairs(world: &World) -> BTreeMap<String, usize> {
    let mut pairs = BTreeMap::new();
    for left in 0..world.entities.len() {
        let a = &world.entities[left];
        if !a.phase.is_active() || a.base.life().reproduction_blocker().is_some() {
            continue;
        }
        for b in &world.entities[left + 1..] {
            if b.phase.is_active()
                && a.base.species() == b.base.species()
                && b.base.life().reproduction_blocker().is_none()
                && (a.pos.x - b.pos.x).abs() <= 1
                && (a.pos.y - b.pos.y).abs() <= 1
            {
                *pairs.entry(format!("{:?}", a.base.species())).or_default() += 1;
            }
        }
    }
    pairs
}

fn projected_issue_rows(
    audit: &ecology_audit::Audit,
    world: &World,
    events: &[curious::simulation::SimulationEvent],
) -> usize {
    let mut parented = audit.parented_births.clone();
    let mut issues = 0usize;
    for event in events {
        match event {
            curious::simulation::SimulationEvent::EntityMated {
                parent1,
                parent2,
                child,
                ..
            } => {
                let parents_known =
                    audit.ledger.contains_key(parent1) && audit.ledger.contains_key(parent2);
                issues += usize::from(!parents_known);
                let expected_generation = audit
                    .ledger
                    .get(parent1)
                    .and_then(|item| item.generation)
                    .zip(audit.ledger.get(parent2).and_then(|item| item.generation))
                    .map(|(first, second)| first.max(second) + 1);
                if let Some(slot) = world.entities.iter().find(|slot| slot.id == *child) {
                    let mismatch = expected_generation
                        .is_some_and(|generation| slot.generation != generation)
                        || slot.parents.len() != 2
                        || !slot.parents.contains(parent1)
                        || !slot.parents.contains(parent2);
                    issues += usize::from(mismatch);
                } else {
                    issues += 1;
                }
                parented.insert(*child);
            }
            curious::simulation::SimulationEvent::EntityBorn { id, species, .. } => {
                if let Some(item) = audit.ledger.get(id) {
                    issues += usize::from(item.species != *species);
                }
                issues += usize::from(!parented.contains(id));
            }
            curious::simulation::SimulationEvent::EntityAte { id, .. }
            | curious::simulation::SimulationEvent::EntityDrank { id, .. }
            | curious::simulation::SimulationEvent::EntityMoved { id, .. }
                if !audit.ledger.contains_key(id) =>
            {
                issues += 1
            }
            curious::simulation::SimulationEvent::EntityMetabolized {
                id,
                vision_cost,
                stamina_cost,
                vision_paid,
                stamina_paid,
            } => {
                if vision_paid > vision_cost || stamina_paid > stamina_cost {
                    issues += 1;
                }
                if !audit.ledger.contains_key(id) {
                    issues += 1;
                }
            }
            _ => {}
        }
    }
    issues
}

fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 8 {
        return Err("usage: ecology_validity_lab SCENARIO.json SEED TICKS MATE_RENDEZVOUS RESERVE_OPPORTUNITIES NEW_OUTPUT.json OUTPUT_BYTE_LIMIT".into());
    }
    let seed: u64 = args[2].parse()?;
    let ticks: usize = args[3].parse()?;
    let mate_rendezvous: bool = args[4].parse()?;
    let reserve_opportunities: bool = args[5].parse()?;
    let output_byte_limit: u64 = args[7].parse()?;
    if ticks == 0 || ticks > MAX_TICKS {
        return Err(format!("ticks must be 1..={MAX_TICKS}").into());
    }
    if output_byte_limit == 0 || output_byte_limit > MAX_RAW_BYTES {
        return Err(format!("output byte limit must be 1..={MAX_RAW_BYTES}").into());
    }

    let scenario_bytes = fs::read(&args[1])?;
    let scenario: Scenario = serde_json::from_slice(&scenario_bytes)?;
    let scenario_name = scenario.name.clone();
    let founders = scenario.founders.len();
    let mut scenario = scenario;
    scenario.genetics.enabled = true;
    scenario.genetics.mutation_per_mille = 20;
    scenario.genetics.extra_traits_enabled = false;
    scenario.policy.mate_rendezvous = mate_rendezvous;
    scenario.policy.reserve_opportunities = reserve_opportunities;
    scenario.policy.water_residual_enabled = false;
    let mut world: World = scenario.build(seed)?;
    for entity in &mut world.entities {
        entity.learner.enabled = true;
        entity.learner.update_enabled = true;
        entity.memory.enabled = true;
        entity.memory.companions_enabled = true;
        entity.hunt_learner.enabled = false;
        entity.memory.refuge_outcomes_enabled = false;
    }
    let initial_state =
        serde_json::to_value(curious::simulation::checkpoint::State::capture(&world))?;
    let founder_species: BTreeMap<usize, String> = world
        .entities
        .iter()
        .map(|entity| (entity.id, format!("{:?}", entity.base.species())))
        .collect();
    let output_path = Path::new(&args[6]);
    let output_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output_path)?;
    let mut raw = BufWriter::new(output_file);
    write_raw_line(
        &mut raw,
        &json!({
            "header": {
                "schema":"curious-ecology-validity-jsonl-v1",
                "seed":seed,
                "requested_ticks":ticks,
                "scenario_name":scenario_name.clone(),
                "founders_at_tick_0":founders,
                "founder_species":founder_species,
                "initial_state":initial_state.clone(),
                "profile":{"mode":"both","companions":true,"genetics_enabled":true,"mutation_per_mille":20,"extra_traits_enabled":false,"water_learner_enabled":true,"water_updates_enabled":true,"water_residual_enabled":false,"hunt_learning_enabled":false,"refuge_outcomes_enabled":false},
                "explicit_policy":{"mate_rendezvous":mate_rendezvous,"reserve_opportunities":reserve_opportunities,"water_residual_enabled":false}
            }
        }),
        output_byte_limit,
    )?;

    let start_stock = ecology_audit::stock(&world);
    let mut previous_stock = start_stock;
    let mut previous_cells = ecology_audit::resource_cells(&world);
    let founder_ids: std::collections::BTreeSet<usize> =
        world.entities.iter().map(|e| e.id).collect();
    let mut audit = ecology_audit::Audit::default();
    audit.seed(&world);
    let mut lineage_events_retained = 0usize;
    let mut event_observations = 0usize;
    let mut adjacent_opportunities = BTreeMap::<String, usize>::new();
    let mut ticks_completed = 0usize;

    for tick in 1..=ticks {
        let pre_state = if AUDIT_FRAME_TICKS.contains(&tick) {
            Some(serde_json::to_value(
                curious::simulation::checkpoint::State::capture(&world),
            )?)
        } else {
            None
        };
        for (species, count) in eligible_adjacent_pairs(&world) {
            *adjacent_opportunities.entry(species).or_default() += count;
        }
        let events = world.tick();
        ticks_completed = tick;
        // Check before the audit retains IDs or event-derived issue rows.
        let projected_identities = audit
            .ledger
            .len()
            .checked_add(
                events
                    .iter()
                    .filter(|e| {
                        matches!(e, curious::simulation::SimulationEvent::EntityBorn { .. })
                    })
                    .count(),
            )
            .ok_or("identity count overflow")?;
        event_observations = event_observations
            .checked_add(events.len())
            .ok_or("event count overflow")?;
        let projected_issue_rows = projected_issue_rows(&audit, &world, &events);
        let tick_lineage_count = events
            .iter()
            .filter(|event| {
                matches!(
                    event,
                    curious::simulation::SimulationEvent::EntityMated { .. }
                        | curious::simulation::SimulationEvent::EntityBorn { .. }
                        | curious::simulation::SimulationEvent::EntityDied { .. }
                )
            })
            .count();
        let current_cells = ecology_audit::resource_cells(&world);
        let pending_replacements = previous_cells
            .keys()
            .chain(current_cells.keys())
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .filter(|pos| {
                let before = previous_cells.get(pos).copied().unwrap_or("empty");
                let after = current_cells.get(pos).copied().unwrap_or("empty");
                before != after && before != "empty" && after != "empty"
            })
            .count();
        let retained_details = audit.reconciliation.len()
            + audit.replacements.len()
            + audit.parent_event_issues.len()
            + audit.unattributed_actor_events.len()
            + audit.trait_payment_issues.len();
        if projected_identities > MAX_IDENTITIES
            || world.entities.len() > MAX_IDENTITIES
            || events.len() > MAX_EVENTS_PER_TICK
            || lineage_events_retained + tick_lineage_count > MAX_LINEAGE_EVENTS
            || retained_details + projected_issue_rows + 1 + pending_replacements
                > MAX_RETAINED_DETAIL_ROWS
        {
            return Err(format!("bounded capture aborted before audit at tick {tick}: identities={projected_identities}, slots={}, events_this_tick={}, retained_rows={retained_details}+{}", world.entities.len(), events.len(), projected_issue_rows + 1 + pending_replacements).into());
        }
        (previous_stock, previous_cells) =
            audit.observe_tick(tick, previous_stock, &previous_cells, &world, &events);

        let mut event_counts = BTreeMap::<String, usize>::new();
        let mut births_this_tick = BTreeMap::<String, usize>::new();
        let mut tick_lineage_events = Vec::new();
        let mut frame_events = Vec::new();
        for event in &events {
            let value = serde_json::to_value(event)?;
            if let Some(kind) = value.as_object().and_then(|object| object.keys().next()) {
                *event_counts.entry(kind.clone()).or_default() += 1;
            }
            if let curious::simulation::SimulationEvent::EntityBorn { species, .. } = event {
                *births_this_tick.entry(format!("{species:?}")).or_default() += 1;
            }
            if matches!(
                event,
                curious::simulation::SimulationEvent::EntityMated { .. }
                    | curious::simulation::SimulationEvent::EntityBorn { .. }
                    | curious::simulation::SimulationEvent::EntityDied { .. }
            ) {
                tick_lineage_events.push(json!({"tick":tick,"event":value.clone()}));
            }
            if AUDIT_FRAME_TICKS.contains(&tick) {
                frame_events.push(value.clone());
            }
        }
        lineage_events_retained += tick_lineage_count;

        let (food_stock, water_stock) = ecology_audit::stock(&world);
        let mut tick_record = json!({
            "tick":tick,
            "alive":alive(&world),
            "food_stock":food_stock,
            "water_stock":water_stock,
            "plant_food_meals_cumulative":audit.food_meals,
            "corpse_food_meals_cumulative":audit.corpse_meals,
            "water_drunk_cumulative":audit.drinks,
            "resource_renewals_cumulative":audit.resource_renewals,
            "births_by_species_this_tick":births_this_tick,
            "event_counts_by_type":event_counts,
            "lineage_events":tick_lineage_events
        });

        if let Some(pre_state) = pre_state {
            tick_record["audit_frame"] = json!({
                "tick":tick,
                "pre_state":pre_state,
                "events":frame_events,
                "post_state":serde_json::to_value(curious::simulation::checkpoint::State::capture(&world))?
            });
        }
        write_raw_line(&mut raw, &tick_record, output_byte_limit)?;

        let population = alive(&world);
        if ["Herbivore", "Carnivore"]
            .iter()
            .any(|species| population.get(*species).copied().unwrap_or(0) == 0)
        {
            break;
        }
    }

    let (food_stock, water_stock) = ecology_audit::stock(&world);
    let living_bred_descendants_by_species = audit
        .bred_ids
        .iter()
        .filter(|id| !founder_ids.contains(*id))
        .filter_map(|&id| {
            let individual = audit.ledger.get(&id)?;
            let living = world
                .entities
                .iter()
                .any(|slot| slot.id == id && slot.base.life().is_alive());
            (living && individual.generation.is_some()).then(|| format!("{:?}", individual.species))
        })
        .fold(BTreeMap::<String, usize>::new(), |mut counts, species| {
            *counts.entry(species).or_default() += 1;
            counts
        });
    let output = json!({
        "schema": "curious-ecology-validity-v1",
        "status": "fixture_validity_diagnostic_only",
        "seed": seed,
        "requested_ticks": ticks,
        "ticks_completed": ticks_completed,
        "scenario_name": scenario_name,
        "founders_at_tick_0": founders,
        "founder_species": founder_species,
        "initial_state": initial_state,
        "profile":{"mode":"both","companions":true,"genetics_enabled":true,"mutation_per_mille":20,"extra_traits_enabled":false,"water_learner_enabled":true,"water_updates_enabled":true,"water_residual_enabled":false,"hunt_learning_enabled":false,"refuge_outcomes_enabled":false},
        "explicit_policy": {"mate_rendezvous": mate_rendezvous, "reserve_opportunities": reserve_opportunities, "water_residual_enabled": false},
        "food_stock_at_start": start_stock.0,
        "water_stock_at_start": start_stock.1,
        "audit": audit.result(&world),
        "final_state": serde_json::to_value(curious::simulation::checkpoint::State::capture(&world))?,
        "living_bred_descendants_by_species": living_bred_descendants_by_species,
        "eligible_adjacent_mating_pairs_observed": adjacent_opportunities,
        "alive_at_end": alive(&world),
        "food_stock_at_end": food_stock,
        "water_stock_at_end": water_stock,
        "capture_limits": {"identity_limit": MAX_IDENTITIES, "lineage_event_limit": MAX_LINEAGE_EVENTS, "lineage_events_retained": lineage_events_retained, "event_limit_per_tick": MAX_EVENTS_PER_TICK, "event_observations_cumulative": event_observations, "aggregate_retained_detail_row_limit": MAX_RETAINED_DETAIL_ROWS}
    });
    write_raw_line(&mut raw, &json!({"summary":output}), output_byte_limit)?;
    raw.get_ref().sync_all()?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("ecology_validity_lab: {error}");
        std::process::exit(2);
    }
}
